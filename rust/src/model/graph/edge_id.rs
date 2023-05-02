use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub struct EdgeId(pub u64);

impl Ord for EdgeId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}
