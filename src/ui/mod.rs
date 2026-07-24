use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::pane::{Direction, Orientation, PaneId, PaneNode, PaneTree};
use crate::settings::AppSettings;
use crate::tab::{TabCollection, TabId};
use crate::terminal::{StartupWarning, Terminal, TerminalEvent};

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
}

// ---------------------------------------------------------------------------
// Shared UI state – all callbacks hold an Rc<UiState>
// ---------------------------------------------------------------------------

struct UiState {
    tab_collection: RefCell<TabCollection>,
    tab_entries: RefCell<HashMap<TabId, TabEntry>>,
    notebook: gtk::Notebook,
    search_bar: gtk::SearchBar,
    search_entry: gtk::Entry,
    status_label: gtk::Label,
    status_revealer: gtk::Revealer,
    window: adw::ApplicationWindow,
    #[allow(dead_code)]
    settings: AppSettings,
    font_desc: gtk::pango::FontDescription,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) fn build_main_window(application: &adw::Application) {
    // -- CSS for active pane indicator
    load_pane_css();

    // -- settings
    let settings = AppSettings::load();
    apply_theme(&settings);
    let font_desc = make_font_desc(&settings);

    // -- domain model
    let tab_collection = TabCollection::new();

    // -- first terminal
    let first_terminal = Terminal::new();
    let first_tab_id = tab_collection.tabs()[0].id;
    let first_pane_id = first_terminal_id(&tab_collection);

    // -- notebook (tab container)
    let notebook = gtk::Notebook::new();
    notebook.set_scrollable(true);
    notebook.set_show_border(false);

    // Build initial tab page from its PaneTree
    let first_page = build_pane_widget(
        &tab_collection.tabs()[0].pane_tree,
        &mut HashMap::new(),
        &font_desc,
    );
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
    let search_entry = gtk::Entry::builder().placeholder_text("Suchen …").build();
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
    let mut terminals = HashMap::new();
    terminals.insert(first_pane_id, first_terminal.clone());

    let mut tab_entries = HashMap::new();
    tab_entries.insert(
        first_tab_id,
        TabEntry {
            terminals,
            label: first_label,
        },
    );

    let state = Rc::new(UiState {
        tab_collection: RefCell::new(tab_collection),
        tab_entries: RefCell::new(tab_entries),
        notebook,
        search_bar,
        search_entry,
        status_label,
        status_revealer,
        window: window.clone(),
        settings,
        font_desc,
    });

    // -- wire everything up
    setup_first_tab(&state, first_tab_id);
    install_tab_actions(application, &state);
    install_pane_actions(application, &state);
    install_clipboard_actions(application, &state);
    install_search_actions(application, &state);
    install_font_actions(application, &state);
    connect_notebook_signals(&state);
    connect_close_request(&state);

    window.present();

    // -- start the first shell
    match first_terminal.start() {
        Ok(warnings) => show_warnings(&state, &warnings),
        Err(error) => show_status(&state, error.user_message()),
    }

    let first_terminal_clone = first_terminal.clone();
    glib::idle_add_local_once(move || first_terminal_clone.focus());
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
    let color_scheme = match settings.appearance.theme.as_str() {
        "light" => adw::ColorScheme::ForceLight,
        "dark" => adw::ColorScheme::ForceDark,
        _ => adw::ColorScheme::Default,
    };
    manager.set_color_scheme(color_scheme);
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
) -> gtk::Widget {
    let active = tree.active_id();
    build_pane_widget_rec(tree.root(), active, existing, font_desc)
}

/// Returns a reference to the root PaneNode. This is a small helper to avoid
/// making `PaneNode` fields public.
fn build_pane_widget_rec(
    node: &PaneNode,
    active_id: PaneId,
    terminals: &mut HashMap<PaneId, Terminal>,
    font_desc: &gtk::pango::FontDescription,
) -> gtk::Widget {
    match node {
        PaneNode::Terminal { id, .. } => {
            let existing = terminals.remove(id);
            let is_new = existing.is_none();
            let terminal = existing.unwrap_or_else(Terminal::new);
            let widget = terminal.widget();
            if *id == active_id {
                widget.add_css_class("active-pane");
            }
            if is_new {
                terminal.set_font(font_desc);
                attach_url_click_handler(&widget, terminal.clone());
            }
            terminals.insert(*id, terminal);
            widget
        }
        PaneNode::Split {
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
            paned.set_position((*ratio * 1000.0) as i32);

            let first_widget = build_pane_widget_rec(first, active_id, terminals, font_desc);
            let second_widget = build_pane_widget_rec(second, active_id, terminals, font_desc);

            paned.set_start_child(Some(&first_widget));
            paned.set_end_child(Some(&second_widget));

            paned.upcast()
        }
    }
}

