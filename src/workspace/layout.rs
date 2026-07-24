//! Layout-Snapshots.
//!
//! Ein [`LayoutSnapshot`] ist die serialisierbare Form des zuletzt
//! aktiven Layouts: Tabs, Pane-Bäume, benutzerdefinierte Titel und
//! das aktive Profil. Er kennt keine GTK- oder VTE-Typen und kann
//! deswegen im `workspace/`-Modul verbleiben.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::pane::{Orientation, PaneId, PaneNode, PaneTree, SplitId};
use crate::tab::{TabCollection, TabId};
use crate::workspace::StartConfig;
use crate::workspace::error::{WorkspaceError, WorkspaceWarning};
use crate::workspace::profile::ProfileId;

/// Persistierbarer Zustand des zuletzt aktiven Layouts.
///
/// `active_profile_id` ist **getrennt** vom Tab-Baum gespeichert: wird
/// das aktive Profil gelöscht, fällt das Layout stillschweigend auf
/// „kein Profil" zurück, behält aber die Tabs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LayoutSnapshot {
    pub(crate) schema_version: u32,
    pub(crate) active_profile_id: Option<ProfileId>,
    pub(crate) active_tab_index: usize,
    pub(crate) tabs: Vec<LayoutTab>,
}

impl LayoutSnapshot {
    /// Erzeugt einen leeren Snapshot mit aktueller Schema-Version.
    pub(crate) fn empty() -> Self {
        Self {
            schema_version: crate::workspace::SCHEMA_VERSION,
            active_profile_id: None,
            active_tab_index: 0,
            tabs: Vec::new(),
        }
    }

    /// Erzeugt einen Snapshot aus dem aktuellen Tab-Zustand.
    pub(crate) fn from_collection(
        collection: &TabCollection,
        active_profile_id: Option<ProfileId>,
    ) -> Self {
        let tabs: Vec<LayoutTab> = collection.tabs().iter().map(LayoutTab::from_tab).collect();
        Self {
            schema_version: crate::workspace::SCHEMA_VERSION,
            active_profile_id,
            active_tab_index: collection.active_index(),
            tabs,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), WorkspaceError> {
        if self.schema_version != crate::workspace::SCHEMA_VERSION {
            return Err(WorkspaceError::UnsupportedVersion(self.schema_version));
        }
        for (index, tab) in self.tabs.iter().cloned().enumerate() {
            let id = u64::try_from(index).map_err(|_| WorkspaceError::InvalidLayout)?;
            tab.into_tab(TabId::new(id))?;
        }
        Ok(())
    }

    /// Stellt aus dem Snapshot einen [`TabCollection`] wieder her.
    ///
    /// Falls `active_profile_id` auf eine ID zeigt, die in `profiles`
    /// nicht vorkommt, wird `active_profile_id` auf `None` gesetzt und
    /// [`WorkspaceWarning::ActiveProfileMissing`] zurückgegeben.
    pub(crate) fn into_collection(
        snapshot: LayoutSnapshot,
        profiles: &BTreeMap<ProfileId, ()>,
    ) -> Result<(TabCollection, Option<ProfileId>, Vec<WorkspaceWarning>), WorkspaceError> {
        if snapshot.schema_version != crate::workspace::SCHEMA_VERSION {
            return Err(WorkspaceError::UnsupportedVersion(snapshot.schema_version));
        }

        let mut warnings = Vec::new();
        let active_profile_id = if let Some(id) = snapshot.active_profile_id {
            if profiles.contains_key(&id) {
                Some(id)
            } else {
                warnings.push(WorkspaceWarning::ActiveProfileMissing);
                None
            }
        } else {
            None
        };

        let tabs = snapshot
            .tabs
            .into_iter()
            .enumerate()
            .map(|(index, tab)| {
                let id = u64::try_from(index).map_err(|_| WorkspaceError::InvalidLayout)?;
                tab.into_tab(TabId::new(id))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let collection = TabCollection::from_tabs(tabs, snapshot.active_tab_index)
            .ok_or(WorkspaceError::InvalidLayout)?;

        Ok((collection, active_profile_id, warnings))
    }
}

/// Persistierbare Form eines Tabs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LayoutTab {
    pub(crate) title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) custom_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) start_config: Option<StartConfig>,
    pub(crate) root: LayoutPane,
}

impl LayoutTab {
    fn from_tab(tab: &crate::tab::TabInfo) -> Self {
        Self {
            title: tab.title.clone(),
            custom_title: tab.custom_title.clone(),
            start_config: tab.start_config.clone(),
            root: LayoutPane::from_node(tab.pane_tree.root()),
        }
    }

    fn into_tab(self, id: TabId) -> Result<crate::tab::TabInfo, WorkspaceError> {
        if let Some(config) = &self.start_config {
            config.validate()?;
        }
        let root = self.root.into_node();
        let pane_tree = PaneTree::from_root(root).ok_or(WorkspaceError::InvalidLayout)?;
        let mut tab = crate::tab::TabInfo::new(id, self.title, pane_tree);
        tab.custom_title = self.custom_title;
        tab.start_config = self.start_config;
        Ok(tab)
    }
}

