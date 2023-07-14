use serde::Deserialize;

#[derive(Copy, Clone, Eq, PartialEq, Deserialize, Debug, Default)]
pub struct Millis(pub i16);
