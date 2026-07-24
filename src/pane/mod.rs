#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PaneId(u64);

impl PaneId {
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub(crate) enum PaneNode {
    Terminal {
        id: PaneId,
        title: String,
    },
    Split {
        orientation: Orientation,
        /// Position of the divider, 0.0–1.0 (fraction of space given to first child).
        ratio: f64,
        first: Box<PaneNode>,
        second: Box<PaneNode>,
    },
}

/// A binary tree of split panes. Always contains at least one Terminal leaf.
///
/// Invariants:
/// - The tree always has at least one Terminal node.
/// - The `active_id` always points to an existing Terminal leaf.
/// - No empty Split nodes.
#[derive(Debug, Clone)]
pub(crate) struct PaneTree {
    root: PaneNode,
    active_id: PaneId,
    next_id: u64,
}

impl PaneTree {
    /// Creates a new tree with a single terminal pane.
    pub(crate) fn new() -> Self {
        let id = PaneId::new(0);
        Self {
            root: PaneNode::Terminal {
                id,
                title: String::from("IV"),
            },
            active_id: id,
            next_id: 1,
        }
    }

    // ------------------------------------------------------------------
    // Queries
    // ------------------------------------------------------------------

    pub(crate) fn active_id(&self) -> PaneId {
        self.active_id
    }

    pub(crate) fn set_active(&mut self, id: PaneId) -> bool {
        if self.contains(id) {
            self.active_id = id;
            true
        } else {
            false
        }
    }

    pub(crate) fn contains(&self, id: PaneId) -> bool {
        contains(&self.root, id)
    }

    /// Number of terminal leaves (actual shell panes).
    #[allow(dead_code)]
    pub(crate) fn leaf_count(&self) -> usize {
        count_leaves(&self.root)
    }

    /// True if the tree is a single pane with no splits.
    pub(crate) fn is_single(&self) -> bool {
        matches!(self.root, PaneNode::Terminal { .. })
    }

    /// Returns a reference to the root node for widget-building traversal.
    pub(crate) fn root(&self) -> &PaneNode {
        &self.root
    }

    // ------------------------------------------------------------------
    // Mutations
    // ------------------------------------------------------------------

    /// Splits the active pane in the given orientation. The existing pane becomes
    /// the first child; a new pane is created as the second child and becomes active.
    ///
    /// Returns the id of the newly created pane.
    pub(crate) fn split_active(&mut self, orientation: Orientation) -> PaneId {
        let new_id = self.allocate_id();
        split_node(&mut self.root, self.active_id, orientation, new_id);
        self.active_id = new_id;
        new_id
    }

    /// Closes the active pane. If the active pane is the only pane in the tree,
    /// returns `None` (the caller should close the tab/window).
    /// Otherwise returns the new active PaneId.
    pub(crate) fn close_active(&mut self) -> Option<PaneId> {
        if self.is_single() {
            return None;
        }

        let (new_root, new_active) = close_node(
            std::mem::replace(
                &mut self.root,
                PaneNode::Terminal {
                    id: PaneId::new(u64::MAX),
                    title: String::new(),
                },
            ),
            self.active_id,
        );

        self.root = new_root;
        self.active_id = new_active;
        Some(new_active)
    }

    /// Moves focus in the given direction. Returns true if focus changed.
    pub(crate) fn move_focus(&mut self, direction: Direction) -> bool {
        if let Some(new_id) = find_neighbor(&self.root, self.active_id, direction) {
            self.active_id = new_id;
            true
        } else {
            false
        }
    }

    /// Updates the title of a specific pane.
    pub(crate) fn set_title(&mut self, id: PaneId, title: String) {
        set_title_in(&mut self.root, id, title);
    }

    /// Returns an iterator over all terminal pane ids in the tree.
    pub(crate) fn pane_ids(&self) -> Vec<PaneId> {
        let mut ids = Vec::new();
        collect_ids(&self.root, &mut ids);
        ids
    }

