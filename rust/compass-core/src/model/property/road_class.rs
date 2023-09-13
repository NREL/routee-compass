use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug, Default, PartialOrd, Ord, Hash,
)]
pub struct RoadClass(pub u8);