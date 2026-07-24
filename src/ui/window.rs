//! Window composition and tab/pane session lifecycle.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use adw::prelude::*;
use gtk::glib;

use crate::pane::{Direction, Orientation, PaneId};
use crate::settings::SettingsWarning;
use crate::tab::{TabCollection, TabId};
use crate::terminal::{CloseRequest, Terminal};

pub(crate) use super::tabs_view::update_tab_collection;
use super::tabs_view::{self, TabEntry};
use super::terminal_events;
use super::theme;
use super::{actions, profile, search};

const DEFAULT_WIDTH: i32 = 900;
const DEFAULT_HEIGHT: i32 = 600;
const MINIMUM_WIDTH: i32 = 480;
const MINIMUM_HEIGHT: i32 = 320;

// ---------------------------------------------------------------------------
// Shared UI state – all callbacks hold an Rc<UiState>
// ---------------------------------------------------------------------------

pub(crate) struct UiState {
    pub(crate) tab_collection: RefCell<TabCollection>,
    pub(crate) tab_entries: RefCell<HashMap<TabId, TabEntry>>,
    pub(crate) notebook: gtk::Notebook,
    pub(super) search_bar: gtk::SearchBar,
    pub(super) search_entry: gtk::SearchEntry,
    pub(super) search_target: RefCell<Option<(TabId, PaneId)>>,
    pub(crate) status_label: gtk::Label,
    pub(crate) status_revealer: gtk::Revealer,
    pub(super) window: adw::ApplicationWindow,
    pub(crate) font_desc: RefCell<gtk::pango::FontDescription>,
    pub(crate) profiles: RefCell<Vec<crate::workspace::StartProfile>>,
    pub(crate) active_profile_id: RefCell<Option<crate::workspace::ProfileId>>,
    pub(crate) workspace_storage: RefCell<Option<crate::workspace::WorkspaceStorage>>,
    pub(crate) layout_debouncer: RefCell<crate::workspace::LayoutDebouncer>,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) fn build_main_window(application: &adw::Application) {
    // -- CSS for active pane indicator
    theme::load_pane_css();

    // -- safe defaults; the configuration file is loaded asynchronously
    let settings = crate::settings::AppSettings::default();
    let font_desc = theme::make_font_desc(&settings);

    // -- domain model
    let tab_collection = TabCollection::new();

    let first_tab_id = tab_collection.tabs()[0].id;

    // -- notebook (tab container)
    let notebook = gtk::Notebook::new();
    notebook.set_scrollable(true);
    notebook.set_show_border(false);

    let first_page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    first_page.set_hexpand(true);
    first_page.set_vexpand(true);
    let first_label = gtk::Label::new(Some("IV"));
    notebook.append_page(&first_page, Some(&first_label));

    // -- status bar
    let status_label = gtk::Label::builder()
        .halign(gtk::Align::Start)
        .margin_start(12)
        .margin_end(12)
        .margin_top(6)
        .margin_bottom(6)
        .wrap(true)
        .build();
    let status_revealer = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideDown)
        .child(&status_label)
        .build();

    // -- content layout
    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text("Suchen …")
        .build();
    let search_bar = gtk::SearchBar::builder().child(&search_entry).build();
    search_bar.connect_entry(&search_entry);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&adw::HeaderBar::new());
    content.append(&notebook);
    content.append(&search_bar);
    content.append(&status_revealer);

    // -- window
    let window = adw::ApplicationWindow::builder()
        .application(application)
        .title("IV")
        .default_width(DEFAULT_WIDTH)
        .default_height(DEFAULT_HEIGHT)
        .content(&content)
        .build();
    window.set_size_request(MINIMUM_WIDTH, MINIMUM_HEIGHT);

    // Bind SearchBar Escape handling to the window.
    search_bar.set_key_capture_widget(Some(&window));

    // -- initial TabEntry
    let mut tab_entries = HashMap::new();
    tab_entries.insert(
        first_tab_id,
        TabEntry {
            terminals: HashMap::new(),
            label: first_label,
            page: first_page,
        },
    );

    let state = Rc::new(UiState {
        tab_collection: RefCell::new(tab_collection),
        tab_entries: RefCell::new(tab_entries),
        notebook,
        search_bar,
        search_entry,
        search_target: RefCell::new(None),
        status_label,
        status_revealer,
        window: window.clone(),
        font_desc: RefCell::new(font_desc),
        profiles: RefCell::new(Vec::new()),
        active_profile_id: RefCell::new(None),
        workspace_storage: RefCell::new(None),
        layout_debouncer: RefCell::new(crate::app::startup::new_debouncer()),
    });

    // -- wire everything up
    let initial_terminals = tabs_view::rebuild_tab_page(&state, first_tab_id);
    actions::install_tab_actions(application, &state);
    actions::install_pane_actions(application, &state);
    actions::install_clipboard_actions(application, &state);
    search::install_search_actions(application, &state);
    actions::install_font_actions(application, &state);
    profile::install_profile_actions(application, &state);
    tabs_view::connect_notebook_signals(&state);
    connect_close_request(&state);
    crate::app::startup::bootstrap_workspace(Rc::clone(&state));

    window.present();

    for terminal in initial_terminals {
        terminal_events::start_terminal(&state, &terminal);
    }

    focus_active_tab(&state);
    theme::load_settings(&state);
}

