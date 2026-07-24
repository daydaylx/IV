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

#[test]
fn closing_inactive_pane_preserves_active_pane() {
    let mut tree = PaneTree::new();
    let first = tree.active_id();
    let second = tree.split_active(Orientation::Horizontal);
    let third = tree.split_active(Orientation::Vertical);
    assert_eq!(tree.active_id(), third);

    assert_eq!(tree.close(second), Some(third));
    assert_eq!(tree.active_id(), third);
    assert!(tree.contains(first));
    assert!(!tree.contains(second));
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
    if let PaneNode::Terminal { title, .. } = tree.root() {
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

#[test]
fn title_returns_the_requested_pane_title() {
    let mut tree = PaneTree::new();
    let first = tree.active_id();
    let second = tree.split_active(Orientation::Horizontal);
    tree.set_title(first, "first".to_owned());
    tree.set_title(second, "second".to_owned());

    assert_eq!(tree.title(first), Some("first"));
    assert_eq!(tree.title(second), Some("second"));
    assert_eq!(tree.title(PaneId::new(999)), None);
}

#[test]
fn split_ratio_is_clamped_and_rejects_non_finite_values() {
    let mut tree = PaneTree::new();
    tree.split_active(Orientation::Horizontal);
    let split_id = match tree.root() {
        PaneNode::Split { id, .. } => *id,
        PaneNode::Terminal { .. } => panic!("expected split root"),
    };

    assert!(tree.set_split_ratio(split_id, 0.0));
    assert!(matches!(
        tree.root(),
        PaneNode::Split { ratio, .. } if *ratio == PaneTree::MIN_SPLIT_RATIO
    ));
    assert!(tree.set_split_ratio(split_id, 1.0));
    assert!(matches!(
        tree.root(),
        PaneNode::Split { ratio, .. } if *ratio == PaneTree::MAX_SPLIT_RATIO
    ));
    assert!(!tree.set_split_ratio(split_id, f64::NAN));
}

#[test]
fn restored_tree_rejects_invalid_state_and_continues_ids() {
    let root = PaneNode::Split {
        id: SplitId::new(7),
        orientation: Orientation::Horizontal,
        ratio: 0.5,
        first: Box::new(PaneNode::Terminal {
            id: PaneId::new(4),
            title: "left".to_owned(),
        }),
        second: Box::new(PaneNode::Terminal {
            id: PaneId::new(9),
            title: "right".to_owned(),
        }),
    };
    let mut tree = PaneTree::from_root(root).expect("valid restored tree");
    assert_eq!(tree.active_id(), PaneId::new(4));
    assert_eq!(tree.split_active(Orientation::Vertical), PaneId::new(10));

    let duplicate = PaneNode::Split {
        id: SplitId::new(0),
        orientation: Orientation::Horizontal,
        ratio: 0.5,
        first: Box::new(PaneNode::Terminal {
            id: PaneId::new(1),
            title: String::new(),
        }),
        second: Box::new(PaneNode::Terminal {
            id: PaneId::new(1),
            title: String::new(),
        }),
    };
    assert!(PaneTree::from_root(duplicate).is_none());
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
