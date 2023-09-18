use serde::{Deserialize, Serialize};

use super::{DistanceUnit, EnergyUnit};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EnergyRateUnit {
    GallonsGasolinePerMile,
    KilowattHoursPerMile,
    KilowattHoursPerKilometer,
}

impl EnergyRateUnit {
    /// energy rates are defined with respect to a distance unit
    pub fn associated_distance_unit(&self) -> DistanceUnit {
        use DistanceUnit as DU;
        use EnergyRateUnit as ERU;
        match self {
            ERU::GallonsGasolinePerMile => DU::Miles,
            ERU::KilowattHoursPerMile => DU::Miles,
            ERU::KilowattHoursPerKilometer => DU::Kilometers,
        }
    }

    pub fn associated_energy_unit(&self) -> EnergyUnit {
        use EnergyRateUnit as ERU;
        use EnergyUnit as EU;

        match self {
            ERU::GallonsGasolinePerMile => EU::GallonsGasoline,
            ERU::KilowattHoursPerMile => EU::KilowattHours,
            ERU::KilowattHoursPerKilometer => EU::KilowattHours,
        }
    }
}
