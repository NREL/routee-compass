use super::{DistanceUnit, EnergyUnit};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Copy)]
pub struct EnergyRateUnit(pub EnergyUnit, pub DistanceUnit);

impl EnergyRateUnit {
    /// energy rates are defined with respect to a distance unit
    pub fn associated_distance_unit(&self) -> DistanceUnit {
        self.1
    }

    pub fn associated_energy_unit(&self) -> EnergyUnit {
        self.0
    }
}

impl std::fmt::Display for EnergyRateUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (du, tu) = (self.0, self.1);
        write!(f, "{}/{}", du, tu)
    }
}

impl FromStr for EnergyRateUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split("/").collect_vec()[..] {
            [eu_str, du_str] => {
                let eu = EnergyUnit::from_str(eu_str).map_err(|e| {
                    format!(
                        "energy rate unit has invalid energy unit value '{}', error: {}",
                        eu_str, e
                    )
                })?;
                let du = DistanceUnit::from_str(eu_str).map_err(|e| {
                    format!(
                        "energy rate unit has invalid distance unit value '{}', error: {}",
                        du_str, e
                    )
                })?;
                Ok(EnergyRateUnit(eu, du))
            }
            _ => Err(format!(
                "expected energy rate unit in the format '<distance>/<time>', found: {}",
                s
            )),
        }
    }
}
