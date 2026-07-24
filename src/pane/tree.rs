use std::collections::HashSet;

use super::navigation;
use super::node::{self, PaneNode};
use super::{Direction, Orientation, PaneId, SplitId};

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
    next_split_id: u64,
}

impl PaneTree {
    pub(crate) const MIN_SPLIT_RATIO: f64 = 0.05;
    pub(crate) const MAX_SPLIT_RATIO: f64 = 0.95;

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
            next_split_id: 0,
        }
    }

    /// Restores a tree from validated persisted nodes.
    ///
    /// Duplicate IDs, non-finite ratios and ratios outside the supported
    /// range are rejected. The first terminal becomes active because the
    /// persisted snapshot does not currently store pane focus.
    #[allow(dead_code, reason = "used by the not-yet-wired Phase-2 layout adapter")]
    pub(crate) fn from_root(root: PaneNode) -> Option<Self> {
        let mut pane_ids = HashSet::new();
        let mut split_ids = HashSet::new();
        if !node::validate_restored_node(&root, &mut pane_ids, &mut split_ids) {
            return None;
        }

        let active_id = node::find_first_leaf(&root);
        let next_id = pane_ids
            .iter()
            .map(|id| id.as_u64())
            .max()?
            .checked_add(1)?;
        let next_split_id = match split_ids.iter().map(|id| id.as_u64()).max() {
            Some(id) => id.checked_add(1)?,
            None => 0,
        };

        Some(Self {
            root,
            active_id,
            next_id,
            next_split_id,
        })
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
        node::contains(&self.root, id)
    }

    pub(crate) fn title(&self, id: PaneId) -> Option<&str> {
        node::title_in(&self.root, id)
    }

    /// Number of terminal leaves (actual shell panes).
    #[allow(dead_code)]
    pub(crate) fn leaf_count(&self) -> usize {
        node::count_leaves(&self.root)
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
        let split_id = self.allocate_split_id();
        let split = node::split_node(
            &mut self.root,
            self.active_id,
            split_id,
            orientation,
            new_id,
        );
        debug_assert!(split, "active pane must exist in the pane tree");
        self.active_id = new_id;
        new_id
    }

    /// Closes the active pane. If the active pane is the only pane in the tree,
    /// returns `None` (the caller should close the tab/window).
    /// Otherwise returns the new active PaneId.
    pub(crate) fn close_active(&mut self) -> Option<PaneId> {
        self.close(self.active_id)
    }

    /// Closes a pane by id. Closing an inactive pane preserves the current active pane.
    pub(crate) fn close(&mut self, id: PaneId) -> Option<PaneId> {
        if self.is_single() || !self.contains(id) {
            return None;
        }

        let previous_active = self.active_id;
        let (new_root, new_active) = node::close_node(
            std::mem::replace(
                &mut self.root,
                PaneNode::Terminal {
                    id: PaneId::new(u64::MAX),
                    title: String::new(),
                },
            ),
            id,
        );

        self.root = new_root;
        self.active_id = if id == previous_active {
            new_active
        } else {
            previous_active
        };
        Some(self.active_id)
    }

    /// Moves focus in the given direction. Returns true if focus changed.
    pub(crate) fn move_focus(&mut self, direction: Direction) -> bool {
        if let Some(new_id) = navigation::find_neighbor(&self.root, self.active_id, direction) {
            self.active_id = new_id;
            true
        } else {
            false
        }
    }

    /// Updates the title of a specific pane.
    pub(crate) fn set_title(&mut self, id: PaneId, title: String) {
        node::set_title_in(&mut self.root, id, title);
    }

    /// Returns an iterator over all terminal pane ids in the tree.
    pub(crate) fn pane_ids(&self) -> Vec<PaneId> {
        let mut ids = Vec::new();
        node::collect_ids(&self.root, &mut ids);
        ids
    }

    pub(crate) fn set_split_ratio(&mut self, id: SplitId, ratio: f64) -> bool {
        if !ratio.is_finite() {
            return false;
        }

        node::set_split_ratio_in(
            &mut self.root,
            id,
            ratio.clamp(Self::MIN_SPLIT_RATIO, Self::MAX_SPLIT_RATIO),
        )
    }

    // ------------------------------------------------------------------
    // Internal
    // ------------------------------------------------------------------

    fn allocate_id(&mut self) -> PaneId {
        let id = PaneId::new(self.next_id);
        self.next_id += 1;
        id
    }

    fn allocate_split_id(&mut self) -> SplitId {
        let id = SplitId::new(self.next_split_id);
        self.next_split_id += 1;
        id
    }
}
