use super::{baseunit, Convert, Energy, UnitError, VolumeUnit};
use crate::model::unit::AsF64;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy)]
#[serde(rename_all = "snake_case", try_from = "String")]
pub enum EnergyUnit {
    /// electric fuel
    KilowattHours,
    /// SI liters of gasoline fuel
    LitersGasoline,
    /// SI liters of diesel fuel
    LitersDiesel,
    /// unit representing either electric or liquid fuel
    GallonsGasolineEquivalent,
    /// 1 [VolumeUnit] Gasoline fuel
    Gasoline(VolumeUnit),
    /// 1 [VolumeUnit] Diesel fuel
    Diesel(VolumeUnit),
    // Joules
    // BTU
    // eV
    // calorie (small c)
}

impl Convert<Energy> for EnergyUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Energy>, to: &Self) -> Result<(), UnitError> {
        use EnergyUnit as S;
        use VolumeUnit as V;
        let conversion_factor = match (self, to) {
            (S::GallonsGasoline, S::GallonsGasoline) => None,
            (S::GallonsGasoline, S::KilowattHours) => Some(32.26),
            (S::GallonsGasoline, S::LitersGasoline) => Some(3.78541),
            (S::GallonsGasoline, S::LitersDiesel) => Some(0.866 * 3.78541),
            (S::KilowattHours, S::GallonsGasoline) => Some(0.031),
            (S::KilowattHours, S::KilowattHours) => None,
            (S::KilowattHours, S::LitersGasoline) => Some(0.031 * 3.78541),
            (S::KilowattHours, S::LitersDiesel) => Some(0.02457 * 3.78541),
            (S::GallonsDiesel, S::GallonsDiesel) => None,
            (S::GallonsDiesel, S::KilowattHours) => Some(40.7),
            (S::GallonsDiesel, S::LitersGasoline) => Some(1.155 * 3.78541),
            (S::GallonsDiesel, S::LitersDiesel) => Some(3.78541),
            (S::KilowattHours, S::GallonsDiesel) => Some(0.02457),
            (S::GallonsDiesel, S::GallonsGasoline) => Some(1.155),
            (S::GallonsGasoline, S::GallonsDiesel) => Some(0.866),
            (S::LitersGasoline, S::LitersGasoline) => None,
            (S::LitersGasoline, S::LitersDiesel) => Some(0.866),
            (S::LitersGasoline, S::GallonsGasoline) => Some(0.264172),
            (S::LitersGasoline, S::GallonsDiesel) => Some(0.264172 * 0.866),
            (S::LitersGasoline, S::KilowattHours) => Some(0.264172 * 32.26),
            (S::LitersDiesel, S::LitersDiesel) => None,
            (S::LitersDiesel, S::LitersGasoline) => Some(1.155),
            (S::LitersDiesel, S::GallonsGasoline) => Some(0.264172 * 1.155),
            (S::LitersDiesel, S::GallonsDiesel) => Some(0.264172),
            (S::LitersDiesel, S::KilowattHours) => Some(0.264172 * 40.7),
            (S::GallonsGasoline, S::GallonsGasolineEquivalent) => None,
            (S::GallonsDiesel, S::GallonsGasolineEquivalent) => Some(1.14),
            (S::KilowattHours, S::GallonsGasolineEquivalent) => Some(0.03),
            (S::LitersGasoline, S::GallonsGasolineEquivalent) => Some(0.264172),
            (S::LitersDiesel, S::GallonsGasolineEquivalent) => Some(3.78541 * 1.14),
            (S::GallonsGasolineEquivalent, S::GallonsGasoline) => None,
            (S::GallonsGasolineEquivalent, S::GallonsDiesel) => Some(0.8771929825),
            (S::GallonsGasolineEquivalent, S::KilowattHours) => Some(33.3333333333),
            (S::GallonsGasolineEquivalent, S::LitersGasoline) => todo!(),
            (S::GallonsGasolineEquivalent, S::LitersDiesel) => todo!(),
            (S::GallonsGasolineEquivalent, S::GallonsGasolineEquivalent) => None,
        };
        if let Some(factor) = conversion_factor {
            let updated = Energy::from(value.as_ref().as_f64() * factor);
            *value.to_mut() = updated;
        }
        Ok(())
    }

    fn convert_to_base(&self, value: &mut std::borrow::Cow<Energy>) -> Result<(), UnitError> {
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
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use EnergyUnit as E;
        match s
            .trim()
            .to_lowercase()
            .replace("_", " ")
            .replace(" ", "")
            .as_str()
        {
            "gallonsgasoline" => Ok(E::Gasoline(VolumeUnit::GallonsUs)),
            "gallonsdiesel" => Ok(E::Diesel(VolumeUnit::GallonsUs)),
            "kilowatthours" | "kilowatthour" | "kwh" => Ok(E::KilowattHours),
            "litersgasoline" => Ok(E::LitersGasoline),
            "litersdiesel" => Ok(E::LitersDiesel),
            "gallonsgasolineequivalent" | "gge" => Ok(E::GallonsGasolineEquivalent),
            _ => Err(format!("unknown energy unit '{}'", s)),
        }
    }
}

impl TryFrom<String> for EnergyUnit {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
