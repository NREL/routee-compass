use serde::{Deserialize, Serialize};

use super::Energy;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum EnergyUnit {
    GallonsGasoline,
    KilowattHours,
}

impl EnergyUnit {
    pub fn convert(&self, value: Energy, target: EnergyUnit) -> Energy {
        use EnergyUnit as S;
        match (self, target) {
            (S::GallonsGasoline, S::GallonsGasoline) => value,
            (S::GallonsGasoline, S::KilowattHours) => value * 33.41,
            (S::KilowattHours, S::GallonsGasoline) => value * 0.0299,
            (S::KilowattHours, S::KilowattHours) => value,
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
