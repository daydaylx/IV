mod navigation;
mod node;
mod tree;

#[cfg(test)]
mod tests;

pub(crate) use node::PaneNode;
pub(crate) use tree::PaneTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PaneId(u64);

impl PaneId {
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }

    #[allow(dead_code, reason = "used by the not-yet-wired Phase-2 layout adapter")]
    pub(crate) fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SplitId(u64);

impl SplitId {
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }

    #[allow(dead_code, reason = "used by the not-yet-wired Phase-2 layout adapter")]
    pub(crate) fn as_u64(self) -> u64 {
        self.0
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
