use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;

use crate::pane::PaneId;
use crate::tab::{TabCollection, TabId};
use crate::terminal::Terminal;
use crate::ui::window;
use crate::ui::window::UiState;
use crate::ui::{pane_view, terminal_events}; // for focus_active_tab, terminal_active_pane_id

pub(crate) struct TabEntry {
    /// Terminal for each pane, keyed by PaneId.
    pub(crate) terminals: HashMap<PaneId, Terminal>,
    pub(crate) label: gtk::Label,
    pub(crate) page: gtk::Box,
}

/// Rebuilds one stable tab page and returns only newly created terminals.
pub(super) fn rebuild_tab_page(state: &Rc<UiState>, tab_id: TabId) -> Vec<Terminal> {
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
    let root = pane_view::build_pane_widget(
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
        terminal_events::connect_pane_events(Rc::downgrade(state), tab_id, *pane_id, terminal);
        terminal_events::connect_pane_title(Rc::downgrade(state), tab_id, *pane_id, terminal);
    }

    new_terminals
        .into_iter()
        .map(|(_, terminal)| terminal)
        .collect()
}

pub(super) fn detach_terminal_widgets(terminals: &HashMap<PaneId, Terminal>) {
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

pub(super) fn connect_notebook_signals(state: &Rc<UiState>) {
    let state_switch = Rc::clone(state);
    state
        .notebook
        .connect_switch_page(move |_notebook, _page, page_num| {
            crate::ui::search::close_search(&state_switch);
            state_switch
                .tab_collection
                .borrow_mut()
                .set_active(page_num as usize);
            window::focus_active_tab(&state_switch);
        });
}

pub(super) fn remove_tab(state: &Rc<UiState>, tab_id: TabId) {
    let index = state.tab_collection.borrow().find_index(tab_id);
    state.tab_collection.borrow_mut().remove(tab_id);
    state.tab_entries.borrow_mut().remove(&tab_id);
    if let Some(index) = index {
        state.notebook.remove_page(Some(index as u32));
    }
}

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
            if let Some(pane_id) = window::terminal_active_pane_id(state, tab_id) {
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

    window::focus_active_tab(state);

    for terminal in &new_terminals_to_start {
        terminal_events::start_terminal(state, terminal);
    }
    let _ = new_terminal_specs;
}
