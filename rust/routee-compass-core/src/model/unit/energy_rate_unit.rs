use super::{DistanceUnit, EnergyUnit};
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct EnergyRateUnit(pub DistanceUnit, pub EnergyUnit);

impl EnergyRateUnit {
    /// energy rates are defined with respect to a distance unit
    pub fn associated_distance_unit(&self) -> DistanceUnit {
        self.0
    }

    pub fn associated_energy_unit(&self) -> EnergyUnit {
        self.1
    }
}

impl std::fmt::Display for EnergyRateUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (du, eu) = (self.0, self.1);
        write!(f, "{}/{}", du, eu)
    }
}

impl FromStr for EnergyRateUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split("/").collect_vec()[..] {
            [du_str, eu_str] => {
                let du = DistanceUnit::from_str(eu_str).map_err(|e| {
                    format!(
                        "energy rate unit has invalid distance unit value '{}', error: {}",
                        du_str, e
                    )
                })?;
                let eu = EnergyUnit::from_str(eu_str).map_err(|e| {
                    format!(
                        "energy rate unit has invalid energy unit value '{}', error: {}",
                        eu_str, e
                    )
                })?;
                Ok(EnergyRateUnit(du, eu))
            }
            _ => Err(format!(
                "expected energy rate unit in the format '<distance>/<energy>', found: {}",
                s
            )),
        }
    }
}

impl<'de> Deserialize<'de> for EnergyRateUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for EnergyRateUnit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}