/// Persistierbare Form eines Pane-Baum-Knotens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LayoutPane {
    Terminal(LayoutTerminal),
    Split(LayoutSplit),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LayoutTerminal {
    pub(crate) id: u64,
    pub(crate) title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LayoutSplit {
    pub(crate) id: u64,
    pub(crate) orientation: LayoutOrientation,
    pub(crate) ratio: f64,
    pub(crate) first: Box<LayoutPane>,
    pub(crate) second: Box<LayoutPane>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LayoutOrientation {
    Horizontal,
    Vertical,
}

impl From<Orientation> for LayoutOrientation {
    fn from(value: Orientation) -> Self {
        match value {
            Orientation::Horizontal => Self::Horizontal,
            Orientation::Vertical => Self::Vertical,
        }
    }
}

impl From<LayoutOrientation> for Orientation {
    fn from(value: LayoutOrientation) -> Self {
        match value {
            LayoutOrientation::Horizontal => Self::Horizontal,
            LayoutOrientation::Vertical => Self::Vertical,
        }
    }
}

impl LayoutPane {
    fn from_node(node: &PaneNode) -> Self {
        match node {
            PaneNode::Terminal { id, title } => Self::Terminal(LayoutTerminal {
                id: (*id).as_u64(),
                title: title.clone(),
            }),
            PaneNode::Split {
                id,
                orientation,
                ratio,
                first,
                second,
            } => Self::Split(LayoutSplit {
                id: (*id).as_u64(),
                orientation: (*orientation).into(),
                ratio: *ratio,
                first: Box::new(LayoutPane::from_node(first)),
                second: Box::new(LayoutPane::from_node(second)),
            }),
        }
    }

    fn into_node(self) -> PaneNode {
        match self {
            Self::Terminal(terminal) => PaneNode::Terminal {
                id: PaneId::new(terminal.id),
                title: terminal.title,
            },
            Self::Split(split) => PaneNode::Split {
                id: SplitId::new(split.id),
                orientation: split.orientation.into(),
                ratio: split.ratio,
                first: Box::new(split.first.into_node()),
                second: Box::new(split.second.into_node()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pane::Orientation;
    use crate::workspace::StartConfig;

    #[test]
    fn empty_snapshot_round_trip() {
        let snapshot = LayoutSnapshot::empty();
        let (collection, active_profile, warnings) =
            LayoutSnapshot::into_collection(snapshot, &BTreeMap::new()).unwrap();
        assert_eq!(collection.tabs().len(), 1);
        assert_eq!(active_profile, None);
        assert!(warnings.is_empty());
    }

    #[test]
    fn unknown_active_profile_becomes_none() {
        let mut snapshot = LayoutSnapshot::empty();
        snapshot.active_profile_id = Some(ProfileId::new(99));
        let (collection, active_profile, warnings) =
            LayoutSnapshot::into_collection(snapshot, &BTreeMap::new()).unwrap();
        assert_eq!(active_profile, None);
        assert_eq!(warnings, vec![WorkspaceWarning::ActiveProfileMissing]);
        assert_eq!(collection.tabs().len(), 1);
    }

    #[test]
    fn known_active_profile_preserved() {
        let mut snapshot = LayoutSnapshot::empty();
        snapshot.active_profile_id = Some(ProfileId::new(7));
        let mut profiles = BTreeMap::new();
        profiles.insert(ProfileId::new(7), ());
        let (_, active_profile, warnings) =
            LayoutSnapshot::into_collection(snapshot, &profiles).unwrap();
        assert_eq!(active_profile, Some(ProfileId::new(7)));
        assert!(warnings.is_empty());
    }

    #[test]
    fn from_collection_round_trip_preserves_tabs() {
        let first = crate::tab::TabInfo::new(TabId::new(0), "IV".to_owned(), PaneTree::new());
        let mut second =
            crate::tab::TabInfo::new(TabId::new(1), "Recherche".to_owned(), PaneTree::new());
        second.custom_title = Some("Recherche".to_owned());
        second.start_config = Some(StartConfig::new("/tmp", None, None).expect("schema-valid"));
        second
            .pane_tree
            .set_title(second.pane_tree.active_id(), "editor".to_owned());
        let collection = TabCollection::from_tabs(vec![first, second], 1).expect("valid tabs");

        let snapshot = LayoutSnapshot::from_collection(&collection, None);
        let (restored, _, warnings) =
            LayoutSnapshot::into_collection(snapshot, &BTreeMap::new()).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(restored.tabs().len(), collection.tabs().len());
        let original = &collection.tabs()[1];
        let restored_tab = &restored.tabs()[1];
        assert_eq!(restored_tab.title, original.title);
        assert_eq!(restored_tab.custom_title, original.custom_title);
        assert_eq!(restored_tab.start_config, original.start_config);
    }

    #[test]
    fn split_orientation_round_trip() {
        // Build a snapshot with a single horizontal split, manually.
        let snapshot = LayoutSnapshot {
            schema_version: 1,
            active_profile_id: None,
            active_tab_index: 0,
            tabs: vec![LayoutTab {
                title: "T".into(),
                custom_title: None,
                start_config: None,
                root: LayoutPane::Split(LayoutSplit {
                    id: 0,
                    orientation: LayoutOrientation::Horizontal,
                    ratio: 0.5,
                    first: Box::new(LayoutPane::Terminal(LayoutTerminal {
                        id: 0,
                        title: "left".into(),
                    })),
                    second: Box::new(LayoutPane::Terminal(LayoutTerminal {
                        id: 1,
                        title: "right".into(),
                    })),
                }),
            }],
        };
        let (collection, _, _) =
            LayoutSnapshot::into_collection(snapshot, &BTreeMap::new()).unwrap();
        assert_eq!(collection.tabs().len(), 1);
        assert!(!collection.tabs()[0].pane_tree.is_single());
        let split_node = collection.tabs()[0].pane_tree.root();
        assert!(matches!(
            split_node,
            PaneNode::Split {
                orientation: Orientation::Horizontal,
                ..
            }
        ));
    }
}
