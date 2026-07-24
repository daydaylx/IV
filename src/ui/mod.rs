use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

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
    terminal: Terminal,
    label: gtk::Label,
}

// ---------------------------------------------------------------------------
// Shared UI state – all callbacks hold an Rc<UiState>
// ---------------------------------------------------------------------------

struct UiState {
    tab_collection: RefCell<TabCollection>,
    terminals: RefCell<HashMap<TabId, TabEntry>>,
    notebook: gtk::Notebook,
    status_label: gtk::Label,
    status_revealer: gtk::Revealer,
    window: adw::ApplicationWindow,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) fn build_main_window(application: &adw::Application) {
    // -- domain model
    let tab_collection = TabCollection::new();

    // -- first terminal
    let first_terminal = Terminal::new();
    let first_id = tab_collection.tabs()[0].id;

    // -- notebook (tab container)
    let notebook = gtk::Notebook::new();
    notebook.set_scrollable(true);
    notebook.set_show_border(false);

    let first_label = gtk::Label::new(Some("IV"));
    notebook.append_page(&first_terminal.widget(), Some(&first_label));

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
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&adw::HeaderBar::new());
    content.append(&notebook);
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

    // -- shared state
    let mut terminals = HashMap::new();
    terminals.insert(
        first_id,
        TabEntry {
            terminal: first_terminal.clone(),
            label: first_label,
        },
    );

    let state = Rc::new(UiState {
        tab_collection: RefCell::new(tab_collection),
        terminals: RefCell::new(terminals),
        notebook,
        status_label,
        status_revealer,
        window: window.clone(),
    });

    // -- wire everything up
    setup_first_tab(&state, first_id);
    install_tab_actions(application, &state);
    install_clipboard_actions(application, &state);
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
// First tab wiring
// ---------------------------------------------------------------------------

fn setup_first_tab(state: &Rc<UiState>, tab_id: TabId) {
    let terminal = state
        .terminals
        .borrow()
        .get(&tab_id)
        .unwrap()
        .terminal
        .clone();
    connect_tab_terminal_events(state, tab_id, &terminal);
    connect_tab_title_changed(state, tab_id, &terminal);
}

// ---------------------------------------------------------------------------
// Connecting terminal events for a single tab
// ---------------------------------------------------------------------------

fn connect_tab_terminal_events(state: &Rc<UiState>, tab_id: TabId, terminal: &Terminal) {
    let state = Rc::clone(state);

    terminal.set_event_handler(Rc::new(move |event| match event {
        TerminalEvent::Started => {}
        TerminalEvent::SpawnFailed(error) => {
            show_status(&state, error.user_message());
        }
        TerminalEvent::Exited(exit) if exit.successful() => {
            handle_tab_exited_success(&state, tab_id);
        }
        TerminalEvent::Exited(exit) => {
            handle_tab_exited_error(&state, tab_id, &exit.user_message());
        }
    }));
}

fn connect_tab_title_changed(state: &Rc<UiState>, tab_id: TabId, terminal: &Terminal) {
    let state = Rc::clone(state);

    terminal.connect_title_changed(move |title| {
        let mut tabs = state.tab_collection.borrow_mut();
        tabs.set_title(tab_id, title.to_owned());

        if let Some(entry) = state.terminals.borrow().get(&tab_id) {
            entry.label.set_label(title);
        }
    });
}

// ---------------------------------------------------------------------------
// Terminal event handlers
// ---------------------------------------------------------------------------

fn handle_tab_exited_success(state: &Rc<UiState>, tab_id: TabId) {
    // Determine if this was the *last* tab before we remove it.
    let was_last = state.tab_collection.borrow().len() <= 1;

    if was_last {
        state.window.close();
    } else {
        remove_tab(state, tab_id);
    }
}

fn handle_tab_exited_error(state: &Rc<UiState>, tab_id: TabId, message: &str) {
    // Show the error only if this tab is currently active.
    if state.tab_collection.borrow().active_id() == tab_id {
        show_status(state, message);
    }

    // Update the tab label to reflect the dead shell.
    if let Some(entry) = state.terminals.borrow().get(&tab_id) {
        entry
            .label
            .set_label(&format!("[beendet] {}", entry.label.label()));
    }
}

// ---------------------------------------------------------------------------
// Tab manipulation
// ---------------------------------------------------------------------------

