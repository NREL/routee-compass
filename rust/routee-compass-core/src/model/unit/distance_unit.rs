use super::{baseunit, Convert, Distance, UnitError};
use crate::model::unit::AsF64;
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
    fn convert(&self, value: &mut Cow<Distance>, to: &Self) -> Result<(), UnitError> {
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
            let updated = Distance::from(value.as_ref().as_f64() * factor);
            *value.to_mut() = updated;
        }
        Ok(())
    }

    fn convert_to_base(&self, value: &mut Cow<Distance>) -> Result<(), UnitError> {
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
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DistanceUnit as D;
        match s.trim().to_lowercase().as_str() {
            "meters" | "meter" => Ok(D::Meters),
            "km" | "kilometers" | "kilometer" => Ok(D::Kilometers),
            "miles" | "mile" => Ok(D::Miles),
            "inches" | "inch" | "in" => Ok(D::Inches),
            "feet" | "ft" => Ok(D::Feet),
            _ => Err(format!("unknown distance unit '{}'", s)),
        }
    }
}

#[cfg(test)]
mod test {

    use crate::model::unit::{DistanceUnit as D, *};
    use std::borrow::Cow;

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
    fn test_m_m() {
        let mut value = Cow::Owned(Distance::ONE);
        D::Meters.convert(&mut value, &D::Meters).unwrap();
        assert_approx_eq(value.into_owned(), Distance::ONE, 0.001);
    }
    #[test]
    fn test_m_km() {
        let mut value = Cow::Owned(Distance::ONE);
        D::Meters.convert(&mut value, &D::Kilometers).unwrap();
        assert_approx_eq(value.into_owned(), Distance::from(0.001), 0.001);
    }
    #[test]
    fn test_m_mi() {
        let mut value = Cow::Owned(Distance::ONE);
        D::Meters.convert(&mut value, &D::Miles).unwrap();
        assert_approx_eq(value.into_owned(), Distance::from(0.000621371), 0.001);
    }
    #[test]
    fn test_km_m() {
        let mut value = Cow::Owned(Distance::ONE);
        D::Kilometers.convert(&mut value, &D::Meters).unwrap();
        assert_approx_eq(value.into_owned(), Distance::from(1000.0), 0.001);
    }
    #[test]
    fn test_km_km() {
        let mut value = Cow::Owned(Distance::ONE);
        D::Kilometers.convert(&mut value, &D::Kilometers).unwrap();
        assert_approx_eq(value.into_owned(), Distance::ONE, 0.001);
    }
    #[test]
    fn test_km_mi() {
        let mut value = Cow::Owned(Distance::ONE);
        D::Kilometers.convert(&mut value, &D::Miles).unwrap();
        assert_approx_eq(value.into_owned(), Distance::from(0.621371), 0.001);
    }
    #[test]
    fn test_mi_m() {
        let mut value = Cow::Owned(Distance::ONE);
        D::Miles.convert(&mut value, &D::Meters).unwrap();
        assert_approx_eq(value.into_owned(), Distance::from(1609.34), 0.001);
    }
    #[test]
    fn test_mi_km() {
        let mut value = Cow::Owned(Distance::ONE);
        D::Miles.convert(&mut value, &D::Kilometers).unwrap();
        assert_approx_eq(value.into_owned(), Distance::from(1.60934), 0.001);
    }
    #[test]
    fn test_mi_mi() {
        let mut value = Cow::Owned(Distance::ONE);
        D::Miles.convert(&mut value, &D::Miles).unwrap();
        assert_approx_eq(value.into_owned(), Distance::ONE, 0.001);
    }
}
