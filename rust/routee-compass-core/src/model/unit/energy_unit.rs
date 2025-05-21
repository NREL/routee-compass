use super::{baseunit, Convert, Energy, UnitError, VolumeUnit};
use crate::model::unit::AsF64;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy)]
#[serde(rename_all = "snake_case", try_from = "String")]
pub enum EnergyUnit {
    /// electric fuel
    KilowattHours,
    /// 1 [VolumeUnit] Gasoline fuel
    Gasoline(VolumeUnit),
    /// 1 [VolumeUnit] Diesel fuel
    Diesel(VolumeUnit),
    /// unit representing either electric or liquid fuel
    GallonsGasolineEquivalent,
    GallonsDieselEquivalent,
    /// Other commonly-used energy units
    KiloJoules,
    BTU
}

impl Convert<Energy> for EnergyUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Energy>, to: &Self) -> Result<(), UnitError> {
        use EnergyUnit as S;
        use VolumeUnit as V;

        const USgal2UKgal: f64 = 0.832674;
        const UKgal2USgal: f64 = 1.20095;
        const USgal2L: f64 = 3.78541;
        const L2USgal: f64 = 0.264172;
        const UKgal2L: f64 = UKgal2USgal * USgal2L;
        const L2UKgal: f64 = L2USgal * USgal2UKgal;

        const Gas2Die: f64 = 0.8803;
        const Die2Gas: f64 = 1.13597;
        const Gas2Kwh: f64 = 33.41;
        const kWh2Gas: f64 = 0.029931;
        const kWh2Kj: f64 = 3_600.;
        const Kj2Kwh: f64 = 0.00027778;  // 1 / 3600
        const kWh2BTU: f64 = 3_412.14;
        const BTU2Kwh: f64 = 0.00029307; // 1 / 3412.14

