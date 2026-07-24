//! Window composition and tab/pane session lifecycle.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use adw::prelude::*;
use gtk::glib;

use crate::pane::{Direction, Orientation, PaneId, PaneNode, PaneTree};
use crate::settings::{AppSettings, SettingsWarning, Theme};
use crate::tab::{TabCollection, TabId};
use crate::terminal::{CloseRequest, StartupWarning, Terminal, TerminalEvent};

use super::{actions, links, profile, search};

const DEFAULT_WIDTH: i32 = 900;
const DEFAULT_HEIGHT: i32 = 600;
const MINIMUM_WIDTH: i32 = 480;
const MINIMUM_HEIGHT: i32 = 320;

// ---------------------------------------------------------------------------
// Per-tab data
// ---------------------------------------------------------------------------

struct TabEntry {
    /// Terminal for each pane, keyed by PaneId.
    terminals: HashMap<PaneId, Terminal>,
    label: gtk::Label,
    page: gtk::Box,
}

// ---------------------------------------------------------------------------
// Shared UI state – all callbacks hold an Rc<UiState>
// ---------------------------------------------------------------------------

pub(crate) struct UiState {
    pub(crate) tab_collection: RefCell<TabCollection>,
    tab_entries: RefCell<HashMap<TabId, TabEntry>>,
    notebook: gtk::Notebook,
    pub(super) search_bar: gtk::SearchBar,
    pub(super) search_entry: gtk::SearchEntry,
    pub(super) search_target: RefCell<Option<(TabId, PaneId)>>,
    status_label: gtk::Label,
    status_revealer: gtk::Revealer,
    pub(super) window: adw::ApplicationWindow,
    font_desc: RefCell<gtk::pango::FontDescription>,
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
    load_pane_css();

    // -- safe defaults; the configuration file is loaded asynchronously
    let settings = AppSettings::default();
    let font_desc = make_font_desc(&settings);

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
    let initial_terminals = rebuild_tab_page(&state, first_tab_id);
    actions::install_tab_actions(application, &state);
    actions::install_pane_actions(application, &state);
    actions::install_clipboard_actions(application, &state);
    search::install_search_actions(application, &state);
    actions::install_font_actions(application, &state);
    profile::install_profile_actions(application, &state);
    connect_notebook_signals(&state);
    connect_close_request(&state);
    crate::app::startup::bootstrap_workspace(Rc::clone(&state));

    window.present();

    for terminal in initial_terminals {
        start_terminal(&state, &terminal);
    }

    focus_active_tab(&state);
    load_settings(&state);
}

// ---------------------------------------------------------------------------
// CSS for active pane
// ---------------------------------------------------------------------------

fn load_pane_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(".active-pane { border: 2px solid @accent_color; }");

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Build a `pango::FontDescription` from settings.
fn make_font_desc(settings: &AppSettings) -> gtk::pango::FontDescription {
    let mut desc = gtk::pango::FontDescription::from_string(&settings.font.family);
    desc.set_size((settings.font.size * gtk::pango::SCALE as f64) as i32);
    desc
}

/// Apply the color scheme preference via libadwaita's style manager.
fn apply_theme(settings: &AppSettings) {
    let manager = adw::StyleManager::default();
    let color_scheme = match settings.theme {
        Theme::System => adw::ColorScheme::Default,
        Theme::Light => adw::ColorScheme::ForceLight,
        Theme::Dark => adw::ColorScheme::ForceDark,
    };
    manager.set_color_scheme(color_scheme);
}

fn load_settings(state: &Rc<UiState>) {
    let state = Rc::downgrade(state);
    glib::MainContext::default().spawn_local(async move {
        let outcome = AppSettings::load_async().await;
        let Some(state) = state.upgrade() else {
            return;
        };

        apply_theme(&outcome.settings);
        let font_desc = make_font_desc(&outcome.settings);
        *state.font_desc.borrow_mut() = font_desc.clone();
        for terminal in state
            .tab_entries
            .borrow()
            .values()
            .flat_map(|entry| entry.terminals.values())
        {
            terminal.set_font(&font_desc);
        }
        show_settings_warnings(&state, &outcome.warnings);
    });
}

