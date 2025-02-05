use super::{baseunit, Convert, Distance};
use crate::{model::unit::AsF64, util::serde::serde_ops::string_deserialize};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, str::FromStr};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DistanceUnit {
    Meters,
    Kilometers,
    Miles,
    Inches,
    Feet,
}

impl Convert<Distance> for DistanceUnit {
    fn convert(&self, value: &mut Cow<Distance>, to: &Self) {
        use DistanceUnit as S;
        let conversion_factor: Option<f64> = match (self, to) {
            (S::Meters, S::Meters) => None,
            (S::Meters, S::Kilometers) => Some(0.001),
            (S::Meters, S::Miles) => Some(0.0006215040398),
            (S::Meters, S::Inches) => Some(39.3701),
            (S::Meters, S::Feet) => Some(3.28084),
            (S::Kilometers, S::Meters) => Some(1000.0),
            (S::Kilometers, S::Kilometers) => None,
            (S::Kilometers, S::Miles) => Some(0.6215040398),
            (S::Kilometers, S::Inches) => Some(39370.1),
            (S::Kilometers, S::Feet) => Some(3280.84),
            (S::Miles, S::Meters) => Some(1609.34),
            (S::Miles, S::Kilometers) => Some(1.60934),
            (S::Miles, S::Miles) => None,
            (S::Miles, S::Inches) => Some(63360.0),
            (S::Miles, S::Feet) => Some(5280.0),
            (S::Inches, S::Meters) => Some(0.0254),
            (S::Inches, S::Kilometers) => Some(0.0000254),
            (S::Inches, S::Miles) => Some(0.0000157828),
            (S::Inches, S::Inches) => None,
            (S::Inches, S::Feet) => Some(0.0833333),
            (S::Feet, S::Meters) => Some(0.3048),
            (S::Feet, S::Kilometers) => Some(0.0003048),
            (S::Feet, S::Miles) => Some(0.000189394),
            (S::Feet, S::Inches) => Some(12.0),
            (S::Feet, S::Feet) => None,
        };
        if let Some(factor) = conversion_factor {
            let mut updated = Distance::from(value.as_ref().as_f64() * factor);
            let value_mut = value.to_mut();
            std::mem::swap(value_mut, &mut updated);
        }
    }

    fn convert_to_base(&self, value: &mut Cow<Distance>) {
        self.convert(value, &baseunit::DISTANCE_UNIT)
    }
}

impl std::fmt::Display for DistanceUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

impl FromStr for DistanceUnit {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        string_deserialize(s)
    }
}

#[cfg(test)]
mod test {

    use crate::model::unit::AsF64;

    use super::Distance;
    use super::DistanceUnit as D;

    fn assert_approx_eq(a: Distance, b: Distance, error: f64) {
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
    fn test_conversions() {
        assert_approx_eq(
            D::Meters.convert(&Distance::ONE, &D::Meters),
            Distance::ONE,
            0.001,
        );
        assert_approx_eq(
            D::Meters.convert(&Distance::ONE, &D::Kilometers),
            Distance::new(0.001),
            0.001,
        );
        assert_approx_eq(
            D::Meters.convert(&Distance::ONE, &D::Miles),
            Distance::new(0.000621371),
            0.001,
        );
        assert_approx_eq(
            D::Kilometers.convert(&Distance::ONE, &D::Meters),
            Distance::new(1000.0),
            0.001,
        );
        assert_approx_eq(
            D::Kilometers.convert(&Distance::ONE, &D::Kilometers),
            Distance::ONE,
            0.001,
        );
        assert_approx_eq(
            D::Kilometers.convert(&Distance::ONE, &D::Miles),
            Distance::new(0.621371),
            0.001,
        );
        assert_approx_eq(
            D::Miles.convert(&Distance::ONE, &D::Meters),
            Distance::new(1609.34),
            0.001,
        );
        assert_approx_eq(
            D::Miles.convert(&Distance::ONE, &D::Kilometers),
            Distance::new(1.60934),
            0.001,
        );
        assert_approx_eq(
            D::Miles.convert(&Distance::ONE, &D::Miles),
            Distance::ONE,
            0.001,
        );
    }
}
