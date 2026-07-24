use crate::pane::{Direction as PaneDirection, Orientation as PaneOrientation, PaneId, PaneTree};
use crate::workspace::StartConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TabId(u64);

impl TabId {
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }

    #[allow(dead_code, reason = "used by the not-yet-wired Phase-2 layout adapter")]
    pub(crate) fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TabInfo {
    pub(crate) id: TabId,
    pub(crate) title: String,
    pub(crate) pane_tree: PaneTree,
    #[allow(dead_code, reason = "reserved for the Phase-2 layout adapter")]
    pub(crate) custom_title: Option<String>,
    #[allow(dead_code, reason = "reserved for the Phase-2 layout adapter")]
    pub(crate) start_config: Option<StartConfig>,
}

impl TabInfo {
    pub(crate) fn new(id: TabId, title: String, pane_tree: PaneTree) -> Self {
        Self {
            id,
            title,
            pane_tree,
            custom_title: None,
            start_config: None,
        }
    }
}

/// Manages the ordered list of tabs and which one is active.
///
/// Does not hold GTK or VTE types. The UI layer maps [`TabId`] to terminal widgets.
pub(crate) struct TabCollection {
    tabs: Vec<TabInfo>,
    active_index: usize,
    next_id: u64,
}

impl TabCollection {
    /// Creates a new collection with a single initial tab containing one pane.
    pub(crate) fn new() -> Self {
        Self {
            tabs: vec![TabInfo::new(
                TabId::new(0),
                String::from("IV"),
                PaneTree::new(),
            )],
            active_index: 0,
            next_id: 1,
        }
    }

    #[allow(dead_code, reason = "used by the not-yet-wired Phase-2 layout adapter")]
    pub(crate) fn from_tabs(tabs: Vec<TabInfo>, active_index: usize) -> Option<Self> {
        if tabs.is_empty() {
            return Some(Self::new());
        }

        let next_id = tabs
            .iter()
            .map(|tab| tab.id.as_u64())
            .max()
            .and_then(|id| id.checked_add(1))?;
        let active_index = active_index.min(tabs.len() - 1);
        Some(Self {
            tabs,
            active_index,
            next_id,
        })
    }

    /// Adds a new tab at the end and activates it. Returns its id and index.
    pub(crate) fn add(&mut self) -> (TabId, usize) {
        let id = TabId::new(self.next_id);
        self.next_id += 1;
        let index = self.tabs.len();
        self.tabs
            .push(TabInfo::new(id, String::from("IV"), PaneTree::new()));
        self.active_index = index;
        (id, index)
    }

    /// Removes a tab by id. Returns it if found.
    ///
    /// After removal the active index is adjusted. If the collection becomes empty,
    /// the caller should close the window.
    pub(crate) fn remove(&mut self, id: TabId) -> Option<TabInfo> {
        let index = self.tabs.iter().position(|t| t.id == id)?;
        let tab = self.tabs.remove(index);

        if !self.tabs.is_empty() {
            if index < self.active_index {
                self.active_index -= 1;
            } else if self.active_index >= self.tabs.len() {
                self.active_index = self.tabs.len() - 1;
            }
        }

        Some(tab)
    }

    pub(crate) fn active_index(&self) -> usize {
        self.active_index
    }

    pub(crate) fn active_id(&self) -> TabId {
        self.tabs[self.active_index].id
    }

    #[allow(dead_code)]
    pub(crate) fn active_info(&self) -> &TabInfo {
        &self.tabs[self.active_index]
    }

