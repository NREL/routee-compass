use serde::Deserialize;

#[derive(Copy, Clone, Eq, PartialEq, Deserialize, Debug, Default)]
pub struct CmPerSecond(pub u32);

impl CmPerSecond {
    pub const INVALID: CmPerSecond = CmPerSecond(0);
}
