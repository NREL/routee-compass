use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uom::si::f64::Energy;

// TODO: These should be added to UOM as new si::energy units, but for now we define them here.
// https://www.eia.gov/energyexplained/units-and-calculators/energy-conversion-calculators.php
pub const GALLON_GASOLINE_TO_BTU: f64 = 120_214.0;
pub const GALLON_DIESEL_TO_BTU: f64 = 137_381.0;
pub const LITER_GASOLINE_TO_BTU: f64 = 31_757.0;
pub const LITER_DIESEL_TO_BTU: f64 = 36_292.0;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy, Hash, PartialOrd, Default)]
#[serde(rename_all = "snake_case", try_from = "String")]
pub enum EnergyUnit {
    KilowattHours,
    #[default]
    GallonsGasolineEquivalent,
    GallonsDieselEquivalent,
    LitersGasolineEquivalent,
    LitersDieselEquivalent,
}

impl EnergyUnit {
    pub fn to_uom(&self, value: f64) -> Energy {
        // TODO: This should be updated when we have gas and diesel units in UOM.
        match self {
            Self::KilowattHours => Energy::new::<uom::si::energy::kilowatt_hour>(value),
            Self::GallonsGasolineEquivalent => {
                Energy::new::<uom::si::energy::btu>(value * GALLON_GASOLINE_TO_BTU)
            }
            Self::GallonsDieselEquivalent => {
                Energy::new::<uom::si::energy::btu>(value * GALLON_DIESEL_TO_BTU)
            }
            Self::LitersGasolineEquivalent => {
                Energy::new::<uom::si::energy::btu>(value * LITER_GASOLINE_TO_BTU)
            }
            Self::LitersDieselEquivalent => {
                Energy::new::<uom::si::energy::btu>(value * LITER_DIESEL_TO_BTU)
            }
        }
    }
    pub fn from_uom(&self, value: Energy) -> f64 {
        match self {
            Self::KilowattHours => value.get::<uom::si::energy::kilowatt_hour>(),
            Self::GallonsGasolineEquivalent => {
                value.get::<uom::si::energy::btu>() / GALLON_GASOLINE_TO_BTU
            }
            Self::GallonsDieselEquivalent => {
                value.get::<uom::si::energy::btu>() / GALLON_DIESEL_TO_BTU
            }
            Self::LitersGasolineEquivalent => {
                value.get::<uom::si::energy::btu>() / LITER_GASOLINE_TO_BTU
            }
            Self::LitersDieselEquivalent => {
                value.get::<uom::si::energy::btu>() / LITER_DIESEL_TO_BTU
            }
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
            "gallonsgasoline" => Ok(E::GallonsDieselEquivalent),
            "gallonsdiesel" => Ok(E::GallonsDieselEquivalent),
            "kilowatthours" | "kilowatthour" | "kwh" => Ok(E::KilowattHours),
            "litersgasoline" => Ok(E::LitersGasolineEquivalent),
            "litersdiesel" => Ok(E::LitersDieselEquivalent),
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