// ---------------------------------------------------------------------------
// Tab manipulation
// ---------------------------------------------------------------------------

pub(super) fn create_new_tab(state: &Rc<UiState>) {
    search::close_search(state);

    let (tab_id, new_index) = {
        let mut tabs = state.tab_collection.borrow_mut();
        tabs.add()
    };

    let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page.set_hexpand(true);
    page.set_vexpand(true);
    let label = gtk::Label::new(Some("IV"));

    state
        .notebook
        .insert_page(&page, Some(&label), Some(new_index as u32));
    state.notebook.set_current_page(Some(new_index as u32));

    state.tab_entries.borrow_mut().insert(
        tab_id,
        TabEntry {
            terminals: HashMap::new(),
            label,
            page,
        },
    );

    for terminal in tabs_view::rebuild_tab_page(state, tab_id) {
        terminal_events::start_terminal(state, &terminal);
    }
    focus_active_tab(state);
}

pub(super) fn close_active_tab(state: &Rc<UiState>) {
    search::close_search(state);

    let active_id = state.tab_collection.borrow().active_id();

    if state.tab_collection.borrow().len() <= 1 {
        state.window.close();
        return;
    }

    let terminals: Vec<Terminal> = state
        .tab_entries
        .borrow()
        .get(&active_id)
        .map(|entry| entry.terminals.values().cloned().collect())
        .unwrap_or_default();

    if terminals.is_empty() {
        tabs_view::remove_tab(state, active_id);
        return;
    }

    let pending = Rc::new(RefCell::new(0u32));
    let state_weak = Rc::downgrade(state);

    for terminal in &terminals {
        let pending_clone = Rc::clone(&pending);
        let state_weak = state_weak.clone();

        let request = terminal.request_close(move || {
            let mut remaining = pending_clone.borrow_mut();
            *remaining = remaining.saturating_sub(1);
            if *remaining == 0
                && let Some(state) = state_weak.upgrade()
            {
                tabs_view::remove_tab(&state, active_id);
            }
        });

        if request == CloseRequest::Pending {
            *pending.borrow_mut() += 1;
        }
    }

    if *pending.borrow() == 0 {
        tabs_view::remove_tab(state, active_id);
    } else {
        show_status(state, "Die Shells werden beendet …");
    }
}

// ---------------------------------------------------------------------------
// Pane actions
// ---------------------------------------------------------------------------

pub(super) fn split_pane(state: &Rc<UiState>, orientation: Orientation) {
    search::close_search(state);

    let tab_id = state.tab_collection.borrow().active_id();
    {
        let mut tabs = state.tab_collection.borrow_mut();
        tabs.split_active(orientation)
    };

    for terminal in tabs_view::rebuild_tab_page(state, tab_id) {
        terminal_events::start_terminal(state, &terminal);
    }
    focus_active_tab(state);
}

pub(super) fn close_active_pane(state: &Rc<UiState>) {
    search::close_search(state);

    let tab_id = state.tab_collection.borrow().active_id();

    let is_last_pane = state
        .tab_collection
        .borrow()
        .pane_tree_for_tab(tab_id)
        .map(|t| t.is_single())
        .unwrap_or(true);

    if is_last_pane {
        // Close the tab instead.
        close_active_tab(state);
        return;
    }

    // Close the pane from the domain model first.
    let closed_pane_id = state.tab_collection.borrow().active_pane_id();
    let result = {
        let mut tabs = state.tab_collection.borrow_mut();
        tabs.close_active_pane()
    };

    if result.is_none() {
        return;
    }

    // Clean up the terminal for the closed pane.
    if let Some(entry) = state.tab_entries.borrow().get(&tab_id)
        && let Some(terminal) = entry.terminals.get(&closed_pane_id)
    {
        terminal.request_close(|| {});
    }

    // Remove the old terminal from the map immediately.
    {
        let mut entries = state.tab_entries.borrow_mut();
        if let Some(entry) = entries.get_mut(&tab_id) {
            entry.terminals.remove(&closed_pane_id);
        }
    }

    tabs_view::rebuild_tab_page(state, tab_id);
    focus_active_tab(state);
}

pub(super) fn move_pane_focus(state: &Rc<UiState>, direction: Direction) {
    search::close_search(state);

    let tab_id = state.tab_collection.borrow().active_id();
    let moved = {
        let mut tabs = state.tab_collection.borrow_mut();
        tabs.move_pane_focus(direction)
    };

    if !moved {
        return;
    }

    update_active_pane_view(state, tab_id);
    focus_active_tab(state);
}

pub(crate) fn activate_pane(state: &Rc<UiState>, tab_id: TabId, pane_id: PaneId) {
    let changed = state
        .tab_collection
        .borrow_mut()
        .pane_tree_for_tab_mut(tab_id)
        .is_some_and(|tree| tree.set_active(pane_id));
    if changed {
        update_active_pane_view(state, tab_id);
    }
}

