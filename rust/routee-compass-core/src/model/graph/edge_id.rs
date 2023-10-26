use std::{cmp::Ordering, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Debug, Default)]
pub struct EdgeId(pub usize);

impl Ord for EdgeId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for EdgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl EdgeId {
    pub fn as_usize(&self) -> usize {
        return self.0;
    }
}
