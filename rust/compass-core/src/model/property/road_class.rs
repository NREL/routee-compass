use serde::Deserialize;

#[derive(Copy, Clone, Eq, PartialEq, Deserialize, Debug)]
pub struct RoadClass(pub u8);
