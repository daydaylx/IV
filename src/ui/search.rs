//! Search-bar lifecycle for the terminal that was active when search opened.

use std::rc::Rc;

use adw::prelude::*;
use gtk::gio;

use super::window::{UiState, focus_active_tab, show_status, terminal_for};

pub(super) fn install_search_actions(application: &adw::Application, state: &Rc<UiState>) {
    let state_search = Rc::clone(state);
    let toggle_action = gio::SimpleAction::new("toggle-search", None);
    toggle_action.connect_activate(move |_, _| toggle_search(&state_search));
    state.window.add_action(&toggle_action);
    application.set_accels_for_action("win.toggle-search", &["<Control><Shift>f"]);

    let state_enter = Rc::clone(state);
    state.search_entry.connect_activate(move |_| {
        if let Some(terminal) = search_terminal(&state_enter) {
            terminal.search_next();
        }
    });

    let state_changed = Rc::clone(state);
    state.search_entry.connect_search_changed(move |entry| {
        let query = entry.text();
        if let Some(terminal) = search_terminal(&state_changed)
            && let Err(error) = terminal.search(query.as_ref())
        {
            show_status(&state_changed, error.user_message());
        }
    });

    let state_prev = Rc::clone(state);
    state.search_entry.connect_previous_match(move |_| {
        if let Some(terminal) = search_terminal(&state_prev) {
            terminal.search_previous();
        }
    });

    let state_stop = Rc::clone(state);
    state
        .search_entry
        .connect_stop_search(move |_| close_search(&state_stop));

    let state_hide = Rc::clone(state);
    state
        .search_bar
        .connect_notify_local(Some("search-mode-enabled"), move |bar, _| {
            if !bar.is_search_mode() {
                clear_search_target(&state_hide);
                focus_active_tab(&state_hide);
            }
        });
}

fn toggle_search(state: &Rc<UiState>) {
    if state.search_bar.is_search_mode() {
        close_search(state);
        return;
    }

    let tabs = state.tab_collection.borrow();
    *state.search_target.borrow_mut() = Some((tabs.active_id(), tabs.active_pane_id()));
    drop(tabs);

    state.search_bar.set_search_mode(true);
    state.search_entry.grab_focus();
    if let Some(terminal) = search_terminal(state) {
        let query = state.search_entry.text();
        if let Err(error) = terminal.search(query.as_ref()) {
            show_status(state, error.user_message());
        }
    }
}

pub(super) fn close_search(state: &Rc<UiState>) {
    clear_search_target(state);
    if state.search_bar.is_search_mode() {
        state.search_bar.set_search_mode(false);
    }
}

fn search_terminal(state: &Rc<UiState>) -> Option<crate::terminal::Terminal> {
    let (tab_id, pane_id) = (*state.search_target.borrow())?;
    terminal_for(state, tab_id, pane_id)
}

fn clear_search_target(state: &Rc<UiState>) {
    if let Some((tab_id, pane_id)) = state.search_target.borrow_mut().take()
        && let Some(terminal) = terminal_for(state, tab_id, pane_id)
    {
        terminal.search_clear();
    }
}