fn create_new_tab(state: &Rc<UiState>) {
    let terminal = Terminal::new();
    let label = gtk::Label::new(Some("IV"));

    let (tab_id, new_index) = {
        let mut tabs = state.tab_collection.borrow_mut();
        tabs.add()
    };

    state.terminals.borrow_mut().insert(
        tab_id,
        TabEntry {
            terminal: terminal.clone(),
            label: label.clone(),
        },
    );
    state
        .notebook
        .insert_page(&terminal.widget(), Some(&label), Some(new_index as u32));

    // Switch to the new tab
    state.notebook.set_current_page(Some(new_index as u32));

    // Wire events
    connect_tab_terminal_events(state, tab_id, &terminal);
    connect_tab_title_changed(state, tab_id, &terminal);

    // Start shell
    match terminal.start() {
        Ok(warnings) => show_warnings(state, &warnings),
        Err(error) => show_status(state, error.user_message()),
    }

    glib::idle_add_local_once(move || terminal.focus());
}

fn close_active_tab(state: &Rc<UiState>) {
    let active_id = state.tab_collection.borrow().active_id();

    // If this is the last tab, close the window instead.
    if state.tab_collection.borrow().len() <= 1 {
        state.window.close();
        return;
    }

    let terminal = state
        .terminals
        .borrow()
        .get(&active_id)
        .unwrap()
        .terminal
        .clone();

    let state_weak = Rc::downgrade(state);
    let close_immediate = terminal.request_close(move || {
        if let Some(state) = state_weak.upgrade() {
            remove_tab(&state, active_id);
        }
    });

    if !close_immediate {
        show_status(state, "Die Shell wird beendet …");
    }
    // If close_immediate (already exited), the callback already fired synchronously.
}

/// Removes a tab from all data structures and the notebook.
fn remove_tab(state: &Rc<UiState>, tab_id: TabId) {
    if let Some(index) = state.tab_collection.borrow().find_index(tab_id) {
        state.notebook.remove_page(Some(index as u32));
    }
    state.tab_collection.borrow_mut().remove(tab_id);
    state.terminals.borrow_mut().remove(&tab_id);
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

fn focus_active_tab(state: &Rc<UiState>) {
    let active_id = state.tab_collection.borrow().active_id();
    if let Some(entry) = state.terminals.borrow().get(&active_id) {
        let terminal = entry.terminal.clone();
        glib::idle_add_local_once(move || terminal.focus());
    }
}

// ---------------------------------------------------------------------------
// Clipboard actions (copy / paste on the active tab)
// ---------------------------------------------------------------------------

fn install_clipboard_actions(application: &adw::Application, state: &Rc<UiState>) {
    let state_copy = Rc::clone(state);
    let copy_action = gio::SimpleAction::new("copy", None);
    copy_action.connect_activate(move |_, _| {
        let active_id = state_copy.tab_collection.borrow().active_id();
        if let Some(entry) = state_copy.terminals.borrow().get(&active_id)
            && let Err(error) = entry.terminal.copy()
        {
            show_status(&state_copy, error.user_message());
        }
    });
    state.window.add_action(&copy_action);

    let state_paste = Rc::clone(state);
    let paste_action = gio::SimpleAction::new("paste", None);
    paste_action.connect_activate(move |_, _| {
        let active_id = state_paste.tab_collection.borrow().active_id();
        if let Some(entry) = state_paste.terminals.borrow().get(&active_id)
            && let Err(error) = entry.terminal.paste()
        {
            show_status(&state_paste, error.user_message());
        }
    });
    state.window.add_action(&paste_action);

    application.set_accels_for_action("win.copy", &["<Control><Shift>c"]);
    application.set_accels_for_action("win.paste", &["<Control><Shift>v"]);
}

// ---------------------------------------------------------------------------
// Notebook signals: sync active tab on switch, focus newly visible terminal
// ---------------------------------------------------------------------------

fn connect_notebook_signals(state: &Rc<UiState>) {
    let state_switch = Rc::clone(state);
    state
        .notebook
        .connect_switch_page(move |_notebook, _page, page_num| {
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
        // Collect owned (TabId, Terminal) pairs so they live long enough
        // for the 'static callbacks.
        let terminal_pairs: Vec<(TabId, Terminal)> = state
            .terminals
            .borrow()
            .iter()
            .map(|(id, entry)| (*id, entry.terminal.clone()))
            .collect();

        let pending = Rc::new(RefCell::new(0u32));
        let mut any_non_immediate = false;
        let window_weak = window.downgrade();

        for (tab_id, terminal) in terminal_pairs {
            let pending_for_callback = Rc::clone(&pending);
            let window_weak = window_weak.clone();
            let state = Rc::clone(&state);

            let immediate = terminal.request_close(move || {
                state.tab_collection.borrow_mut().remove(tab_id);
                state.terminals.borrow_mut().remove(&tab_id);

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
