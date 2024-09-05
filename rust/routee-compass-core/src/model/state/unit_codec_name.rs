use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitCodecType {
    FloatingPoint,
    SignedInteger,
    UnsignedInteger,
    Boolean,
}

impl Display for UnitCodecType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            UnitCodecType::FloatingPoint => String::from("floating_point"),
            UnitCodecType::SignedInteger => String::from("signed_integer"),
            UnitCodecType::UnsignedInteger => String::from("unsigned_integer"),
            UnitCodecType::Boolean => String::from("boolean"),
        };
        write!(f, "{}", msg)
    }
}
