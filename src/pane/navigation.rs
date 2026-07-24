use super::node::PaneNode;
use super::{Direction, Orientation, PaneId};

/// Find a neighboring pane in the given direction from `from_id`.
/// Returns `None` if no neighbor exists in that direction.
pub(crate) fn find_neighbor(
    root: &PaneNode,
    from_id: PaneId,
    direction: Direction,
) -> Option<PaneId> {
    find_neighbor_impl(root, from_id, direction).0
}

/// Returns (neighbor_id, found_target).
fn find_neighbor_impl(
    node: &PaneNode,
    from_id: PaneId,
    direction: Direction,
) -> (Option<PaneId>, bool) {
    match node {
        PaneNode::Terminal { id, .. } => {
            if *id == from_id {
                (None, true)
            } else {
                (None, false)
            }
        }
        PaneNode::Split {
            orientation,
            first,
            second,
            ..
        } => {
            let (result_first, found_in_first) = find_neighbor_impl(first, from_id, direction);
            if result_first.is_some() {
                return (result_first, true);
            }

            let (result_second, found_in_second) = find_neighbor_impl(second, from_id, direction);
            if result_second.is_some() {
                return (result_second, true);
            }

            // Not found deeper; check if we cross this split boundary.
            match (*orientation, direction) {
                (Orientation::Horizontal, Direction::Left) if found_in_second => {
                    (Some(rightmost_leaf(first)), true)
                }
                (Orientation::Horizontal, Direction::Right) if found_in_first => {
                    (Some(leftmost_leaf(second)), true)
                }
                (Orientation::Vertical, Direction::Up) if found_in_second => {
                    (Some(bottommost_leaf(first)), true)
                }
                (Orientation::Vertical, Direction::Down) if found_in_first => {
                    (Some(topmost_leaf(second)), true)
                }
                _ => (None, found_in_first || found_in_second),
            }
        }
    }
}

fn rightmost_leaf(node: &PaneNode) -> PaneId {
    match node {
        PaneNode::Terminal { id, .. } => *id,
        PaneNode::Split {
            orientation,
            second,
            ..
        } => match orientation {
            Orientation::Horizontal => rightmost_leaf(second),
            Orientation::Vertical => rightmost_leaf(second),
        },
    }
}

fn leftmost_leaf(node: &PaneNode) -> PaneId {
    match node {
        PaneNode::Terminal { id, .. } => *id,
        PaneNode::Split {
            orientation, first, ..
        } => match orientation {
            Orientation::Horizontal => leftmost_leaf(first),
            Orientation::Vertical => leftmost_leaf(first),
        },
    }
}

fn topmost_leaf(node: &PaneNode) -> PaneId {
    match node {
        PaneNode::Terminal { id, .. } => *id,
        PaneNode::Split {
            orientation, first, ..
        } => match orientation {
            Orientation::Vertical => topmost_leaf(first),
            Orientation::Horizontal => topmost_leaf(first),
        },
    }
}

fn bottommost_leaf(node: &PaneNode) -> PaneId {
    match node {
        PaneNode::Terminal { id, .. } => *id,
        PaneNode::Split {
            orientation,
            second,
            ..
        } => match orientation {
            Orientation::Vertical => bottommost_leaf(second),
            Orientation::Horizontal => bottommost_leaf(second),
        },
    }
}
