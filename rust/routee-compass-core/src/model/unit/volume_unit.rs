use serde::{Deserialize, Serialize};
use std::{borrow::Cow, str::FromStr};

use super::{baseunit, AsF64, Convert, UnitError, Volume};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case", try_from = "String")]
pub enum VolumeUnit {
    GallonsUs,
    GallonsUk,
    Liters,
}

impl Convert<Volume> for VolumeUnit {
    fn convert(&self, value: &mut Cow<Volume>, to: &Self) -> Result<(), UnitError> {
        use VolumeUnit as V;
        let conversion_factor: Option<f64> = match (self, to) {
            (V::Liters, V::Liters) => None,
            (V::Liters, V::GallonsUs) => Some(0.264172),
            (V::Liters, V::GallonsUk) => Some(0.219969),
            (V::GallonsUs, V::Liters) => Some(3.78541),
            (V::GallonsUs, V::GallonsUs) => None,
            (V::GallonsUs, V::GallonsUk) => Some(0.832674),
            (V::GallonsUk, V::Liters) => Some(4.54609),
            (V::GallonsUk, V::GallonsUs) => Some(1.20095),
            (V::GallonsUk, V::GallonsUk) => None,
        };
        if let Some(factor) = conversion_factor {
            let updated = Volume::from(value.as_ref().as_f64() * factor);
            *value.to_mut() = updated;
        }
        Ok(())
    }

    fn convert_to_base(&self, value: &mut Cow<Volume>) -> Result<(), UnitError> {
        self.convert(value, &baseunit::VOLUME_UNIT)
    }
}

impl std::fmt::Display for VolumeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

impl FromStr for VolumeUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use VolumeUnit as V;
        match s.trim().to_lowercase().as_str() {
            "gal" | "usgal" | "usgals" => Ok(V::GallonsUs),
            "ukgal" | "ukgals" => Ok(V::GallonsUk),
            "liter" | "liters" | "l" => Ok(V::Liters),
            _ => Err(format!("unknown volume unit '{}'", s)),
        }
    }
}

impl TryFrom<String> for VolumeUnit {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

#[cfg(test)]
mod test {

    use crate::model::unit::{VolumeUnit as V, *};
    use std::borrow::Cow;

    fn assert_approx_eq(a: Volume, b: Volume, error: f64) {
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
    fn test_l_l() {
        let mut value = Cow::Owned(Volume::ONE);
        V::Liters.convert(&mut value, &V::Liters).unwrap();
        assert_approx_eq(value.into_owned(), Volume::ONE, 0.001);
    }

    #[test]
    fn test_l_gal() {
        let mut value = Cow::Owned(Volume::ONE);
        V::Liters.convert(&mut value, &V::GallonsUs).unwrap();
        assert_approx_eq(value.into_owned(), Volume::from(0.264172), 0.001);
    }

    #[test]
    fn test_l_ukgal() {
        let mut value = Cow::Owned(Volume::ONE);
        V::Liters.convert(&mut value, &V::GallonsUk).unwrap();
        assert_approx_eq(value.into_owned(), Volume::from(0.219969), 0.001);
    }

    #[test]
    fn test_gal_l() {
        let mut value = Cow::Owned(Volume::ONE);
        V::GallonsUs.convert(&mut value, &V::Liters).unwrap();
        assert_approx_eq(value.into_owned(), Volume::from(3.78541), 0.001);
    }

    #[test]
    fn test_gal_gal() {
        let mut value = Cow::Owned(Volume::ONE);
        V::GallonsUs.convert(&mut value, &V::GallonsUs).unwrap();
        assert_approx_eq(value.into_owned(), Volume::ONE, 0.001);
    }

    #[test]
    fn test_gal_ukgal() {
        let mut value = Cow::Owned(Volume::ONE);
        V::GallonsUs.convert(&mut value, &V::GallonsUk).unwrap();
        assert_approx_eq(value.into_owned(), Volume::from(0.832674), 0.001);
    }

    #[test]
    fn test_ukgal_l() {
        let mut value = Cow::Owned(Volume::ONE);
        V::GallonsUk.convert(&mut value, &V::Liters).unwrap();
        assert_approx_eq(value.into_owned(), Volume::from(4.54609), 0.001);
    }

    #[test]
    fn test_ukgal_gal() {
        let mut value = Cow::Owned(Volume::ONE);
        V::GallonsUk.convert(&mut value, &V::GallonsUs).unwrap();
        assert_approx_eq(value.into_owned(), Volume::from(1.20095), 0.001);
    }

    #[test]
    fn test_ukgal_ukgal() {
        let mut value = Cow::Owned(Volume::ONE);
        V::GallonsUk.convert(&mut value, &V::GallonsUk).unwrap();
        assert_approx_eq(value.into_owned(), Volume::ONE, 0.001);
    }
}