fn update_active_pane_view(state: &Rc<UiState>, tab_id: TabId) {
    let (active_pane_id, title) = {
        let tabs = state.tab_collection.borrow();
        let Some(tree) = tabs.pane_tree_for_tab(tab_id) else {
            return;
        };
        (
            tree.active_id(),
            tree.title(tree.active_id()).unwrap_or("IV").to_owned(),
        )
    };

    if let Some(entry) = state.tab_entries.borrow().get(&tab_id) {
        entry.label.set_label(&title);
        for (&pane_id, terminal) in &entry.terminals {
            let widget = terminal.widget();
            if pane_id == active_pane_id {
                widget.add_css_class("active-pane");
            } else {
                widget.remove_css_class("active-pane");
            }
        }
    }
}

pub(super) fn next_tab(state: &Rc<UiState>) {
    state.tab_collection.borrow_mut().next();
    let index = state.tab_collection.borrow().active_index();
    state.notebook.set_current_page(Some(index as u32));
    focus_active_tab(state);
}

pub(super) fn previous_tab(state: &Rc<UiState>) {
    state.tab_collection.borrow_mut().prev();
    let index = state.tab_collection.borrow().active_index();
    state.notebook.set_current_page(Some(index as u32));
    focus_active_tab(state);
}

pub(super) fn focus_active_tab(state: &Rc<UiState>) {
    if let Some(terminal) = active_terminal(state) {
        let t = terminal.clone();
        glib::idle_add_local_once(move || t.focus());
    }
}

pub(super) fn active_terminal(state: &Rc<UiState>) -> Option<Terminal> {
    let tabs = state.tab_collection.borrow();
    let tab_id = tabs.active_id();
    let pane_id = tabs.active_pane_id();
    drop(tabs);

    terminal_for(state, tab_id, pane_id)
}

pub(super) fn terminal_for(
    state: &Rc<UiState>,
    tab_id: TabId,
    pane_id: PaneId,
) -> Option<Terminal> {
    state
        .tab_entries
        .borrow()
        .get(&tab_id)
        .and_then(|entry| entry.terminals.get(&pane_id).cloned())
}

pub(crate) fn attach_pane_focus_handler(
    widget: &gtk::Widget,
    state: Weak<UiState>,
    tab_id: TabId,
    pane_id: PaneId,
) {
    widget.connect_has_focus_notify(move |widget| {
        if widget.has_focus()
            && let Some(state) = state.upgrade()
        {
            activate_pane(&state, tab_id, pane_id);
        }
    });
}

// ---------------------------------------------------------------------------
// Window close: coordinate graceful shutdown of all tabs
// ---------------------------------------------------------------------------

fn connect_close_request(state: &Rc<UiState>) {
    let state = Rc::clone(state);
    let window = state.window.clone();

    window.connect_close_request(move |window| {
        // Persist the current layout before any shell termination.
        crate::app::startup::save_layout_now(&state);

        let all_terminals: Vec<Terminal> = state
            .tab_entries
            .borrow()
            .values()
            .flat_map(|entry| entry.terminals.values().cloned())
            .collect();

        if all_terminals.is_empty() {
            return glib::Propagation::Proceed;
        }

        let pending = Rc::new(RefCell::new(0u32));
        let mut any_pending = false;
        let window_weak = window.downgrade();

        for terminal in all_terminals {
            let pending_for_callback = Rc::clone(&pending);
            let window_weak = window_weak.clone();

            let request = terminal.request_close(move || {
                let mut remaining = pending_for_callback.borrow_mut();
                *remaining = remaining.saturating_sub(1);
                if *remaining == 0
                    && let Some(w) = window_weak.upgrade()
                {
                    w.close();
                }
            });

            if request == CloseRequest::Pending {
                *pending.borrow_mut() += 1;
                any_pending = true;
            }
        }

        if any_pending {
            state.status_label.set_label("Die Shells werden beendet …");
            state.status_revealer.set_reveal_child(true);
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub(crate) fn show_settings_warnings(state: &Rc<UiState>, warnings: &[SettingsWarning]) {
    if warnings.is_empty() {
        return;
    }

    let message = warnings
        .iter()
        .map(|warning| warning.user_message())
        .collect::<Vec<_>>()
        .join(" ");
    show_status(state, &message);
}

pub(crate) fn show_status(state: &Rc<UiState>, message: &str) {
    state.status_label.set_label(message);
    state.status_revealer.set_reveal_child(true);
}

pub(crate) fn terminal_active_pane_id(state: &Rc<UiState>, tab_id: TabId) -> Option<PaneId> {
    let collection = state.tab_collection.borrow();
    collection
        .tabs()
        .iter()
        .find(|t| t.id == tab_id)
        .map(|t| t.pane_tree.active_id())
}

/// Records the active profile in the UI state.
///
/// This is purely informational for now; profile actions (T12) will
/// read this value to know which profile is active.
pub(crate) fn update_profile_state(
    state: &Rc<UiState>,
    active_profile_id: Option<crate::workspace::ProfileId>,
) {
    *state.active_profile_id.borrow_mut() = active_profile_id;
}
