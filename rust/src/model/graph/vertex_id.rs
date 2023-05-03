use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Debug)]
pub struct VertexId(pub u64);

impl Display for VertexId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
