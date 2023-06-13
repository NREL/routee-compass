use serde::Deserialize;

#[derive(Copy, Clone, Eq, PartialEq, Deserialize, Debug)]
pub struct Millis(pub i16);
