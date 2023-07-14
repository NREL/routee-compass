use serde::Deserialize;

#[derive(Copy, Clone, Eq, PartialEq, Deserialize, Debug, Default)]
pub struct RoadClass(pub u8);
