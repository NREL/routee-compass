use super::Energy;
use crate::util::serde::serde_ops::string_deserialize;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum EnergyUnit {
    GallonsGasoline,
    GallonsDiesel,
    KilowattHours,
    LitersGasoline,
    LitersDiesel,
}

impl EnergyUnit {
    // see https://epact.energy.gov/fuel-conversion-factors
    // see https://www.eia.gov/energyexplained/units-and-calculators/energy-conversion-calculators.php
    pub fn convert(&self, value: &Energy, target: &EnergyUnit) -> Energy {
        use EnergyUnit as S;
        match (self, target) {
            (S::GallonsGasoline, S::GallonsGasoline) => *value,
            (S::GallonsGasoline, S::KilowattHours) => *value * 32.26,
            (S::GallonsGasoline, S::LitersGasoline) => *value * 3.785,
            // GG->LD: GG -> GD -> LD
            (S::GallonsGasoline, S::LitersDiesel) => *value * 0.866 * 3.785,
            (S::KilowattHours, S::GallonsGasoline) => *value * 0.031,
            (S::KilowattHours, S::KilowattHours) => *value,
            // KWH->LG: KWH -> GG -> LG
            (S::KilowattHours, S::LitersGasoline) => *value * 0.031 * 3.785,
            // KWH->LD: KWH -> GD -> LD
            (S::KilowattHours, S::LitersDiesel) => *value * 0.02457 * 3.785,
            (S::GallonsDiesel, S::GallonsDiesel) => *value,
            (S::GallonsDiesel, S::KilowattHours) => *value * 40.7,
            // GD->LG: GD -> GG -> LG
            (S::GallonsDiesel, S::LitersGasoline) => *value * 1.155 * 3.785,
            (S::GallonsDiesel, S::LitersDiesel) => *value * 3.785,
            (S::KilowattHours, S::GallonsDiesel) => *value * 0.02457,
            (S::GallonsDiesel, S::GallonsGasoline) => *value * 1.155,
            (S::GallonsGasoline, S::GallonsDiesel) => *value * 0.866,
            (S::LitersGasoline, S::LitersGasoline) => *value,
            // LG->LD: LG -> GG -> GD -> LD
            (S::LitersGasoline, S::LitersDiesel) => *value * 0.866,
            (S::LitersGasoline, S::GallonsGasoline) => *value * 0.264,
            // LG->GD: LG -> LD -> GD
            (S::LitersGasoline, S::GallonsDiesel) => *value * 0.264 * 0.866,
            // LG->KWH: LG -> GG -> KWH
            (S::LitersGasoline, S::KilowattHours) => *value * 0.264 * 32.26,
            (S::LitersDiesel, S::LitersDiesel) => *value,
            // LD->LG: LD -> GD -> GG -> LG
            (S::LitersDiesel, S::LitersGasoline) => *value * 1.155,
            // LD->GG: LD -> LG -> GG
            (S::LitersDiesel, S::GallonsGasoline) => *value * 0.264 * 1.155,
            (S::LitersDiesel, S::GallonsDiesel) => *value * 0.264,
            // LD->KWH: LD -> GD -> KWH
            (S::LitersDiesel, S::KilowattHours) => *value * 0.264 * 40.7,
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