// ---------------------------------------------------------------------------
// Widget tree builder
// ---------------------------------------------------------------------------

/// Recursively builds a GTK widget tree from a [`PaneNode`].
/// Populates `terminals` with new Terminal instances for each pane leaf.
///
/// The active pane (matching `tree.active_id()`) receives the `active-pane`
/// CSS class; panes whose terminals are already in `existing` reuse them.
fn build_pane_widget(
    tree: &PaneTree,
    existing: &mut HashMap<PaneId, Terminal>,
    font_desc: &gtk::pango::FontDescription,
    state: Weak<UiState>,
    tab_id: TabId,
    new_terminals: &mut Vec<(PaneId, Terminal)>,
) -> gtk::Widget {
    let mut context = PaneBuildContext {
        active_id: tree.active_id(),
        terminals: existing,
        font_desc,
        state,
        tab_id,
        new_terminals,
    };
    build_pane_widget_rec(tree.root(), &mut context)
}

struct PaneBuildContext<'a> {
    active_id: PaneId,
    terminals: &'a mut HashMap<PaneId, Terminal>,
    font_desc: &'a gtk::pango::FontDescription,
    state: Weak<UiState>,
    tab_id: TabId,
    new_terminals: &'a mut Vec<(PaneId, Terminal)>,
}

fn build_pane_widget_rec(node: &PaneNode, context: &mut PaneBuildContext<'_>) -> gtk::Widget {
    match node {
        PaneNode::Terminal { id, .. } => {
            let existing = context.terminals.remove(id);
            let is_new = existing.is_none();
            let terminal = existing.unwrap_or_else(Terminal::new);
            let widget = terminal.widget();
            if *id == context.active_id {
                widget.add_css_class("active-pane");
            } else {
                widget.remove_css_class("active-pane");
            }
            if is_new {
                terminal.set_font(context.font_desc);
                links::attach_url_click_handler(&widget, terminal.clone(), context.state.clone());
                attach_pane_focus_handler(&widget, context.state.clone(), context.tab_id, *id);
                context.new_terminals.push((*id, terminal.clone()));
            }
            context.terminals.insert(*id, terminal);
            widget
        }
        PaneNode::Split {
            id,
            orientation,
            ratio,
            first,
            second,
        } => {
            let paned = match orientation {
                Orientation::Horizontal => gtk::Paned::new(gtk::Orientation::Horizontal),
                Orientation::Vertical => gtk::Paned::new(gtk::Orientation::Vertical),
            };
            paned.set_wide_handle(true);
            paned.set_shrink_start_child(false);
            paned.set_shrink_end_child(false);

            let ratio_state = Rc::new(Cell::new(*ratio));
            let ratio_for_resize = Rc::clone(&ratio_state);
            paned.connect_notify_local(Some("max-position"), move |paned, _| {
                let max = paned.max_position();
                if max > 0 {
                    paned.set_position((ratio_for_resize.get() * f64::from(max)).round() as i32);
                }
            });

            let ratio_for_position = Rc::clone(&ratio_state);
            let state_for_position = context.state.clone();
            let tab_id = context.tab_id;
            let split_id = *id;
            paned.connect_notify_local(Some("position"), move |paned, _| {
                let max = paned.max_position();
                if max <= 0 {
                    return;
                }
                let ratio = (f64::from(paned.position()) / f64::from(max))
                    .clamp(PaneTree::MIN_SPLIT_RATIO, PaneTree::MAX_SPLIT_RATIO);
                ratio_for_position.set(ratio);
                if let Some(state) = state_for_position.upgrade()
                    && let Some(tree) = state
                        .tab_collection
                        .borrow_mut()
                        .pane_tree_for_tab_mut(tab_id)
                {
                    tree.set_split_ratio(split_id, ratio);
                }
            });

            let first_widget = build_pane_widget_rec(first, context);
            let second_widget = build_pane_widget_rec(second, context);

            paned.set_start_child(Some(&first_widget));
            paned.set_end_child(Some(&second_widget));

            paned.upcast()
        }
    }
}

