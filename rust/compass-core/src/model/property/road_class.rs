use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct RoadClass(pub u8);