    // ------------------------------------------------------------------
    // Internal
    // ------------------------------------------------------------------

    fn allocate_id(&mut self) -> PaneId {
        let id = PaneId::new(self.next_id);
        self.next_id += 1;
        id
    }
}

// --------------------------------------------------------------------------
// Recursive tree operations
// --------------------------------------------------------------------------

fn contains(node: &PaneNode, id: PaneId) -> bool {
    match node {
        PaneNode::Terminal { id: node_id, .. } => *node_id == id,
        PaneNode::Split { first, second, .. } => contains(first, id) || contains(second, id),
    }
}

#[allow(dead_code)]
fn count_leaves(node: &PaneNode) -> usize {
    match node {
        PaneNode::Terminal { .. } => 1,
        PaneNode::Split { first, second, .. } => count_leaves(first) + count_leaves(second),
    }
}

/// Splits the pane with `target_id` in-place. Does nothing if not found.
fn split_node(node: &mut PaneNode, target_id: PaneId, orientation: Orientation, new_id: PaneId) {
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
            orientation,
            ratio: 0.5,
            first: Box::new(old),
            second: Box::new(PaneNode::Terminal {
                id: new_id,
                title: String::from("IV"),
            }),
        };
    } else if let PaneNode::Split { first, second, .. } = node {
        if contains(first, target_id) {
            split_node(first, target_id, orientation, new_id);
        } else if contains(second, target_id) {
            split_node(second, target_id, orientation, new_id);
        }
    }
}

/// Closes the pane with `target_id` and returns the new subtree and the new active pane id.
/// If the pane is in a split, the sibling is pulled up and becomes active.
fn close_node(node: PaneNode, target_id: PaneId) -> (PaneNode, PaneId) {
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

fn find_first_leaf(node: &PaneNode) -> PaneId {
    match node {
        PaneNode::Terminal { id, .. } => *id,
        PaneNode::Split { first, .. } => find_first_leaf(first),
    }
}

fn set_title_in(node: &mut PaneNode, id: PaneId, title: String) {
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

fn collect_ids(node: &PaneNode, out: &mut Vec<PaneId>) {
    match node {
        PaneNode::Terminal { id, .. } => out.push(*id),
        PaneNode::Split { first, second, .. } => {
            collect_ids(first, out);
            collect_ids(second, out);
        }
    }
}

// --------------------------------------------------------------------------
// Direction-based focus navigation
// --------------------------------------------------------------------------

impl PaneNode {
    fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal { .. })
    }
}