// ---------------------------------------------------------------------------
// Rebuild a tab's page after structural changes
// ---------------------------------------------------------------------------

/// Rebuilds one stable tab page and returns only newly created terminals.
fn rebuild_tab_page(state: &Rc<UiState>, tab_id: TabId) -> Vec<Terminal> {
    let pane_tree = {
        let tabs = state.tab_collection.borrow();
        let Some(tree) = tabs.pane_tree_for_tab(tab_id) else {
            return Vec::new();
        };
        tree.clone()
    };

    let page = {
        let entries = state.tab_entries.borrow();
        let Some(entry) = entries.get(&tab_id) else {
            return Vec::new();
        };
        detach_terminal_widgets(&entry.terminals);
        entry.page.clone()
    };

    // Removing and dropping the old root unparents all retained VTE widgets
    // before the new split hierarchy is built.
    while let Some(child) = page.first_child() {
        page.remove(&child);
    }

    let old_terminals = {
        let mut entries = state.tab_entries.borrow_mut();
        entries
            .get_mut(&tab_id)
            .map(|entry| std::mem::take(&mut entry.terminals))
    };
    let Some(mut terminals) = old_terminals else {
        return Vec::new();
    };

    let valid_ids = pane_tree.pane_ids();
    terminals.retain(|id, _| valid_ids.contains(id));

    let font_desc = state.font_desc.borrow().clone();
    let mut new_terminals = Vec::new();
    let root = build_pane_widget(
        &pane_tree,
        &mut terminals,
        &font_desc,
        Rc::downgrade(state),
        tab_id,
        &mut new_terminals,
    );
    page.append(&root);

    if let Some(entry) = state.tab_entries.borrow_mut().get_mut(&tab_id) {
        entry.terminals = terminals;
    }

    for (pane_id, terminal) in &new_terminals {
        connect_pane_events(Rc::downgrade(state), tab_id, *pane_id, terminal);
        connect_pane_title(Rc::downgrade(state), tab_id, *pane_id, terminal);
    }

    new_terminals
        .into_iter()
        .map(|(_, terminal)| terminal)
        .collect()
}

fn detach_terminal_widgets(terminals: &HashMap<PaneId, Terminal>) {
    for terminal in terminals.values() {
        let widget = terminal.widget();
        let Some(parent) = widget.parent() else {
            continue;
        };

        if let Ok(paned) = parent.clone().downcast::<gtk::Paned>() {
            if paned.start_child().as_ref() == Some(&widget) {
                paned.set_start_child(gtk::Widget::NONE);
            } else if paned.end_child().as_ref() == Some(&widget) {
                paned.set_end_child(gtk::Widget::NONE);
            }
        } else if let Ok(container) = parent.downcast::<gtk::Box>() {
            container.remove(&widget);
        }
    }
}

// ---------------------------------------------------------------------------
// Connecting terminal events for a single pane
// ---------------------------------------------------------------------------

fn connect_pane_events(state: Weak<UiState>, tab_id: TabId, pane_id: PaneId, terminal: &Terminal) {
    terminal.set_event_handler(Rc::new(move |event| match event {
        TerminalEvent::Started => {}
        TerminalEvent::SpawnFailed(error) => {
            if let Some(state) = state.upgrade() {
                show_status(&state, error.user_message());
            }
        }
        TerminalEvent::Exited(exit) if exit.successful() => {
            if let Some(state) = state.upgrade() {
                handle_pane_exited_success(&state, tab_id, pane_id);
            }
        }
        TerminalEvent::Exited(exit) => {
            if let Some(state) = state.upgrade() {
                handle_pane_exited_error(&state, tab_id, pane_id, &exit.user_message());
            }
        }
    }));
}