    pub(crate) fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }

    pub(crate) fn next(&mut self) {
        if self.tabs.len() > 1 {
            self.active_index = (self.active_index + 1) % self.tabs.len();
        }
    }

    pub(crate) fn prev(&mut self) {
        if self.tabs.len() > 1 {
            if self.active_index == 0 {
                self.active_index = self.tabs.len() - 1;
            } else {
                self.active_index -= 1;
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.tabs.len()
    }

    pub(crate) fn tabs(&self) -> &[TabInfo] {
        &self.tabs
    }

    pub(crate) fn find_index(&self, id: TabId) -> Option<usize> {
        self.tabs.iter().position(|t| t.id == id)
    }

    #[allow(dead_code)]
    pub(crate) fn set_title(&mut self, id: TabId, title: String) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.title = title;
        }
    }

    // ------------------------------------------------------------------
    // Pane delegation (active tab)
    // ------------------------------------------------------------------

    pub(crate) fn active_pane_id(&self) -> PaneId {
        self.tabs[self.active_index].pane_tree.active_id()
    }

    /// Splits the active pane of the active tab. Returns the new PaneId.
    pub(crate) fn split_active(&mut self, orientation: PaneOrientation) -> PaneId {
        self.tabs[self.active_index]
            .pane_tree
            .split_active(orientation)
    }

    /// Closes the active pane of the active tab.
    /// Returns the new active PaneId, or None if it was the last pane.
    pub(crate) fn close_active_pane(&mut self) -> Option<PaneId> {
        self.tabs[self.active_index].pane_tree.close_active()
    }

    /// Moves pane focus in the given direction within the active tab.
    pub(crate) fn move_pane_focus(&mut self, direction: PaneDirection) -> bool {
        self.tabs[self.active_index].pane_tree.move_focus(direction)
    }

    pub(crate) fn pane_tree_for_tab(&self, tab_id: TabId) -> Option<&PaneTree> {
        self.tabs
            .iter()
            .find(|t| t.id == tab_id)
            .map(|t| &t.pane_tree)
    }

    pub(crate) fn pane_tree_for_tab_mut(&mut self, tab_id: TabId) -> Option<&mut PaneTree> {
        self.tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .map(|t| &mut t.pane_tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_collection_has_one_tab() {
        let c = TabCollection::new();
        assert_eq!(c.len(), 1);
        assert_eq!(c.active_index(), 0);
        assert_eq!(c.active_id(), TabId::new(0));
    }

    #[test]
    fn add_increases_count_and_activates_new_tab() {
        let mut c = TabCollection::new();
        let (id1, idx1) = c.add();
        assert_eq!(c.len(), 2);
        assert_eq!(idx1, 1);
        assert_eq!(c.active_index(), 1);
        assert_eq!(id1, TabId::new(1));

        let (id2, idx2) = c.add();
        assert_eq!(c.len(), 3);
        assert_eq!(idx2, 2);
        assert_eq!(c.active_index(), 2);
        assert_eq!(id2, TabId::new(2));
    }

    #[test]
    fn remove_active_adjusts_index() {
        let mut c = TabCollection::new();
        let (_id1, _) = c.add();
        let (id2, _) = c.add();
        assert_eq!(c.active_index(), 2);

        c.remove(id2);
        assert_eq!(c.len(), 2);
        assert_eq!(c.active_index(), 1);
    }

    #[test]
    fn remove_non_active_keeps_index() {
        let mut c = TabCollection::new();
        c.add();
        let (id2, _) = c.add();
        // active is 2
        c.set_active(0);

        c.remove(id2);
        assert_eq!(c.len(), 2);
        assert_eq!(c.active_index(), 0);
    }

    #[test]
    fn removing_tab_before_active_preserves_active_id() {
        let mut c = TabCollection::new();
        let (id1, _) = c.add();
        let (id2, _) = c.add();
        c.add();
        c.set_active(2);
        assert_eq!(c.active_id(), id2);

        c.remove(id1);

        assert_eq!(c.active_id(), id2);
        assert_eq!(c.active_index(), 1);
    }

    #[test]
    fn remove_last_tab_empties_collection() {
        let mut c = TabCollection::new();
        let id = c.tabs[0].id;
        c.remove(id);
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn remove_non_existent_returns_none() {
        let mut c = TabCollection::new();
        assert!(c.remove(TabId::new(999)).is_none());
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn next_prev_cycle() {
        let mut c = TabCollection::new();
        c.add();
        c.add();
        // tabs: [0, 1, 2], active: 2

        c.next();
        assert_eq!(c.active_index(), 0);
        c.next();
        assert_eq!(c.active_index(), 1);
        c.prev();
        assert_eq!(c.active_index(), 0);
        c.prev();
        assert_eq!(c.active_index(), 2);
    }

    #[test]
    fn next_prev_single_tab_no_op() {
        let mut c = TabCollection::new();
        c.next();
        assert_eq!(c.active_index(), 0);
        c.prev();
        assert_eq!(c.active_index(), 0);
    }

    #[test]
    fn set_active_clamps() {
        let mut c = TabCollection::new();
        c.set_active(0);
        assert_eq!(c.active_index(), 0);
        c.set_active(5);
        assert_eq!(c.active_index(), 0);
    }

    #[test]
    fn ids_are_unique() {
        let mut c = TabCollection::new();
        let (id1, _) = c.add();
        let (id2, _) = c.add();
        assert_ne!(id1, id2);
    }

    #[test]
    fn set_title_updates_correct_tab() {
        let mut c = TabCollection::new();
        let (id, _) = c.add();
        c.set_title(id, String::from("zsh"));
        assert_eq!(c.tabs[1].title, "zsh");
        assert_eq!(c.tabs[0].title, "IV");
    }
}
