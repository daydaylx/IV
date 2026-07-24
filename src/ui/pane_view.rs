use std::cell::Cell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use gtk::prelude::*;

use crate::pane::{Orientation, PaneId, PaneNode, PaneTree};
use crate::tab::TabId;
use crate::terminal::Terminal;
use crate::ui::links;
use crate::ui::window::{self, UiState};

/// Recursively builds a GTK widget tree from a [`PaneNode`].
/// Populates `terminals` with new Terminal instances for each pane leaf.
///
/// The active pane (matching `tree.active_id()`) receives the `active-pane`
/// CSS class; panes whose terminals are already in `existing` reuse them.
pub(super) fn build_pane_widget(
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
                window::attach_pane_focus_handler(
                    &widget,
                    context.state.clone(),
                    context.tab_id,
                    *id,
                );
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