// ---------------------------------------------------------------------------
// Rebuild a tab's page after structural changes
// ---------------------------------------------------------------------------

/// Rebuilds the notebook page for the given tab from its PaneTree.
/// Updates the TabEntry's terminal map to match. Returns the new root widget.
fn rebuild_tab_page(state: &Rc<UiState>, tab_id: TabId) -> Option<gtk::Widget> {
    let pane_tree = {
        let tabs = state.tab_collection.borrow();
        tabs.pane_tree_for_tab(tab_id)?.clone()
    };

    let index = state.tab_collection.borrow().find_index(tab_id)?;

    // Take existing terminals out of the entry so we can reuse them.
    let old_terminals = {
        let mut entries = state.tab_entries.borrow_mut();
        entries
            .get_mut(&tab_id)
            .map(|entry| std::mem::take(&mut entry.terminals))
    }?;

    let mut terminals = old_terminals;
    // Remove panes that no longer exist from the map.
    let valid_ids = pane_tree.pane_ids();
    terminals.retain(|id, _| valid_ids.contains(id));

    let page = build_pane_widget(&pane_tree, &mut terminals, &state.font_desc);

    // Replace the notebook page. We need to remove the old page and insert the new one.
    // gtk::Notebook doesn't have replace_page, so remove + insert.
    let notebook = state.notebook.clone();
    // Get the tab label before removing.
    let label = {
        let entries = state.tab_entries.borrow();
        entries
            .get(&tab_id)
            .map(|e| e.label.clone())
            .unwrap_or_else(|| gtk::Label::new(Some("IV")))
    };

    notebook.remove_page(Some(index as u32));
    notebook.insert_page(&page, Some(&label), Some(index as u32));
    notebook.set_current_page(Some(index as u32));

    // Update TabEntry.
    {
        let mut entries = state.tab_entries.borrow_mut();
        if let Some(entry) = entries.get_mut(&tab_id) {
            entry.terminals = terminals;
        }
    }

    // Wire up events for any newly created terminals.
    wire_tab_terminals(state, tab_id);

    Some(page)
}

// ---------------------------------------------------------------------------
// Terminal event wiring for all panes in a tab
// ---------------------------------------------------------------------------

fn wire_tab_terminals(state: &Rc<UiState>, tab_id: TabId) {
    let entries = state.tab_entries.borrow();
    if let Some(entry) = entries.get(&tab_id) {
        for (&pane_id, terminal) in &entry.terminals {
            connect_pane_events(state, tab_id, pane_id, terminal);
            connect_pane_title(state, tab_id, pane_id, terminal);
        }
    }
}

// ---------------------------------------------------------------------------
// Connecting terminal events for a single pane
// ---------------------------------------------------------------------------

fn connect_pane_events(state: &Rc<UiState>, tab_id: TabId, pane_id: PaneId, terminal: &Terminal) {
    let state = Rc::clone(state);
    terminal.set_event_handler(Rc::new(move |event| match event {
        TerminalEvent::Started => {}
        TerminalEvent::SpawnFailed(error) => {
            show_status(&state, error.user_message());
        }
        TerminalEvent::Exited(exit) if exit.successful() => {
            handle_pane_exited_success(&state, tab_id, pane_id);
        }
        TerminalEvent::Exited(exit) => {
            handle_pane_exited_error(&state, tab_id, pane_id, &exit.user_message());
        }
    }));
}

