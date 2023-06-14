use std::{cmp::Ordering, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Debug, Default)]
pub struct VertexId(pub u64);

impl Ord for VertexId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl Display for VertexId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