        let conversion_factor = match (self, to) {
            (S::Gasoline(V::GallonsUs), S::Gasoline(V::GallonsUs)) => None,
            (S::Gasoline(V::GallonsUs), S::GallonsGasolineEquivalent) => None,
            (S::Gasoline(V::GallonsUs), S::Gasoline(V::GallonsUk)) => Some(USgal2UKgal),
            (S::Gasoline(V::GallonsUs), S::Gasoline(V::Liters)) => Some(USgal2L),
            (S::Gasoline(V::GallonsUs), S::Diesel(V::GallonsUs)) => Some(Gas2Die),
            (S::Gasoline(V::GallonsUs), S::GallonsDieselEquivalent) => Some(Gas2Die),
            (S::Gasoline(V::GallonsUs), S::Diesel(V::GallonsUk)) => Some(USgal2UKgal * Gas2Die),
            (S::Gasoline(V::GallonsUs), S::Diesel(V::Liters)) => Some(USgal2L * Gas2Die),
            (S::Gasoline(V::GallonsUs), S::KilowattHours) => Some(Gas2Kwh),
            (S::Gasoline(V::GallonsUs), S::KiloJoules) => Some(Gas2Kwh * kWh2Kj),
            (S::Gasoline(V::GallonsUs), S::BTU) => Some(Gas2Kwh * kWh2BTU),

            (S::GallonsGasolineEquivalent, S::Gasoline(V::GallonsUs)) => None,
            (S::GallonsGasolineEquivalent, S::GallonsGasolineEquivalent) => None,
            (S::GallonsGasolineEquivalent, S::Gasoline(V::GallonsUk)) => Some(USgal2UKgal),
            (S::GallonsGasolineEquivalent, S::Gasoline(V::Liters)) => Some(USgal2L),
            (S::GallonsGasolineEquivalent, S::Diesel(V::GallonsUs)) => Some(Gas2Die),
            (S::GallonsGasolineEquivalent, S::GallonsDieselEquivalent) => Some(Gas2Die),
            (S::GallonsGasolineEquivalent, S::Diesel(V::GallonsUk)) => Some(USgal2UKgal * Gas2Die),
            (S::GallonsGasolineEquivalent, S::Diesel(V::Liters)) => Some(USgal2L * Gas2Die),
            (S::GallonsGasolineEquivalent, S::KilowattHours) => Some(Gas2Kwh),
            (S::GallonsGasolineEquivalent, S::KiloJoules) => Some(Gas2Kwh * kWh2Kj),
            (S::GallonsGasolineEquivalent, S::BTU) => Some(Gas2Kwh * kWh2BTU),
            
            (S::Gasoline(V::Liters), S::Gasoline(V::GallonsUs)) => Some(L2USgal),
            (S::Gasoline(V::Liters), S::GallonsGasolineEquivalent) => Some(L2USgal),
            (S::Gasoline(V::Liters), S::Gasoline(V::GallonsUk)) => Some(L2UKgal),
            (S::Gasoline(V::Liters), S::Gasoline(V::Liters)) => None,
            (S::Gasoline(V::Liters), S::Diesel(V::GallonsUs)) => Some(L2USgal * Gas2Die),
            (S::Gasoline(V::Liters), S::GallonsDieselEquivalent) => Some(L2USgal * Gas2Die),
            (S::Gasoline(V::Liters), S::Diesel(V::GallonsUk)) => Some(L2UKgal * Gas2Die),
            (S::Gasoline(V::Liters), S::Diesel(V::Liters)) => Some(Gas2Die),
            (S::Gasoline(V::Liters), S::KilowattHours) => Some(L2USgal * Gas2Kwh),
            (S::Gasoline(V::Liters), S::KiloJoules) => Some(L2USgal * Gas2Kwh * kWh2Kj),       
            (S::Gasoline(V::Liters), S::BTU) => Some(L2USgal * Gas2Kwh * kWh2BTU),       
            
            (S::Diesel(V::GallonsUs), S::Gasoline(V::GallonsUs)) => Some(Die2Gas),
            (S::Diesel(V::GallonsUs), S::GallonsGasolineEquivalent) => Some(Die2Gas),
            (S::Diesel(V::GallonsUs), S::Gasoline(V::GallonsUk)) => Some(USgal2UKgal * Die2Gas),
            (S::Diesel(V::GallonsUs), S::Gasoline(V::Liters)) => Some(USgal2L * Die2Gas),
            (S::Diesel(V::GallonsUs), S::Diesel(V::GallonsUs)) => None,
            (S::Diesel(V::GallonsUs), S::GallonsDieselEquivalent) => None,
            (S::Diesel(V::GallonsUs), S::Diesel(V::GallonsUk)) => Some(USgal2UKgal),
            (S::Diesel(V::GallonsUs), S::Diesel(V::Liters)) => Some(USgal2L),
            (S::Diesel(V::GallonsUs), S::KilowattHours) => Some(Die2Gas * Gas2Kwh),
            (S::Diesel(V::GallonsUs), S::KiloJoules) => Some(Die2Gas * Gas2Kwh * kWh2Kj),
            (S::Diesel(V::GallonsUs), S::BTU) => Some(Die2Gas * Gas2Kwh * kWh2BTU),

            (S::GallonsDieselEquivalent, S::Gasoline(V::GallonsUs)) => Some(Die2Gas),
            (S::GallonsDieselEquivalent, S::GallonsGasolineEquivalent) => Some(Die2Gas),
            (S::GallonsDieselEquivalent, S::Gasoline(V::GallonsUk)) => Some(USgal2UKgal * Die2Gas),
            (S::GallonsDieselEquivalent, S::Gasoline(V::Liters)) => Some(USgal2L * Die2Gas),
            (S::GallonsDieselEquivalent, S::Diesel(V::GallonsUs)) => None,
            (S::GallonsDieselEquivalent, S::GallonsDieselEquivalent) => None,
            (S::GallonsDieselEquivalent, S::Diesel(V::GallonsUk)) => Some(USgal2UKgal),
            (S::GallonsDieselEquivalent, S::Diesel(V::Liters)) => Some(USgal2L),
            (S::GallonsDieselEquivalent, S::KilowattHours) => Some(Die2Gas * Gas2Kwh),
            (S::GallonsDieselEquivalent, S::KiloJoules) => Some(Die2Gas * Gas2Kwh * kWh2Kj),
            (S::GallonsDieselEquivalent, S::BTU) => Some(Die2Gas * Gas2Kwh * kWh2Kj),
            
            (S::Diesel(V::Liters), S::Gasoline(V::GallonsUs)) => Some(L2USgal * Die2Gas),
            (S::Diesel(V::Liters), S::GallonsGasolineEquivalent) => Some(L2USgal * Die2Gas),
            (S::Diesel(V::Liters), S::Gasoline(V::GallonsUk)) => Some(L2UKgal * Die2Gas),
            (S::Diesel(V::Liters), S::Gasoline(V::Liters)) => Some(Die2Gas),
            (S::Diesel(V::Liters), S::Diesel(V::GallonsUs)) => Some(L2USgal),
            (S::Diesel(V::Liters), S::GallonsDieselEquivalent) => Some(L2USgal),
            (S::Diesel(V::Liters), S::Diesel(V::GallonsUk)) => Some(L2UKgal),
            (S::Diesel(V::Liters), S::Diesel(V::Liters)) => None,
            (S::Diesel(V::Liters), S::KilowattHours) => Some(L2USgal * Die2Gas * Gas2Kwh),
            (S::Diesel(V::Liters), S::KiloJoules) => Some(L2USgal * Die2Gas * Gas2Kwh * kWh2Kj),
            (S::Diesel(V::Liters), S::BTU) => Some(L2USgal * Die2Gas * Gas2Kwh * kWh2BTU),
            
            (S::KilowattHours, S::Gasoline(V::GallonsUs)) => Some(kWh2Gas),
            (S::KilowattHours, S::GallonsGasolineEquivalent) => Some(kWh2Gas),
            (S::KilowattHours, S::Gasoline(V::GallonsUk)) => Some(kWh2Gas * USgal2UKgal),
            (S::KilowattHours, S::Gasoline(V::Liters)) => Some(kWh2Gas * USgal2L),
            (S::KilowattHours, S::Diesel(V::GallonsUs)) => Some(kWh2Gas * Gas2Die),
            (S::KilowattHours, S::GallonsDieselEquivalent) => Some(kWh2Gas * Gas2Die),
            (S::KilowattHours, S::Diesel(V::GallonsUk)) => Some(kWh2Gas * Gas2Die * USgal2UKgal),
            (S::KilowattHours, S::Diesel(V::Liters)) => Some(kWh2Gas * Gas2Die * USgal2L),
            (S::KilowattHours, S::KilowattHours) => None,
            (S::KilowattHours, S::KiloJoules) => Some(kWh2Kj),
            (S::KilowattHours, S::BTU) => Some(kWh2BTU),
            
            (S::Gasoline(V::GallonsUk), S::Gasoline(V::GallonsUs)) => Some(UKgal2USgal),
            (S::Gasoline(V::GallonsUk), S::GallonsGasolineEquivalent) => Some(UKgal2USgal),
            (S::Gasoline(V::GallonsUk), S::Gasoline(V::GallonsUk)) => None,
            (S::Gasoline(V::GallonsUk), S::Gasoline(V::Liters)) => Some(UKgal2L),
            (S::Gasoline(V::GallonsUk), S::Diesel(V::GallonsUs)) => Some(UKgal2USgal * Gas2Die),
            (S::Gasoline(V::GallonsUk), S::GallonsDieselEquivalent) => Some(UKgal2USgal * Gas2Die),
            (S::Gasoline(V::GallonsUk), S::Diesel(V::GallonsUk)) => Some(Gas2Die),
            (S::Gasoline(V::GallonsUk), S::Diesel(V::Liters)) => Some(UKgal2L * Gas2Die),
            (S::Gasoline(V::GallonsUk), S::KilowattHours) => Some(UKgal2USgal * Gas2Kwh),
            (S::Gasoline(V::GallonsUk), S::KiloJoules) => Some(UKgal2USgal * Gas2Kwh * kWh2Kj),
            (S::Gasoline(V::GallonsUk), S::BTU) => Some(UKgal2USgal * Gas2Kwh * kWh2BTU),
            
            (S::Diesel(V::GallonsUk), S::Gasoline(V::GallonsUs)) => Some(UKgal2USgal * Die2Gas),
            (S::Diesel(V::GallonsUk), S::GallonsGasolineEquivalent) => Some(UKgal2USgal * Die2Gas),
            (S::Diesel(V::GallonsUk), S::Gasoline(V::GallonsUk)) => Some(Die2Gas),
            (S::Diesel(V::GallonsUk), S::Gasoline(V::Liters)) => Some(UKgal2L * Die2Gas),
            (S::Diesel(V::GallonsUk), S::Diesel(V::GallonsUs)) => Some(UKgal2USgal),
            (S::Diesel(V::GallonsUk), S::GallonsDieselEquivalent) => Some(UKgal2USgal),
            (S::Diesel(V::GallonsUk), S::Diesel(V::GallonsUk)) => None,
            (S::Diesel(V::GallonsUk), S::Diesel(V::Liters)) => Some(UKgal2L),
            (S::Diesel(V::GallonsUk), S::KilowattHours) => Some(UKgal2USgal * Die2Gas * Gas2Kwh),
            (S::Diesel(V::GallonsUk), S::KiloJoules) => Some(UKgal2USgal * Die2Gas * Gas2Kwh * kWh2Kj),
            (S::Diesel(V::GallonsUk), S::BTU) => Some(UKgal2USgal * Die2Gas * Gas2Kwh * kWh2BTU),

            (S::KiloJoules, S::Gasoline(V::GallonsUs)) => Some(Kj2Kwh * kWh2Gas),
            (S::KiloJoules, S::GallonsGasolineEquivalent) => Some(Kj2Kwh * kWh2Gas),
            (S::KiloJoules, S::Gasoline(V::GallonsUk)) => Some(Kj2Kwh * kWh2Gas * USgal2UKgal),
            (S::KiloJoules, S::Gasoline(V::Liters)) => Some(Kj2Kwh * kWh2Gas * USgal2L),
            (S::KiloJoules, S::Diesel(V::GallonsUs)) => Some(Kj2Kwh * kWh2Gas * Gas2Die),
            (S::KiloJoules, S::GallonsDieselEquivalent) => Some(Kj2Kwh * kWh2Gas * Gas2Die),
            (S::KiloJoules, S::Diesel(V::GallonsUk)) => Some(Kj2Kwh * kWh2Gas * Gas2Die * USgal2UKgal),
            (S::KiloJoules, S::Diesel(V::Liters)) => Some(Kj2Kwh * kWh2Gas * Gas2Die * USgal2L),
            (S::KiloJoules, S::KilowattHours) => Some(Kj2Kwh),
            (S::KiloJoules, S::KiloJoules) => None,
            (S::KiloJoules, S::BTU) => Some(Kj2Kwh * kWh2BTU),

            (S::BTU, S::Gasoline(V::GallonsUs)) => Some(BTU2Kwh * kWh2Gas),
            (S::BTU, S::GallonsGasolineEquivalent) => Some(BTU2Kwh * kWh2Gas),
            (S::BTU, S::Gasoline(V::GallonsUk)) => Some(BTU2Kwh * kWh2Gas * USgal2UKgal),
            (S::BTU, S::Gasoline(V::Liters)) => Some(BTU2Kwh * kWh2Gas * USgal2L),
            (S::BTU, S::Diesel(V::GallonsUs)) => Some(BTU2Kwh * kWh2Gas * Gas2Die),
            (S::BTU, S::GallonsDieselEquivalent) => Some(BTU2Kwh * kWh2Gas * Gas2Die),
            (S::BTU, S::Diesel(V::GallonsUk)) => Some(BTU2Kwh * kWh2Gas * Gas2Die * USgal2UKgal),
            (S::BTU, S::Diesel(V::Liters)) => Some(BTU2Kwh * kWh2Gas * Gas2Die * USgal2L),
            (S::BTU, S::KilowattHours) => Some(BTU2Kwh),
            (S::BTU, S::KiloJoules) => Some(BTU2Kwh * kWh2Kj),
            (S::BTU, S::BTU) => None,
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
            "ukgallonsgasoline" => Ok(E::Gasoline(VolumeUnit::GallonsUk)),
            "ukgallonsdiesel" => Ok(E::Diesel(VolumeUnit::GallonsUk)),
            "kilowatthours" | "kilowatthour" | "kwh" => Ok(E::KilowattHours),
            "litersgasoline" => Ok(E::Gasoline(VolumeUnit::Liters)),
            "litersdiesel" => Ok(E::Diesel(VolumeUnit::Liters)),
            "gallonsgasolineequivalent" | "gge" => Ok(E::GallonsGasolineEquivalent),
            "kilojoules" | "kj" => Ok(E::KiloJoules),
            "btu" | "britishthermalunit" => Ok(E::BTU),
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