fn connect_pane_title(state: Weak<UiState>, tab_id: TabId, pane_id: PaneId, terminal: &Terminal) {
    terminal.connect_title_changed(move |title| {
        let Some(state) = state.upgrade() else {
            return;
        };
        if let Some(tree) = state
            .tab_collection
            .borrow_mut()
            .pane_tree_for_tab_mut(tab_id)
        {
            tree.set_title(pane_id, title.to_owned());
        }

        // Update the tab label if this is the active pane.
        if let Some(entry) = state.tab_entries.borrow().get(&tab_id) {
            let is_active = state
                .tab_collection
                .borrow()
                .pane_tree_for_tab(tab_id)
                .map(|t| t.active_id() == pane_id)
                .unwrap_or(false);
            if is_active {
                entry.label.set_label(title);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Pane event handlers
// ---------------------------------------------------------------------------

fn handle_pane_exited_success(state: &Rc<UiState>, tab_id: TabId, pane_id: PaneId) {
    let should_close_tab = {
        let tabs = state.tab_collection.borrow();
        let tree = match tabs.pane_tree_for_tab(tab_id) {
            Some(t) => t,
            None => return,
        };
        // If this is the only pane in the tab, close the tab.
        tree.is_single()
    };

    if should_close_tab {
        let was_last = state.tab_collection.borrow().len() <= 1;
        if was_last {
            state.window.close();
        } else {
            remove_tab(state, tab_id);
        }
    } else {
        let result = {
            let mut tabs = state.tab_collection.borrow_mut();
            if let Some(tree) = tabs.pane_tree_for_tab_mut(tab_id) {
                tree.close(pane_id)
            } else {
                None
            }
        };
        if result.is_some() {
            rebuild_tab_page(state, tab_id);
        }
    }
}

fn handle_pane_exited_error(state: &Rc<UiState>, tab_id: TabId, pane_id: PaneId, message: &str) {
    // Show error if this is the active pane of the active tab.
    let is_active_tab = state.tab_collection.borrow().active_id() == tab_id;
    let is_active_pane = state
        .tab_collection
        .borrow()
        .pane_tree_for_tab(tab_id)
        .map(|t| t.active_id() == pane_id)
        .unwrap_or(false);

    if is_active_tab && is_active_pane {
        show_status(state, message);
    }

    // Update the tab label if this was the active pane.
    if is_active_pane && let Some(entry) = state.tab_entries.borrow().get(&tab_id) {
        entry
            .label
            .set_label(&format!("[beendet] {}", entry.label.label()));
    }
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

    for terminal in rebuild_tab_page(state, tab_id) {
        start_terminal(state, &terminal);
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
        remove_tab(state, active_id);
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
                remove_tab(&state, active_id);
            }
        });

        if request == CloseRequest::Pending {
            *pending.borrow_mut() += 1;
        }
    }

    if *pending.borrow() == 0 {
        remove_tab(state, active_id);
    } else {
        show_status(state, "Die Shells werden beendet …");
    }
}

fn remove_tab(state: &Rc<UiState>, tab_id: TabId) {
    let index = state.tab_collection.borrow().find_index(tab_id);
    state.tab_collection.borrow_mut().remove(tab_id);
    state.tab_entries.borrow_mut().remove(&tab_id);
    if let Some(index) = index {
        state.notebook.remove_page(Some(index as u32));
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

    for terminal in rebuild_tab_page(state, tab_id) {
        start_terminal(state, &terminal);
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

    rebuild_tab_page(state, tab_id);
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

fn activate_pane(state: &Rc<UiState>, tab_id: TabId, pane_id: PaneId) {
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

fn attach_pane_focus_handler(
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
// Notebook signals: sync active tab on switch, focus newly visible terminal
// ---------------------------------------------------------------------------

fn connect_notebook_signals(state: &Rc<UiState>) {
    let state_switch = Rc::clone(state);
    state
        .notebook
        .connect_switch_page(move |_notebook, _page, page_num| {
            search::close_search(&state_switch);
            state_switch
                .tab_collection
                .borrow_mut()
                .set_active(page_num as usize);
            focus_active_tab(&state_switch);
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

fn start_terminal(state: &Rc<UiState>, terminal: &Terminal) {
    match terminal.start() {
        Ok(warnings) => show_warnings(state, &warnings),
        Err(error) => show_status(state, error.user_message()),
    }
}

fn show_warnings(state: &Rc<UiState>, warnings: &[StartupWarning]) {
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

fn show_settings_warnings(state: &Rc<UiState>, warnings: &[SettingsWarning]) {
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

// ---------------------------------------------------------------------------
// Workspace integration: replace tabs and apply profiles
// ---------------------------------------------------------------------------

/// Replaces the tab collection in the UI with the loaded snapshot.
///
/// This is used by the workspace bootstrap to restore the last layout.
/// The existing notebook pages are removed, the new tabs are
/// inserted, and terminals are started for each restored pane.
pub(crate) fn update_tab_collection(state: &Rc<UiState>, new_collection: TabCollection) {
    // Detach existing notebook pages.
    let existing_pages: Vec<(TabId, gtk::Box, gtk::Label)> = {
        let entries = state.tab_entries.borrow();
        entries
            .iter()
            .map(|(id, entry)| (*id, entry.page.clone(), entry.label.clone()))
            .collect()
    };
    for (_, page, _) in &existing_pages {
        state.notebook.detach_tab(page);
    }
    state.tab_entries.borrow_mut().clear();
    *state.tab_collection.borrow_mut() = new_collection;

    // Build fresh tab pages for every restored tab.
    let new_tabs: Vec<TabId> = state
        .tab_collection
        .borrow()
        .tabs()
        .iter()
        .map(|tab| tab.id)
        .collect();
    let mut new_terminals_to_start: Vec<Terminal> = Vec::new();
    let mut new_terminal_specs: Vec<(
        TabId,
        PaneId,
        Terminal,
        Option<crate::workspace::StartConfig>,
    )> = Vec::new();

    for tab_id in new_tabs {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
        page.set_hexpand(true);
        page.set_vexpand(true);
        let title = {
            let collection = state.tab_collection.borrow();
            collection
                .tabs()
                .iter()
                .find(|t| t.id == tab_id)
                .map(|t| t.custom_title.clone().unwrap_or_else(|| t.title.clone()))
                .unwrap_or_else(|| "IV".to_owned())
        };
        let label = gtk::Label::new(Some(&title));
        state.notebook.append_page(&page, Some(&label));
        state.tab_entries.borrow_mut().insert(
            tab_id,
            TabEntry {
                terminals: HashMap::new(),
                label: label.clone(),
                page: page.clone(),
            },
        );

        let (pane_ids, start_config) = {
            let collection = state.tab_collection.borrow();
            let Some(tab) = collection.tabs().iter().find(|t| t.id == tab_id) else {
                continue;
            };
            (tab.pane_tree.pane_ids(), tab.start_config.clone())
        };

        let new_terminals = rebuild_tab_page(state, tab_id);
        for terminal in new_terminals {
            new_terminals_to_start.push(terminal.clone());
            if let Some(pane_id) = terminal_active_pane_id(state, tab_id) {
                new_terminal_specs.push((tab_id, pane_id, terminal, start_config.clone()));
            }
        }
        let _ = pane_ids; // currently informational; could be used for diagnostics
    }

    // Restore active tab index.
    let active_index = state.tab_collection.borrow().active_index();
    if active_index < state.notebook.n_pages() as usize {
        state.notebook.set_current_page(Some(active_index as u32));
    }

    focus_active_tab(state);

    for terminal in &new_terminals_to_start {
        start_terminal(state, terminal);
    }
    let _ = new_terminal_specs;
}

fn terminal_active_pane_id(state: &Rc<UiState>, tab_id: TabId) -> Option<PaneId> {
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
