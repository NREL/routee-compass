use super::{baseunit, Convert, Energy};
use crate::{model::unit::AsF64, util::serde::serde_ops::string_deserialize};
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

impl Convert<Energy> for EnergyUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Energy>, to: &Self) {
        use EnergyUnit as S;
        let conversion_factor = match (self, to) {
            (S::GallonsGasoline, S::GallonsGasoline) => None,
            (S::GallonsGasoline, S::KilowattHours) => Some(32.26),
            (S::GallonsGasoline, S::LitersGasoline) => Some(3.785),
            // GG->LD: GG -> GD -> LD
            (S::GallonsGasoline, S::LitersDiesel) => Some(0.866 * 3.785),
            (S::KilowattHours, S::GallonsGasoline) => Some(0.031),
            (S::KilowattHours, S::KilowattHours) => None,
            // KWH->LG: KWH -> GG -> LG
            (S::KilowattHours, S::LitersGasoline) => Some(0.031 * 3.785),
            // KWH->LD: KWH -> GD -> LD
            (S::KilowattHours, S::LitersDiesel) => Some(0.02457 * 3.785),
            (S::GallonsDiesel, S::GallonsDiesel) => None,
            (S::GallonsDiesel, S::KilowattHours) => Some(40.7),
            // GD->LG: GD -> GG -> LG
            (S::GallonsDiesel, S::LitersGasoline) => Some(1.155 * 3.785),
            (S::GallonsDiesel, S::LitersDiesel) => Some(3.785),
            (S::KilowattHours, S::GallonsDiesel) => Some(0.02457),
            (S::GallonsDiesel, S::GallonsGasoline) => Some(1.155),
            (S::GallonsGasoline, S::GallonsDiesel) => Some(0.866),
            (S::LitersGasoline, S::LitersGasoline) => None,
            // LG->LD: LG -> GG -> GD -> LD
            (S::LitersGasoline, S::LitersDiesel) => Some(0.866),
            (S::LitersGasoline, S::GallonsGasoline) => Some(0.264),
            // LG->GD: LG -> LD -> GD
            (S::LitersGasoline, S::GallonsDiesel) => Some(0.264 * 0.866),
            // LG->KWH: LG -> GG -> KWH
            (S::LitersGasoline, S::KilowattHours) => Some(0.264 * 32.26),
            (S::LitersDiesel, S::LitersDiesel) => None,
            // LD->LG: LD -> GD -> GG -> LG
            (S::LitersDiesel, S::LitersGasoline) => Some(1.155),
            // LD->GG: LD -> LG -> GG
            (S::LitersDiesel, S::GallonsGasoline) => Some(0.264 * 1.155),
            (S::LitersDiesel, S::GallonsDiesel) => Some(0.264),
            // LD->KWH: LD -> GD -> KWH
            (S::LitersDiesel, S::KilowattHours) => Some(0.264 * 40.7),
        };
        if let Some(factor) = conversion_factor {
            let mut updated = Energy::from(value.as_ref().as_f64() * factor);
            let value_mut = value.to_mut();
            std::mem::swap(value_mut, &mut updated);
        }
    }

    fn convert_to_base(&self, value: &mut std::borrow::Cow<Energy>) {
        self.convert(value, &baseunit::ENERGY_UNIT)
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
