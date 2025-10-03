use serde::{Deserialize, Serialize};

use crate::model::unit::DistanceUnit;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DistanceTraversalConfig {
    pub distance_unit: Option<DistanceUnit>,
}
