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
    Joules
    // BTU
    // eV
    // calorie (small c)
}

impl Convert<Energy> for EnergyUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Energy>, to: &Self) -> Result<(), UnitError> {
        use EnergyUnit as S;
        use VolumeUnit as V;
        let conversion_factor = match (self, to) {
            (S::Gasoline(V::GallonsUs), S::Gasoline(V::GallonsUs)) => None,             // (S::GallonsGasoline, S::GallonsGasoline) => None,
            (S::Gasoline(V::GallonsUs), S::Gasoline(V::GallonsUk)) => Some(0.832674),
            (S::Gasoline(V::GallonsUs), S::Gasoline(V::Liters)) => Some(3.78541),       // (S::GallonsGasoline, S::LitersGasoline) => Some(3.78541),
            (S::Gasoline(V::GallonsUs), S::Diesel(V::GallonsUs)) => Some(0.866),        // (S::GallonsGasoline, S::GallonsDiesel) => Some(0.866),
            // GG_us->GD_uk: GG_us->GG_uk->GD_uk
            (S::Gasoline(V::GallonsUs), S::Diesel(V::GallonsUk)) => Some(0.832674 * 0.866),
            // GG->LD: GG -> GD -> LD
            (S::Gasoline(V::GallonsUs), S::Diesel(V::Liters)) => Some(0.866 * 3.78541), // (S::GallonsGasoline, S::LitersDiesel) => Some(0.866 * 3.78541),
            (S::Gasoline(V::GallonsUs), S::KilowattHours) => Some(32.26),               // (S::GallonsGasoline, S::KilowattHours) => Some(32.26),
            (S::Gasoline(V::GallonsUs), S::Joules) => Some(32.26 * 3_600_000.),
            
            (S::Gasoline(V::Liters), S::Gasoline(V::GallonsUs)) => Some(0.264172),      // (S::LitersGasoline, S::GallonsGasoline) => Some(0.264172),
            (S::Gasoline(V::Liters), S::Gasoline(V::GallonsUk)) => Some(0.219969),
            (S::Gasoline(V::Liters), S::Gasoline(V::Liters)) => None,                   // (S::LitersGasoline, S::LitersGasoline) => None,
            // LG->GD: LG -> LD -> GD
            (S::Gasoline(V::Liters), S::Diesel(V::GallonsUs)) => Some(0.264172 * 0.866),// (S::LitersGasoline, S::GallonsDiesel) => Some(0.264172 * 0.866),
            // LG->GD_uk: LG->GD_us->GD_uk
            (S::Gasoline(V::Liters), S::Diesel(V::GallonsUk)) => Some(0.264172 * 0.866 * 0.832674),
            // LG->LD: LG -> GG -> GD -> LD
            (S::Gasoline(V::Liters), S::Diesel(V::Liters)) => Some(0.866),              // (S::LitersGasoline, S::LitersDiesel) => Some(0.866),
            // LG->KWH: LG -> GG -> KWH
            (S::Gasoline(V::Liters), S::KilowattHours) => Some(0.264172 * 32.26),       // (S::LitersGasoline, S::KilowattHours) => Some(0.264172 * 32.26),
            (S::Gasoline(V::Liters), S::Joules) => Some(0.264172 * 32.26 * 3_600_000.),       
            
            (S::Diesel(V::GallonsUs), S::Gasoline(V::GallonsUs)) => Some(1.155),        // (S::GallonsDiesel, S::GallonsGasoline) => Some(1.155),
            (S::Diesel(V::GallonsUs), S::Gasoline(V::GallonsUk)) => Some(1.155 * 0.832674),
            // GD->LG: GD -> GG -> LG
            (S::Diesel(V::GallonsUs), S::Gasoline(V::Liters)) => Some(1.155 * 3.78541), // (S::GallonsDiesel, S::LitersGasoline) => Some(1.155 * 3.78541),
            (S::Diesel(V::GallonsUs), S::Diesel(V::GallonsUs)) => None,                 // (S::GallonsDiesel, S::GallonsDiesel) => None,
            (S::Diesel(V::GallonsUs), S::Diesel(V::GallonsUk)) => Some(0.832674),
            (S::Diesel(V::GallonsUs), S::Diesel(V::Liters)) => Some(3.78541),           // (S::GallonsDiesel, S::LitersDiesel) => Some(3.78541),
            (S::Diesel(V::GallonsUs), S::KilowattHours) => Some(40.7),                  // (S::GallonsDiesel, S::KilowattHours) => Some(40.7),
            (S::Diesel(V::GallonsUs), S::Joules) => Some(40.7 * 3_600_000.),
            
            (S::Diesel(V::Liters), S::Gasoline(V::GallonsUs)) => Some(0.264172 * 1.155),// (S::LitersDiesel, S::GallonsGasoline) => Some(0.264172 * 1.155),
            (S::Diesel(V::Liters), S::Gasoline(V::GallonsUk)) => Some(0.264172 * 1.155 * 0.832674),
            // LD->LG: LD -> GD -> GG -> LG
            (S::Diesel(V::Liters), S::Gasoline(V::Liters)) => Some(1.155),              // (S::LitersDiesel, S::LitersGasoline) => Some(1.155),
            // LD->GG: LD -> LG -> GG
            (S::Diesel(V::Liters), S::Diesel(V::GallonsUs)) => Some(0.264172),          // (S::LitersDiesel, S::GallonsDiesel) => Some(0.264172),
            (S::Diesel(V::Liters), S::Diesel(V::GallonsUk)) => Some(0.264172 * 0.832674),
            (S::Diesel(V::Liters), S::Diesel(V::Liters)) => None,                       // (S::LitersDiesel, S::LitersDiesel) => None,
            // LD->KWH: LD -> GD -> KWH
            (S::Diesel(V::Liters), S::KilowattHours) => Some(0.264172 * 40.7),          // (S::LitersDiesel, S::KilowattHours) => Some(0.264172 * 40.7),
            (S::Diesel(V::Liters), S::Joules) => Some(0.264172 * 40.7 * 3_600_000.),
            
            (S::KilowattHours, S::Gasoline(V::GallonsUs)) => Some(0.031),               // (S::KilowattHours, S::GallonsGasoline) => Some(0.031),
            (S::KilowattHours, S::Gasoline(V::GallonsUk)) => Some(0.031 * 0.832674),
            // KWH->LG: KWH -> GG -> LG
            (S::KilowattHours, S::Gasoline(V::Liters)) => Some(0.031 * 3.78541),        // (S::KilowattHours, S::LitersGasoline) => Some(0.031 * 3.78541),
            (S::KilowattHours, S::Diesel(V::GallonsUs)) => Some(0.02457),               // (S::KilowattHours, S::GallonsDiesel) => Some(0.02457),
            (S::KilowattHours, S::Diesel(V::GallonsUk)) => Some(0.02457 * 0.832674),
            // KWH->LD: KWH -> GD -> LD
            (S::KilowattHours, S::Diesel(V::Liters)) => Some(0.02457 * 3.78541),        // (S::KilowattHours, S::LitersDiesel) => Some(0.02457 * 3.78541),
            (S::KilowattHours, S::KilowattHours) => None,
            (S::KilowattHours, S::Joules) => Some(3_600_000.),
            
            (S::Gasoline(V::GallonsUk), S::Gasoline(V::GallonsUs)) => Some(1.20095),
            (S::Gasoline(V::GallonsUk), S::Gasoline(V::GallonsUk)) => None,
            (S::Gasoline(V::GallonsUk), S::Gasoline(V::Liters)) => Some(4.54609),
            (S::Gasoline(V::GallonsUk), S::Diesel(V::GallonsUs)) => Some(1.20095 * 0.866),
            (S::Gasoline(V::GallonsUk), S::Diesel(V::GallonsUk)) => Some(0.866),
            (S::Gasoline(V::GallonsUk), S::Diesel(V::Liters)) => Some(4.54609 * 0.866),
            (S::Gasoline(V::GallonsUk), S::KilowattHours) => Some(1.20095 * 32.26),
            (S::Gasoline(V::GallonsUk), S::Joules) => Some(1.20095 * 32.26 * 3_600_000.),
            
            (S::Diesel(V::GallonsUk), S::Gasoline(V::GallonsUs)) => Some(1.20095 * 1.155),
            (S::Diesel(V::GallonsUk), S::Gasoline(V::GallonsUk)) => Some(1.155),
            (S::Diesel(V::GallonsUk), S::Gasoline(V::Liters)) => Some(1.155 * 4.54609),
            (S::Diesel(V::GallonsUk), S::Diesel(V::GallonsUs)) => Some(1.20095),
            (S::Diesel(V::GallonsUk), S::Diesel(V::GallonsUk)) => None,
            (S::Diesel(V::GallonsUk), S::Diesel(V::Liters)) => Some(4.54609),
            (S::Diesel(V::GallonsUk), S::KilowattHours) => Some(1.20095 * 40.7),
            (S::Diesel(V::GallonsUk), S::Joules) => Some(1.20095 * 40.7 * 3_600_000.),

            // These are all transformed to kWh and then apply the kWh -> * conversion
            (S::Joules, S::Gasoline(V::GallonsUs)) => Some(0.000000277778 * 0.031),
            (S::Joules, S::Gasoline(V::GallonsUk)) => Some(0.000000277778 * 0.031 * 0.832674),
            (S::Joules, S::Gasoline(V::Liters)) => Some(0.000000277778 * 0.031 * 3.78541),
            (S::Joules, S::Diesel(V::GallonsUs)) => Some(0.000000277778 * 0.02457),
            (S::Joules, S::Diesel(V::GallonsUk)) => Some(0.000000277778 * 0.02457 * 0.832674),
            (S::Joules, S::Diesel(V::Liters)) => Some(0.000000277778 * 0.02457 * 3.78541),
            (S::Joules, S::KilowattHours) => Some(0.000000277778),
            (S::Joules, S::Joules) => None,
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
