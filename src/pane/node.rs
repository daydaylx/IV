use std::collections::HashSet;

use super::{Orientation, PaneId, SplitId, tree::PaneTree};

#[derive(Debug, Clone)]
pub(crate) enum PaneNode {
    Terminal {
        id: PaneId,
        title: String,
    },
    Split {
        id: SplitId,
        orientation: Orientation,
        /// Position of the divider, 0.0–1.0 (fraction of space given to first child).
        ratio: f64,
        first: Box<PaneNode>,
        second: Box<PaneNode>,
    },
}

impl PaneNode {
    pub(crate) fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal { .. })
    }
}

pub(crate) fn contains(node: &PaneNode, id: PaneId) -> bool {
    match node {
        PaneNode::Terminal { id: node_id, .. } => *node_id == id,
        PaneNode::Split { first, second, .. } => contains(first, id) || contains(second, id),
    }
}

#[allow(dead_code)]
pub(crate) fn count_leaves(node: &PaneNode) -> usize {
    match node {
        PaneNode::Terminal { .. } => 1,
        PaneNode::Split { first, second, .. } => count_leaves(first) + count_leaves(second),
    }
}

/// Splits the pane with `target_id` in-place. Does nothing if not found.
pub(crate) fn split_node(
    node: &mut PaneNode,
    target_id: PaneId,
    split_id: SplitId,
    orientation: Orientation,
    new_id: PaneId,
) -> bool {
    if let PaneNode::Terminal { id, .. } = node
        && *id == target_id
    {
        // Replace this terminal with a Split.
        let old = std::mem::replace(
            node,
            PaneNode::Terminal {
                id: PaneId::new(u64::MAX),
                title: String::new(),
            },
        );
        *node = PaneNode::Split {
            id: split_id,
            orientation,
            ratio: 0.5,
            first: Box::new(old),
            second: Box::new(PaneNode::Terminal {
                id: new_id,
                title: String::from("IV"),
            }),
        };
        true
    } else if let PaneNode::Split { first, second, .. } = node {
        if contains(first, target_id) {
            split_node(first, target_id, split_id, orientation, new_id)
        } else if contains(second, target_id) {
            split_node(second, target_id, split_id, orientation, new_id)
        } else {
            false
        }
    } else {
        false
    }
}

/// Closes the pane with `target_id` and returns the new subtree and the new active pane id.
/// If the pane is in a split, the sibling is pulled up and becomes active.
pub(crate) fn close_node(node: PaneNode, target_id: PaneId) -> (PaneNode, PaneId) {
    match node {
        PaneNode::Terminal { id, .. } if id == target_id => {
            // Should never be called on the root – caller checks is_single().
            // Return a dummy to satisfy the compiler.
            (
                PaneNode::Terminal {
                    id: PaneId::new(u64::MAX),
                    title: String::new(),
                },
                PaneId::new(u64::MAX),
            )
        }
        term @ PaneNode::Terminal { .. } => (term, PaneId::new(u64::MAX)),
        PaneNode::Split {
            id,
            orientation,
            ratio,
            first,
            second,
        } => {
            let first_contains = contains(&first, target_id);
            let second_contains = contains(&second, target_id);

            if first_contains && first.is_terminal() {
                // target is the immediate first child → pull up second.
                let active = find_first_leaf(&second);
                (*second, active)
            } else if second_contains && second.is_terminal() {
                // target is the immediate second child → pull up first.
                let active = find_first_leaf(&first);
                (*first, active)
            } else if first_contains {
                let (new_first, active) = close_node(*first, target_id);
                (
                    PaneNode::Split {
                        id,
                        orientation,
                        ratio,
                        first: Box::new(new_first),
                        second,
                    },
                    active,
                )
            } else if second_contains {
                let (new_second, active) = close_node(*second, target_id);
                (
                    PaneNode::Split {
                        id,
                        orientation,
                        ratio,
                        first,
                        second: Box::new(new_second),
                    },
                    active,
                )
            } else {
                (
                    PaneNode::Split {
                        id,
                        orientation,
                        ratio,
                        first,
                        second,
                    },
                    PaneId::new(u64::MAX),
                )
            }
        }
    }
}

pub(crate) fn find_first_leaf(node: &PaneNode) -> PaneId {
    match node {
        PaneNode::Terminal { id, .. } => *id,
        PaneNode::Split { first, .. } => find_first_leaf(first),
    }
}

pub(crate) fn set_title_in(node: &mut PaneNode, id: PaneId, title: String) {
    match node {
        PaneNode::Terminal {
            id: node_id,
            title: t,
            ..
        } if *node_id == id => {
            *t = title;
        }
        PaneNode::Terminal { .. } => {}
        PaneNode::Split { first, second, .. } => {
            set_title_in(first, id, title.clone());
            set_title_in(second, id, title);
        }
    }
}

pub(crate) fn title_in(node: &PaneNode, id: PaneId) -> Option<&str> {
    match node {
        PaneNode::Terminal { id: node_id, title } if *node_id == id => Some(title),
        PaneNode::Terminal { .. } => None,
        PaneNode::Split { first, second, .. } => {
            title_in(first, id).or_else(|| title_in(second, id))
        }
    }
}

pub(crate) fn collect_ids(node: &PaneNode, out: &mut Vec<PaneId>) {
    match node {
        PaneNode::Terminal { id, .. } => out.push(*id),
        PaneNode::Split { first, second, .. } => {
            collect_ids(first, out);
            collect_ids(second, out);
        }
    }
}

#[allow(dead_code, reason = "used by PaneTree::from_root for Phase-2 restore")]
pub(crate) fn validate_restored_node(
    node: &PaneNode,
    pane_ids: &mut HashSet<PaneId>,
    split_ids: &mut HashSet<SplitId>,
) -> bool {
    match node {
        PaneNode::Terminal { id, .. } => pane_ids.insert(*id),
        PaneNode::Split {
            id,
            ratio,
            first,
            second,
            ..
        } => {
            split_ids.insert(*id)
                && ratio.is_finite()
                && (PaneTree::MIN_SPLIT_RATIO..=PaneTree::MAX_SPLIT_RATIO).contains(ratio)
                && validate_restored_node(first, pane_ids, split_ids)
                && validate_restored_node(second, pane_ids, split_ids)
        }
    }
}

pub(crate) fn set_split_ratio_in(node: &mut PaneNode, id: SplitId, ratio: f64) -> bool {
    match node {
        PaneNode::Terminal { .. } => false,
        PaneNode::Split {
            id: node_id,
            ratio: node_ratio,
            ..
        } if *node_id == id => {
            *node_ratio = ratio;
            true
        }
        PaneNode::Split { first, second, .. } => {
            set_split_ratio_in(first, id, ratio) || set_split_ratio_in(second, id, ratio)
        }
    }
}
