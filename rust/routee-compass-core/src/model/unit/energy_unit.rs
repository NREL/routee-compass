use super::Energy;
use crate::util::serde_ops::string_deserialize;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum EnergyUnit {
    GallonsGasoline,
    GallonsDiesel,
    KilowattHours,
}

impl EnergyUnit {
    // see https://epact.energy.gov/fuel-conversion-factors
    pub fn convert(&self, value: Energy, target: EnergyUnit) -> Energy {
        use EnergyUnit as S;
        match (self, target) {
            (S::GallonsGasoline, S::GallonsGasoline) => value,
            (S::GallonsGasoline, S::KilowattHours) => value * 32.26,
            (S::KilowattHours, S::GallonsGasoline) => value * 0.031,
            (S::KilowattHours, S::KilowattHours) => value,
            (S::GallonsDiesel, S::GallonsDiesel) => value,
            (S::GallonsDiesel, S::KilowattHours) => value * 40.7,
            (S::KilowattHours, S::GallonsDiesel) => value * 0.02457,
            (S::GallonsDiesel, S::GallonsGasoline) => value * 1.155,
            (S::GallonsGasoline, S::GallonsDiesel) => value * 0.866,
        }
    }
}

impl std::fmt::Display for EnergyUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

impl FromStr for EnergyUnit {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        string_deserialize(s)
    }
}
