use crate::model::unit::Energy;

use super::{baseunit, AsF64, Convert, Distance, DistanceUnit, EnergyRate, EnergyUnit, UnitError};
use itertools::Itertools;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::{borrow::Cow, str::FromStr};

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum EnergyRateUnit {
    DistancePerEnergy(DistanceUnit, EnergyUnit),
    EnergyPerDistance(EnergyUnit, DistanceUnit),
}

impl EnergyRateUnit {
    pub const GGPM: EnergyRateUnit =
        EnergyRateUnit::EnergyPerDistance(EnergyUnit::GallonsGasoline, DistanceUnit::Miles);
    pub const GDPM: EnergyRateUnit =
        EnergyRateUnit::EnergyPerDistance(EnergyUnit::GallonsDiesel, DistanceUnit::Miles);
    pub const KWHPM: EnergyRateUnit =
        EnergyRateUnit::EnergyPerDistance(EnergyUnit::KilowattHours, DistanceUnit::Miles);
    pub const KWHPKM: EnergyRateUnit =
        EnergyRateUnit::EnergyPerDistance(EnergyUnit::KilowattHours, DistanceUnit::Kilometers);

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

    pub fn distance_numerator(&self) -> bool {
        match self {
            EnergyRateUnit::DistancePerEnergy(_, _) => true,
            EnergyRateUnit::EnergyPerDistance(_, _) => false,
        }
    }

    pub fn energy_rate_numerator(&self) -> bool {
        match self {
            EnergyRateUnit::DistancePerEnergy(_, _) => false,
            EnergyRateUnit::EnergyPerDistance(_, _) => true,
        }
    }

    /// true if this and the other EnergyRateUnit are both either distance/energy or energy/distance format
    pub fn matches_format(&self, other: &EnergyRateUnit) -> bool {
        match (self, other) {
            (EnergyRateUnit::DistancePerEnergy(_, _), EnergyRateUnit::DistancePerEnergy(_, _)) => {
                true
            }
            (EnergyRateUnit::DistancePerEnergy(_, _), EnergyRateUnit::EnergyPerDistance(_, _)) => {
                false
            }
            (EnergyRateUnit::EnergyPerDistance(_, _), EnergyRateUnit::DistancePerEnergy(_, _)) => {
                false
            }
            (EnergyRateUnit::EnergyPerDistance(_, _), EnergyRateUnit::EnergyPerDistance(_, _)) => {
                true
            }
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
        let (from_du, from_eu) = (
            self.associated_distance_unit(),
            self.associated_energy_unit(),
        );
        let (to_du, to_eu) = (to.associated_distance_unit(), to.associated_energy_unit());
        let matches_format = self.matches_format(to);
        if matches_format && from_du == to_du && from_eu == to_eu {
            return Ok(());
        }

        let mut e = if self.energy_rate_numerator() {
            Cow::Owned(Energy::from(value.as_f64()))
        } else {
            Cow::Owned(Energy::ONE)
        };
        let mut d = if self.distance_numerator() {
            Cow::Owned(Distance::from(value.as_f64()))
        } else {
            Cow::Owned(Distance::ONE)
        };
        from_du.convert(&mut d, &to_du)?;
        from_eu.convert(&mut e, &to_eu)?;

        let (energy_rate, _) = EnergyRate::from_energy_and_distance(
            (&e, &to_eu),
            (&d, &to_du),
            to.energy_rate_numerator(),
        );

        *value.to_mut() = energy_rate;

        Ok(())
    }

    fn convert_to_base(&self, value: &mut Cow<EnergyRate>) -> Result<(), UnitError> {
        self.convert(value, &baseunit::ENERGY_RATE_UNIT)
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
    use std::{borrow::Cow, str::FromStr};

    use super::EnergyRateUnit as ERU;
    use crate::model::unit::{AsF64, Convert, DistanceUnit as DU, EnergyRate, EnergyUnit as EU};
    use serde_json::{self as sj, json};

    fn assert_approx_eq(a: EnergyRate, b: EnergyRate, error: f64) {
        let result = match (a, b) {
            (c, d) if c < d => (d - c).as_f64() < error,
            (c, d) if c > d => (c - d).as_f64() < error,
            (_, _) => true,
        };
        assert!(
            result,
            "{} ~= {} is not true within an error of {}",
            a, b, error
        )
    }

    #[test]
    fn test_mpg_from_str() {
        let result = ERU::from_str("gallons gasoline/mile");
        let expected = ERU::EnergyPerDistance(EU::GallonsGasoline, DU::Miles);
        assert_eq!(result, Ok(expected))
    }

    #[test]
    fn test_gpm_from_json() {
        let result: Result<ERU, String> =
            sj::from_value(json!("gallons gasoline/mile")).map_err(|e| e.to_string());
        let expected = ERU::EnergyPerDistance(EU::GallonsGasoline, DU::Miles);
        assert_eq!(result, Ok(expected))
    }

    #[test]
    fn test_mpg_from_json() {
        let result: Result<ERU, String> =
            sj::from_value(json!("miles/gallons gasoline")).map_err(|e| e.to_string());
        let expected = ERU::DistancePerEnergy(DU::Miles, EU::GallonsGasoline);
        assert_eq!(result, Ok(expected))
    }

    #[test]
    fn test_convert_mpg_gpm() {
        let mut energy_rate = Cow::Owned(EnergyRate::from(35.0));
        let mpg = ERU::DistancePerEnergy(DU::Miles, EU::GallonsGasoline);
        let gpm = ERU::EnergyPerDistance(EU::GallonsGasoline, DU::Miles);
        mpg.convert(&mut energy_rate, &gpm).unwrap();
        assert_approx_eq(
            energy_rate.into_owned(),
            EnergyRate::from(1.0 / 35.0),
            0.001,
        );
    }

    #[test]
    fn test_convert_mpg_kpl() {
        let mut energy_rate = Cow::Owned(EnergyRate::from(10.0));
        let mpg = ERU::DistancePerEnergy(DU::Miles, EU::GallonsGasoline);
        let kpl = ERU::DistancePerEnergy(DU::Kilometers, EU::LitersGasoline);
        mpg.convert(&mut energy_rate, &kpl).unwrap();
        assert_approx_eq(energy_rate.into_owned(), EnergyRate::from(4.25144), 0.001);
    }

    #[test]
    fn test_convert_lp100km_mpg() {
        let mut energy_rate = Cow::Owned(EnergyRate::from(1.0));
        let lp100k = ERU::EnergyPerDistance(EU::LitersDiesel, DU::Kilometers);
        let mpg = ERU::DistancePerEnergy(DU::Miles, EU::GallonsDiesel);
        lp100k.convert(&mut energy_rate, &mpg).unwrap();
        let lp1km = energy_rate.into_owned();
        let lp100km = EnergyRate::from(lp1km.as_f64() * 100.0);
        assert_approx_eq(lp100km, EnergyRate::from(235.214583), 0.1);
    }
}
