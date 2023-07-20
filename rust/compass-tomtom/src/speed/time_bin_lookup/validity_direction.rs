use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum ValidityDirection {
    BothDirections = 1,
    PositiveDirection = 2,
    NegativeDirection = 3,
    NotPresent = 9,
}
