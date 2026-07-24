use std::rc::{Rc, Weak};

use gtk::prelude::*;

use crate::pane::PaneId;
use crate::tab::TabId;
use crate::terminal::{StartupWarning, Terminal, TerminalEvent};
use crate::ui::tabs_view;
use crate::ui::window::{self, UiState};

pub(super) fn connect_pane_events(
    state: Weak<UiState>,
    tab_id: TabId,
    pane_id: PaneId,
    terminal: &Terminal,
) {
    terminal.set_event_handler(Rc::new(move |event| match event {
        TerminalEvent::Started => {}
        TerminalEvent::SpawnFailed(error) => {
            if let Some(state) = state.upgrade() {
                window::show_status(&state, error.user_message());
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

pub(super) fn connect_pane_title(
    state: Weak<UiState>,
    tab_id: TabId,
    pane_id: PaneId,
    terminal: &Terminal,
) {
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
            tabs_view::remove_tab(state, tab_id);
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
            tabs_view::rebuild_tab_page(state, tab_id);
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
        window::show_status(state, message);
    }

    // Update the tab label if this was the active pane.
    if is_active_pane && let Some(entry) = state.tab_entries.borrow().get(&tab_id) {
        entry
            .label
            .set_label(&format!("[beendet] {}", entry.label.label()));
    }
}

pub(crate) fn start_terminal(state: &Rc<UiState>, terminal: &Terminal) {
    match terminal.start() {
        Ok(warnings) => show_warnings(state, &warnings),
        Err(error) => window::show_status(state, error.user_message()),
    }
}

pub(super) fn show_warnings(state: &Rc<UiState>, warnings: &[StartupWarning]) {
    if warnings.is_empty() {
        return;
    }

    let message = warnings
        .iter()
        .map(|warning| warning.user_message())
        .collect::<Vec<_>>()
        .join(" ");
    window::show_status(state, &message);
}