fn connect_pane_title(state: &Rc<UiState>, tab_id: TabId, pane_id: PaneId, terminal: &Terminal) {
    let state = Rc::clone(state);
    terminal.connect_title_changed(move |title| {
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
// First tab wiring
// ---------------------------------------------------------------------------

fn setup_first_tab(state: &Rc<UiState>, tab_id: TabId) {
    wire_tab_terminals(state, tab_id);
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
        // Close the pane from the tree and rebuild.
        let result = {
            let mut tabs = state.tab_collection.borrow_mut();
            if let Some(tree) = tabs.pane_tree_for_tab_mut(tab_id) {
                tree.set_active(pane_id);
                tree.close_active()
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

fn create_new_tab(state: &Rc<UiState>) {
    close_search(state);

    let (tab_id, new_index) = {
        let mut tabs = state.tab_collection.borrow_mut();
        tabs.add()
    };

    let pane_tree = state
        .tab_collection
        .borrow()
        .pane_tree_for_tab(tab_id)
        .cloned()
        .unwrap();

    let mut terminals = HashMap::new();
    let page = build_pane_widget(&pane_tree, &mut terminals, &state.font_desc);
    let label = gtk::Label::new(Some("IV"));

    state
        .notebook
        .insert_page(&page, Some(&label), Some(new_index as u32));
    state.notebook.set_current_page(Some(new_index as u32));

    state
        .tab_entries
        .borrow_mut()
        .insert(tab_id, TabEntry { terminals, label });

    wire_tab_terminals(state, tab_id);

    // Start the first terminal.
    if let Some(terminal) = active_terminal(state) {
        match terminal.start() {
            Ok(warnings) => show_warnings(state, &warnings),
            Err(error) => show_status(state, error.user_message()),
        }
        let t = terminal.clone();
        glib::idle_add_local_once(move || t.focus());
    }
}

fn close_active_tab(state: &Rc<UiState>) {
    close_search(state);

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

        let immediate = terminal.request_close(move || {
            let mut remaining = pending_clone.borrow_mut();
            *remaining = remaining.saturating_sub(1);
            if *remaining == 0
                && let Some(state) = state_weak.upgrade()
            {
                remove_tab(&state, active_id);
            }
        });

        if !immediate {
            *pending.borrow_mut() += 1;
        }
    }
}

fn remove_tab(state: &Rc<UiState>, tab_id: TabId) {
    if let Some(index) = state.tab_collection.borrow().find_index(tab_id) {
        state.notebook.remove_page(Some(index as u32));
    }
    state.tab_collection.borrow_mut().remove(tab_id);
    state.tab_entries.borrow_mut().remove(&tab_id);
}

// ---------------------------------------------------------------------------
// Pane actions
// ---------------------------------------------------------------------------

fn split_pane(state: &Rc<UiState>, orientation: Orientation) {
    close_search(state);

    let tab_id = state.tab_collection.borrow().active_id();
    let _new_pane_id = {
        let mut tabs = state.tab_collection.borrow_mut();
        tabs.split_active(orientation)
    };

    let _old_ids = {
        // Collect existing terminal ids so we know which is new.
        state
            .tab_entries
            .borrow()
            .get(&tab_id)
            .map(|e| e.terminals.keys().copied().collect::<Vec<_>>())
            .unwrap_or_default()
    };

    // Rebuild the tab page – this will create a new Terminal for the new pane.
    if let Some(_page) = rebuild_tab_page(state, tab_id) {
        // Start the new terminal (the one whose PaneId wasn't in old_ids).
        if let Some(new_terminal) = active_terminal(state) {
            match new_terminal.start() {
                Ok(warnings) => show_warnings(state, &warnings),
                Err(error) => show_status(state, error.user_message()),
            }
            let t = new_terminal.clone();
            glib::idle_add_local_once(move || t.focus());
        }
    }
}

fn close_active_pane(state: &Rc<UiState>) {
    close_search(state);

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
        let state_weak = Rc::downgrade(state);
        terminal.request_close(move || {
            if let Some(state) = state_weak.upgrade() {
                // Remove the terminal from the entry.
                state
                    .tab_entries
                    .borrow_mut()
                    .get_mut(&tab_id)
                    .map(|e| e.terminals.remove(&closed_pane_id));
            }
        });
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

fn move_pane_focus(state: &Rc<UiState>, direction: Direction) {
    let tab_id = state.tab_collection.borrow().active_id();
    let moved = {
        let mut tabs = state.tab_collection.borrow_mut();
        tabs.move_pane_focus(direction)
    };

    if !moved {
        return;
    }

    // Close search on pane focus change.
    close_search(state);

    // Update the active-pane CSS class on the widgets.
    let active_pane_id = state
        .tab_collection
        .borrow()
        .pane_tree_for_tab(tab_id)
        .map(|t| t.active_id())
        .unwrap();

    if let Some(entry) = state.tab_entries.borrow().get(&tab_id) {
        for (&pane_id, terminal) in &entry.terminals {
            let widget = terminal.widget();
            if pane_id == active_pane_id {
                widget.add_css_class("active-pane");
            } else {
                widget.remove_css_class("active-pane");
            }
        }
    }

    focus_active_tab(state);
}

// ---------------------------------------------------------------------------
// Actions: new-tab, close-tab, next-tab, prev-tab
// ---------------------------------------------------------------------------

fn install_tab_actions(application: &adw::Application, state: &Rc<UiState>) {
    let state_new = Rc::clone(state);
    let new_tab_action = gio::SimpleAction::new("new-tab", None);
    new_tab_action.connect_activate(move |_, _| create_new_tab(&state_new));
    state.window.add_action(&new_tab_action);

    let state_close = Rc::clone(state);
    let close_tab_action = gio::SimpleAction::new("close-tab", None);
    close_tab_action.connect_activate(move |_, _| close_active_tab(&state_close));
    state.window.add_action(&close_tab_action);

    let state_next = Rc::clone(state);
    let next_tab_action = gio::SimpleAction::new("next-tab", None);
    next_tab_action.connect_activate(move |_, _| {
        state_next.tab_collection.borrow_mut().next();
        let idx = state_next.tab_collection.borrow().active_index();
        state_next.notebook.set_current_page(Some(idx as u32));
        focus_active_tab(&state_next);
    });
    state.window.add_action(&next_tab_action);

    let state_prev = Rc::clone(state);
    let prev_tab_action = gio::SimpleAction::new("prev-tab", None);
    prev_tab_action.connect_activate(move |_, _| {
        state_prev.tab_collection.borrow_mut().prev();
        let idx = state_prev.tab_collection.borrow().active_index();
        state_prev.notebook.set_current_page(Some(idx as u32));
        focus_active_tab(&state_prev);
    });
    state.window.add_action(&prev_tab_action);

    application.set_accels_for_action("win.new-tab", &["<Control><Shift>t"]);
    application.set_accels_for_action("win.close-tab", &["<Control><Shift>w"]);
    application.set_accels_for_action("win.next-tab", &["<Control>Page_Down"]);
    application.set_accels_for_action("win.prev-tab", &["<Control>Page_Up"]);
}

// ---------------------------------------------------------------------------
// Actions: pane split, close, focus
// ---------------------------------------------------------------------------

fn install_pane_actions(application: &adw::Application, state: &Rc<UiState>) {
    // Split horizontally (side by side)
    let state_h = Rc::clone(state);
    let split_h = gio::SimpleAction::new("split-horizontal", None);
    split_h.connect_activate(move |_, _| split_pane(&state_h, Orientation::Horizontal));
    state.window.add_action(&split_h);

    // Split vertically (stacked)
    let state_v = Rc::clone(state);
    let split_v = gio::SimpleAction::new("split-vertical", None);
    split_v.connect_activate(move |_, _| split_pane(&state_v, Orientation::Vertical));
    state.window.add_action(&split_v);

    // Close active pane
    let state_c = Rc::clone(state);
    let close_pane = gio::SimpleAction::new("close-pane", None);
    close_pane.connect_activate(move |_, _| close_active_pane(&state_c));
    state.window.add_action(&close_pane);

    // Focus navigation
    for (name, direction) in &[
        ("focus-left", Direction::Left),
        ("focus-right", Direction::Right),
        ("focus-up", Direction::Up),
        ("focus-down", Direction::Down),
    ] {
        let state_f = Rc::clone(state);
        let dir = *direction;
        let action = gio::SimpleAction::new(name, None);
        action.connect_activate(move |_, _| move_pane_focus(&state_f, dir));
        state.window.add_action(&action);
    }

    application.set_accels_for_action("win.split-horizontal", &["<Control><Shift>Right"]);
    application.set_accels_for_action("win.split-vertical", &["<Control><Shift>Down"]);
    application.set_accels_for_action("win.close-pane", &["<Control><Shift>q"]);
    application.set_accels_for_action("win.focus-left", &["<Alt>Left"]);
    application.set_accels_for_action("win.focus-right", &["<Alt>Right"]);
    application.set_accels_for_action("win.focus-up", &["<Alt>Up"]);
    application.set_accels_for_action("win.focus-down", &["<Alt>Down"]);
}

fn focus_active_tab(state: &Rc<UiState>) {
    if let Some(terminal) = active_terminal(state) {
        let t = terminal.clone();
        glib::idle_add_local_once(move || t.focus());
    }
}

fn active_terminal(state: &Rc<UiState>) -> Option<Terminal> {
    let tabs = state.tab_collection.borrow();
    let tab_id = tabs.active_id();
    let pane_id = tabs.active_pane_id();
    drop(tabs);

    state
        .tab_entries
        .borrow()
        .get(&tab_id)
        .and_then(|entry| entry.terminals.get(&pane_id).cloned())
}

// ---------------------------------------------------------------------------
// Clipboard actions (copy / paste on the active tab)
// ---------------------------------------------------------------------------

fn install_clipboard_actions(application: &adw::Application, state: &Rc<UiState>) {
    let state_copy = Rc::clone(state);
    let copy_action = gio::SimpleAction::new("copy", None);
    copy_action.connect_activate(move |_, _| {
        if let Some(terminal) = active_terminal(&state_copy)
            && let Err(error) = terminal.copy()
        {
            show_status(&state_copy, error.user_message());
        }
    });
    state.window.add_action(&copy_action);

    let state_paste = Rc::clone(state);
    let paste_action = gio::SimpleAction::new("paste", None);
    paste_action.connect_activate(move |_, _| {
        if let Some(terminal) = active_terminal(&state_paste)
            && let Err(error) = terminal.paste()
        {
            show_status(&state_paste, error.user_message());
        }
    });
    state.window.add_action(&paste_action);

    application.set_accels_for_action("win.copy", &["<Control><Shift>c"]);
    application.set_accels_for_action("win.paste", &["<Control><Shift>v"]);
}

// ---------------------------------------------------------------------------
// Search bar
// ---------------------------------------------------------------------------

fn install_search_actions(application: &adw::Application, state: &Rc<UiState>) {
    // Ctrl+Shift+F toggles the search bar.
    let state_search = Rc::clone(state);
    let toggle_action = gio::SimpleAction::new("toggle-search", None);
    toggle_action.connect_activate(move |_, _| toggle_search(&state_search));
    state.window.add_action(&toggle_action);
    application.set_accels_for_action("win.toggle-search", &["<Control><Shift>f"]);

    // Enter in the search entry → find next.
    let state_enter = Rc::clone(state);
    state.search_entry.connect_activate(move |_entry| {
        if let Some(terminal) = active_terminal(&state_enter) {
            terminal.search_next();
        }
    });

    // Text changed → execute search in active pane.
    let state_changed = Rc::clone(state);
    state.search_entry.connect_changed(move |entry| {
        let query = entry.text();
        if let Some(terminal) = active_terminal(&state_changed)
            && let Err(error) = terminal.search(query.as_ref())
        {
            show_status(&state_changed, error.user_message());
        }
    });

    // Shift+Enter → find previous. We use a key controller on the entry.
    let state_prev = Rc::clone(state);
    let key_controller = gtk::EventControllerKey::new();
    key_controller.connect_key_pressed(move |_controller, keyval, _keycode, modifiers| {
        if keyval == gtk::gdk::Key::Return && modifiers.contains(gtk::gdk::ModifierType::SHIFT_MASK)
        {
            if let Some(terminal) = active_terminal(&state_prev) {
                terminal.search_previous();
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    state.search_entry.add_controller(key_controller);

    // When the search bar is hidden, clear the search.
    let state_hide = Rc::clone(state);
    state
        .search_bar
        .connect_notify_local(Some("search-mode-enabled"), move |bar, _pspec| {
            if !bar.is_search_mode() {
                if let Some(terminal) = active_terminal(&state_hide) {
                    terminal.search_clear();
                }
                // Refocus the terminal.
                focus_active_tab(&state_hide);
            }
        });
}

fn toggle_search(state: &Rc<UiState>) {
    if state.search_bar.is_search_mode() {
        state.search_bar.set_search_mode(false);
    } else {
        state.search_bar.set_search_mode(true);
        state.search_entry.grab_focus();
    }
}

fn close_search(state: &Rc<UiState>) {
    if state.search_bar.is_search_mode() {
        state.search_bar.set_search_mode(false);
        // The notify signal handler above will clear search and refocus.
    }
}

// ---------------------------------------------------------------------------
// Font zoom actions (Ctrl++ / Ctrl+- / Ctrl+0)
// ---------------------------------------------------------------------------

fn install_font_actions(application: &adw::Application, state: &Rc<UiState>) {
    let state_zoom_in = Rc::clone(state);
    let zoom_in = gio::SimpleAction::new("zoom-in", None);
    zoom_in.connect_activate(move |_, _| {
        if let Some(terminal) = active_terminal(&state_zoom_in) {
            terminal.zoom_font(0.1);
        }
    });
    state.window.add_action(&zoom_in);

    let state_zoom_out = Rc::clone(state);
    let zoom_out = gio::SimpleAction::new("zoom-out", None);
    zoom_out.connect_activate(move |_, _| {
        if let Some(terminal) = active_terminal(&state_zoom_out) {
            terminal.zoom_font(-0.1);
        }
    });
    state.window.add_action(&zoom_out);

    let state_zoom_reset = Rc::clone(state);
    let zoom_reset = gio::SimpleAction::new("zoom-reset", None);
    zoom_reset.connect_activate(move |_, _| {
        if let Some(terminal) = active_terminal(&state_zoom_reset) {
            terminal.reset_font_zoom();
        }
    });
    state.window.add_action(&zoom_reset);

    application.set_accels_for_action("win.zoom-in", &["<Control>plus", "<Control>equal"]);
    application.set_accels_for_action("win.zoom-out", &["<Control>minus"]);
    application.set_accels_for_action("win.zoom-reset", &["<Control>0"]);
}

/// Attach a click gesture that opens hyperlinks via Ctrl+Click.
fn attach_url_click_handler(widget: &gtk::Widget, terminal: Terminal) {
    let click = gtk::GestureClick::new();
    click.set_button(1); // left button only

    click.connect_pressed(move |gesture, _n_press, x, y| {
        // Only open on Ctrl+Click.
        let modifiers = gesture.current_event_state();
        if !modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
            return;
        }

        if let Some(uri) = terminal.hyperlink_at(x, y) {
            open_uri(&uri);
        }
    });

    widget.add_controller(click);
}

/// Open a URI with the default handler.
fn open_uri(uri: &str) {
    if let Err(err) = gio::AppInfo::launch_default_for_uri(uri, gio::AppLaunchContext::NONE) {
        eprintln!("IV: Konnte URI nicht öffnen: {err}");
    }
}

// ---------------------------------------------------------------------------
// Notebook signals: sync active tab on switch, focus newly visible terminal
// ---------------------------------------------------------------------------

fn connect_notebook_signals(state: &Rc<UiState>) {
    let state_switch = Rc::clone(state);
    state
        .notebook
        .connect_switch_page(move |_notebook, _page, page_num| {
            close_search(&state_switch);
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
        let mut any_non_immediate = false;
        let window_weak = window.downgrade();

        for terminal in all_terminals {
            let pending_for_callback = Rc::clone(&pending);
            let window_weak = window_weak.clone();

            let immediate = terminal.request_close(move || {
                let mut remaining = pending_for_callback.borrow_mut();
                *remaining = remaining.saturating_sub(1);
                if *remaining == 0
                    && let Some(w) = window_weak.upgrade()
                {
                    w.close();
                }
            });

            if !immediate {
                *pending.borrow_mut() += 1;
                any_non_immediate = true;
            }
        }

        if any_non_immediate {
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

fn first_terminal_id(tabs: &TabCollection) -> PaneId {
    tabs.tabs()[0].pane_tree.active_id()
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

fn show_status(state: &Rc<UiState>, message: &str) {
    state.status_label.set_label(message);
    state.status_revealer.set_reveal_child(true);
}
