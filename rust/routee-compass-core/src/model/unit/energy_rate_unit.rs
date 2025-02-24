use super::{Convert, DistanceUnit, EnergyRate, EnergyUnit, UnitError};
use itertools::Itertools;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::{borrow::Cow, str::FromStr};

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum EnergyRateUnit {
    DistancePerEnergy(DistanceUnit, EnergyUnit),
    EnergyPerDistance(EnergyUnit, DistanceUnit),
}

impl EnergyRateUnit {
    /// energy rates are defined with respect to a distance unit
    pub fn associated_distance_unit(&self) -> DistanceUnit {
        match self {
            EnergyRateUnit::DistancePerEnergy(distance_unit, _) => *distance_unit,
            EnergyRateUnit::EnergyPerDistance(_, distance_unit) => *distance_unit,
        }
    }

    pub fn associated_energy_unit(&self) -> EnergyUnit {
        match self {
            EnergyRateUnit::DistancePerEnergy(_, energy_unit) => *energy_unit,
            EnergyRateUnit::EnergyPerDistance(energy_unit, _) => *energy_unit,
        }
    }
}

impl std::fmt::Display for EnergyRateUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnergyRateUnit::DistancePerEnergy(distance_unit, energy_unit) => {
                write!(f, "{}/{}", distance_unit, energy_unit)
            }
            EnergyRateUnit::EnergyPerDistance(energy_unit, distance_unit) => {
                write!(f, "{}/{}", energy_unit, distance_unit)
            }
        }
    }
}

impl FromStr for EnergyRateUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split("/").collect_vec()[..] {
            ["mpg"] => Ok(EnergyRateUnit::DistancePerEnergy(
                DistanceUnit::Miles,
                EnergyUnit::GallonsGasoline,
            )),
            [s1, s2] => try_deserialize_unknown_unit_order(s1, s2),
            _ => Err(format!(
                "expected energy rate unit in the format '<energy>/<distance>', found: {}",
                s
            )),
        }
    }
}

impl Convert<EnergyRate> for EnergyRateUnit {
    fn convert(&self, value: &mut Cow<EnergyRate>, to: &Self) -> Result<(), UnitError> {
        todo!()
    }

    fn convert_to_base(&self, value: &mut Cow<EnergyRate>) -> Result<(), UnitError> {
        todo!()
    }
}

/// helper to attempt deserialization when unit ordering is unknown for energy rate type
fn try_deserialize_unknown_unit_order(s1: &str, s2: &str) -> Result<EnergyRateUnit, String> {
    let ed_result = match (EnergyUnit::from_str(s1), DistanceUnit::from_str(s2)) {
        (Ok(eu), Ok(du)) => Ok(EnergyRateUnit::EnergyPerDistance(eu, du)),
        (Ok(_), Err(e2)) => Err(format!("assuming 'e/d' format, unable to decode distance unit {} due to: {}", s2, e2)),
        (Err(e1), Ok(_)) => Err(format!("assuming 'e/d' format, unable to decode energy unit {} due to: {}", s1, e1)),
        (Err(e1), Err(e2)) => Err(format!("assuming 'e/d' format, unable to decode energy unit {} and distance unit {} due to: {}, {}", s1,s2,e1,e2)),
    };
    match ed_result {
        Ok(eru) => Ok(eru),
        Err(ed_err) => {
            let de_result = match (DistanceUnit::from_str(s1), EnergyUnit::from_str(s2)) {
                (Ok(du), Ok(eu)) => Ok(EnergyRateUnit::DistancePerEnergy(du, eu)),
                (Ok(_), Err(e2)) => Err(format!("assuming 'd/e' format, unable to decode energy unit {} due to: {}", s2, e2)),
                (Err(e1), Ok(_)) => Err(format!("assuming 'd/e' format, unable to decode distance unit {} due to: {}", s1, e1)),
                (Err(e1), Err(e2)) => Err(format!("assuming 'd/e' format, unable to decode distance unit {} and energy unit {} due to: {}, {}", s1,s2,e1,e2)),
            };
            de_result.map_err(|de_err| {
                format!("failed to decode energy rate unit. {}. {}.", de_err, ed_err)
            })
        }
    }
}

struct StrVisitor;

impl Visitor<'_> for StrVisitor {
    type Value = EnergyRateUnit;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string value in the form '<energy_unit>/<distance_unit>'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let result = FromStr::from_str(v);
        result.map_err(|e| {
            serde::de::Error::custom(format!(
                "while attempting to deserialize value '{}', had the following error: {}",
                v, e
            ))
        })
    }
}

impl<'de> Deserialize<'de> for EnergyRateUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(StrVisitor)
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::EnergyRateUnit;
    use crate::model::unit::{DistanceUnit, EnergyUnit};
    use serde_json::{self as sj, json};

    #[test]
    fn test_mpg_from_str() {
        let result = EnergyRateUnit::from_str("gallons gasoline/mile");
        let expected =
            EnergyRateUnit::EnergyPerDistance(EnergyUnit::GallonsGasoline, DistanceUnit::Miles);
        assert_eq!(result, Ok(expected))
    }

    #[test]
    fn test_gpm_from_json() {
        let result: Result<EnergyRateUnit, String> =
            sj::from_value(json!("gallons gasoline/mile")).map_err(|e| e.to_string());
        let expected =
            EnergyRateUnit::EnergyPerDistance(EnergyUnit::GallonsGasoline, DistanceUnit::Miles);
        assert_eq!(result, Ok(expected))
    }

    #[test]
    fn test_mpg_from_json() {
        let result: Result<EnergyRateUnit, String> =
            sj::from_value(json!("miles/gallons gasoline")).map_err(|e| e.to_string());
        let expected =
            EnergyRateUnit::DistancePerEnergy(DistanceUnit::Miles, EnergyUnit::GallonsGasoline);
        assert_eq!(result, Ok(expected))
    }
}
