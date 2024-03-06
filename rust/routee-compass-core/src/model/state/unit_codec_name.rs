use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitCodecType {
    FloatingPoint,
    SignedInteger,
    UnsignedInteger,
    Boolean,
}

impl ToString for UnitCodecType {
    fn to_string(&self) -> String {
        match self {
            UnitCodecType::FloatingPoint => String::from("floating_point"),
            UnitCodecType::SignedInteger => String::from("signed_integer"),
            UnitCodecType::UnsignedInteger => String::from("unsigned_integer"),
            UnitCodecType::Boolean => String::from("boolean"),
        }
    }
}
