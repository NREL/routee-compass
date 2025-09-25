use std::{cmp::Ordering, fmt::Display};

use allocative::Allocative;
use serde::{Deserialize, Serialize};

/// denotes a list of edges that cover the graph. some subsets of the
/// graph may be covered by separate sub-graphs when the modes of transportation
/// are disjoint. for example, to model a transit, ferry, or airplane transportation
/// layer, a separate edge list is required, which are connected to the same set of
/// graph vertices as the other graphs.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Default, Allocative)]
pub struct EdgeListId(pub usize);

impl PartialOrd for EdgeListId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EdgeListId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for EdgeListId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl EdgeListId {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}
