use super::{DistanceUnit, EnergyUnit};
use crate::util::serde_ops::string_deserialize;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum EnergyRateUnit {
    GallonsGasolinePerMile,
    GallonsDieselPerMile,
    KilowattHoursPerMile,
    KilowattHoursPerKilometer,
    KilowattHoursPerMeter,
}

impl EnergyRateUnit {
    /// energy rates are defined with respect to a distance unit
    pub fn associated_distance_unit(&self) -> DistanceUnit {
        use DistanceUnit as DU;
        use EnergyRateUnit as ERU;
        match self {
            ERU::GallonsGasolinePerMile => DU::Miles,
            ERU::GallonsDieselPerMile => DU::Miles,
            ERU::KilowattHoursPerMile => DU::Miles,
            ERU::KilowattHoursPerKilometer => DU::Kilometers,
            ERU::KilowattHoursPerMeter => DU::Meters,
        }
    }

    pub fn associated_energy_unit(&self) -> EnergyUnit {
        use EnergyRateUnit as ERU;
        use EnergyUnit as EU;

        match self {
            ERU::GallonsGasolinePerMile => EU::GallonsGasoline,
            ERU::GallonsDieselPerMile => EU::GallonsDiesel,
            ERU::KilowattHoursPerMile => EU::KilowattHours,
            ERU::KilowattHoursPerKilometer => EU::KilowattHours,
            ERU::KilowattHoursPerMeter => EU::KilowattHours,
        }
    }
}

impl std::fmt::Display for EnergyRateUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

impl FromStr for EnergyRateUnit {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        string_deserialize(s)
    }
}
