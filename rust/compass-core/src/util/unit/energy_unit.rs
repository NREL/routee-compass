use super::{Distance, DistanceUnit, Energy, EnergyRate, EnergyRateUnit, UnitError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EnergyUnit {
    GallonsGasoline,
    KilowattHours,
}

impl EnergyUnit {}
