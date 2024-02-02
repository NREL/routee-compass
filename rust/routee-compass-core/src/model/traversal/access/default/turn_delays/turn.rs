use crate::model::traversal::access::access_model_error::AccessModelError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Turn {
    NoTurn,
    SlightRight,
    SlightLeft,
    Right,
    Left,
    SharpRight,
    SharpLeft,
    UTurn,
}

impl Turn {
    pub fn from_angle(angle: i16) -> Result<Self, AccessModelError> {
        match angle {
            -180..=-160 => Ok(Turn::UTurn),
            -159..=-135 => Ok(Turn::SharpLeft),
            -134..=-45 => Ok(Turn::Left),
            -44..=-20 => Ok(Turn::SlightLeft),
            -19..=19 => Ok(Turn::NoTurn),
            20..=44 => Ok(Turn::SlightRight),
            45..=134 => Ok(Turn::Right),
            135..=159 => Ok(Turn::SharpRight),
            160..=180 => Ok(Turn::UTurn),
            _ => Err(AccessModelError::RuntimeError {
                name: String::from("turn delays"),
                error: format!("Angle {0} out of range of -180 to 180", angle),
            }),
        }
    }
}