/// Find a neighboring pane in the given direction from `from_id`.
/// Returns `None` if no neighbor exists in that direction.
fn find_neighbor(root: &PaneNode, from_id: PaneId, direction: Direction) -> Option<PaneId> {
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

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Creation ---------------------------------------------------------

    #[test]
    fn new_tree_is_single() {
        let tree = PaneTree::new();
        assert!(tree.is_single());
        assert_eq!(tree.leaf_count(), 1);
    }

    #[test]
    fn new_tree_has_valid_active() {
        let tree = PaneTree::new();
        assert!(tree.contains(tree.active_id()));
    }

    // -- Split ------------------------------------------------------------

    #[test]
    fn split_increases_leaf_count() {
        let mut tree = PaneTree::new();
        let initial = tree.leaf_count();

        tree.split_active(Orientation::Horizontal);
        assert_eq!(tree.leaf_count(), initial + 1);
        assert!(!tree.is_single());

        tree.split_active(Orientation::Vertical);
        assert_eq!(tree.leaf_count(), initial + 2);
    }

    #[test]
    fn split_activates_new_pane() {
        let mut tree = PaneTree::new();
        let old_active = tree.active_id();
        let new_id = tree.split_active(Orientation::Horizontal);

        assert_ne!(new_id, old_active);
        assert_eq!(tree.active_id(), new_id);
    }

    #[test]
    fn split_preserves_old_pane() {
        let mut tree = PaneTree::new();
        let old_id = tree.active_id();
        tree.split_active(Orientation::Horizontal);

        assert!(tree.contains(old_id));
    }

    // -- Close ------------------------------------------------------------

    #[test]
    fn close_single_pane_returns_none() {
        let mut tree = PaneTree::new();
        assert_eq!(tree.close_active(), None);
    }

    #[test]
    fn close_in_split_reduces_count() {
        let mut tree = PaneTree::new();
        let first_id = tree.active_id();
        tree.split_active(Orientation::Horizontal);
        let count_before = tree.leaf_count();

        // Active is the new (second) pane. Close it.
        let result = tree.close_active();
        assert!(result.is_some());
        assert_eq!(tree.leaf_count(), count_before - 1);
        // The first pane should now be the only one.
        assert!(tree.is_single());
        assert_eq!(tree.active_id(), first_id);
    }

    #[test]
    fn close_first_child_pulls_up_second() {
        let mut tree = PaneTree::new();
        let first_id = tree.active_id();
        let new_id = tree.split_active(Orientation::Horizontal);

        // Now active is new_id (second child). Switch back to first.
        tree.set_active(first_id);
        assert_eq!(tree.active_id(), first_id);

        // Close first → second should be pulled up.
        let result = tree.close_active();
        assert_eq!(result, Some(new_id));
        assert!(tree.is_single());
    }

    #[test]
    fn close_in_nested_split() {
        let mut tree = PaneTree::new();
        tree.split_active(Orientation::Horizontal); // root HSplit: [P0, P1(active)]
        let p1 = tree.active_id();
        tree.split_active(Orientation::Vertical); // root HSplit: [P0, VSplit: [P1, P2(active)]]
        let _p2 = tree.active_id();

        assert_eq!(tree.leaf_count(), 3);

        // Close P2 → VSplit collapses into P1.
        let result = tree.close_active();
        assert_eq!(result, Some(p1));
        assert_eq!(tree.leaf_count(), 2);
        assert!(!tree.is_single());
    }

    // -- Focus navigation -------------------------------------------------

    #[test]
    fn move_left_right_in_horizontal_split() {
        let mut tree = PaneTree::new();
        let left_id = tree.active_id();
        let right_id = tree.split_active(Orientation::Horizontal);

        // Active is right. Move left.
        assert!(tree.move_focus(Direction::Left));
        assert_eq!(tree.active_id(), left_id);

        // Move right.
        assert!(tree.move_focus(Direction::Right));
        assert_eq!(tree.active_id(), right_id);

        // Can't go further right.
        assert!(!tree.move_focus(Direction::Right));
        assert_eq!(tree.active_id(), right_id);
    }

    #[test]
    fn move_up_down_in_vertical_split() {
        let mut tree = PaneTree::new();
        let top_id = tree.active_id();
        let bottom_id = tree.split_active(Orientation::Vertical);

        // Active is bottom. Move up.
        assert!(tree.move_focus(Direction::Up));
        assert_eq!(tree.active_id(), top_id);

        // Move down.
        assert!(tree.move_focus(Direction::Down));
        assert_eq!(tree.active_id(), bottom_id);

        // Can't go further down.
        assert!(!tree.move_focus(Direction::Down));
        assert_eq!(tree.active_id(), bottom_id);
    }

    #[test]
    fn move_focus_no_op_when_no_neighbor() {
        let mut tree = PaneTree::new();
        let id = tree.active_id();

        assert!(!tree.move_focus(Direction::Left));
        assert!(!tree.move_focus(Direction::Right));
        assert!(!tree.move_focus(Direction::Up));
        assert!(!tree.move_focus(Direction::Down));
        assert_eq!(tree.active_id(), id);
    }

    #[test]
    fn move_across_nested_splits() {
        // Build: VSplit(first=HSplit(P0, P2), second=P1)
        // Layout:
        // +-------+-------+
        // |  P0   |  P2   |
        // +-------+-------+
        // |      P1        |
        // +----------------+
        let mut tree = PaneTree::new();
        let p0 = tree.active_id();
        tree.split_active(Orientation::Vertical); // P0 above P1; active=P1
        let p1 = tree.active_id();
        tree.set_active(p0);
        tree.split_active(Orientation::Horizontal); // P0 splits to P0|P2; active=P2
        let p2 = tree.active_id();

        // P2 going left should reach P0 (its sibling in the horizontal split).
        assert!(tree.move_focus(Direction::Left));
        assert_eq!(tree.active_id(), p0);

        // P0 going right → P2.
        assert!(tree.move_focus(Direction::Right));
        assert_eq!(tree.active_id(), p2);

        // P2 going down → P1 (across the vertical split boundary).
        assert!(tree.move_focus(Direction::Down));
        assert_eq!(tree.active_id(), p1);

        // P1 going up → back to the bottommost of the top row.
        // The top row is HSplit(P0, P2); bottommost of that is P2 (right side).
        assert!(tree.move_focus(Direction::Up));
        assert_eq!(tree.active_id(), p2);
    }

    // -- set_title --------------------------------------------------------

    #[test]
    fn set_title_updates_terminal() {
        let mut tree = PaneTree::new();
        let id = tree.active_id();
        tree.set_title(id, String::from("zsh"));

        // Verify via the tree structure
        if let PaneNode::Terminal { title, .. } = &tree.root {
            assert_eq!(title, "zsh");
        } else {
            panic!("Expected terminal");
        }
    }

    #[test]
    fn set_title_non_existent_is_noop() {
        let mut tree = PaneTree::new();
        let original = tree.active_id();
        tree.set_title(PaneId::new(999), String::from("nope"));
        assert_eq!(tree.active_id(), original);
    }

    // -- IDs --------------------------------------------------------------

    #[test]
    fn ids_are_unique() {
        let mut tree = PaneTree::new();
        let id0 = tree.active_id();
        let id1 = tree.split_active(Orientation::Horizontal);
        let id2 = tree.split_active(Orientation::Vertical);

        assert_ne!(id0, id1);
        assert_ne!(id1, id2);
        assert_ne!(id0, id2);
    }

    #[test]
    fn pane_ids_returns_all() {
        let mut tree = PaneTree::new();
        let id0 = tree.active_id();
        let id1 = tree.split_active(Orientation::Horizontal);
        let id2 = tree.split_active(Orientation::Vertical);

        let ids = tree.pane_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&id0));
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn set_active_invalid_is_noop() {
        let mut tree = PaneTree::new();
        let original = tree.active_id();
        assert!(!tree.set_active(PaneId::new(999)));
        assert_eq!(tree.active_id(), original);
    }

    #[test]
    fn set_active_valid_works() {
        let mut tree = PaneTree::new();
        let _new_id = tree.split_active(Orientation::Horizontal);
        let original_active = tree.active_id();
        let other_id = tree
            .pane_ids()
            .into_iter()
            .find(|id| *id != original_active)
            .unwrap();

        assert!(tree.set_active(other_id));
        assert_eq!(tree.active_id(), other_id);
    }

    // -- After-close invariants -------------------------------------------

    #[test]
    fn active_exists_after_close() {
        let mut tree = PaneTree::new();
        tree.split_active(Orientation::Horizontal);
        let result = tree.close_active();
        assert!(result.is_some());
        assert!(tree.contains(tree.active_id()));
    }

    #[test]
    fn no_duplicate_ids_after_operations() {
        let mut tree = PaneTree::new();
        tree.split_active(Orientation::Horizontal);
        tree.split_active(Orientation::Vertical);
        tree.set_active(tree.pane_ids()[0]);

        let ids = tree.pane_ids();
        let mut sorted = ids.clone();
        sorted.sort_by_key(|id| id.0);
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "duplicate ids found");
    }
}
