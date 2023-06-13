use serde::Deserialize;

#[derive(Copy, Clone, Eq, PartialEq, Deserialize, Debug)]
pub struct CmPerSecond(pub u32);
