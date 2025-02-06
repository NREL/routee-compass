use super::{baseunit, AsF64, Convert, DistanceUnit, Speed, TimeUnit};
use crate::util::serde::serde_ops::string_deserialize;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SpeedUnit {
    KilometersPerHour,
    MilesPerHour,
    MetersPerSecond,
}

impl std::fmt::Display for SpeedUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

impl FromStr for SpeedUnit {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        string_deserialize(s)
    }
}

impl Convert<Speed> for SpeedUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Speed>, to: &Self) {
        /// converts a value from the current speed unit to some target speed unit.
        use SpeedUnit as S;
        let conversion_factor = match (self, to) {
            (S::KilometersPerHour, S::KilometersPerHour) => None,
            (S::KilometersPerHour, S::MilesPerHour) => Some(0.621371),
            (S::KilometersPerHour, S::MetersPerSecond) => Some(0.2777777778),
            (S::MilesPerHour, S::KilometersPerHour) => Some(1.60934),
            (S::MilesPerHour, S::MilesPerHour) => None,
            (S::MilesPerHour, S::MetersPerSecond) => Some(0.44704),
            (S::MetersPerSecond, S::KilometersPerHour) => Some(3.6),
            (S::MetersPerSecond, S::MilesPerHour) => Some(2.237),
            (S::MetersPerSecond, S::MetersPerSecond) => None,
        };
        if let Some(factor) = conversion_factor {
            let mut updated = Speed::from(value.as_ref().as_f64() * factor);
            let value_mut = value.to_mut();
            std::mem::swap(value_mut, &mut updated);
        }
    }

    fn convert_to_base(&self, value: &mut std::borrow::Cow<Speed>) {
        self.convert(value, &baseunit::SPEED_UNIT)
    }
}

impl From<(&DistanceUnit, &TimeUnit)> for SpeedUnit {
    fn from(value: (&DistanceUnit, &TimeUnit)) -> Self {
        use DistanceUnit as D;
        use SpeedUnit as S;
        use TimeUnit as T;
        match value {
            (&D::Meters, &T::Hours) => todo!(),
            (&D::Meters, &T::Minutes) => todo!(),
            (&D::Meters, &T::Seconds) => S::MetersPerSecond,
            (&D::Meters, &T::Milliseconds) => todo!(),
            (&D::Kilometers, &T::Hours) => S::KilometersPerHour,
            (&D::Kilometers, &T::Minutes) => todo!(),
            (&D::Kilometers, &T::Seconds) => todo!(),
            (&D::Kilometers, &T::Milliseconds) => todo!(),
            (&D::Miles, &T::Hours) => S::MilesPerHour,
            (&D::Miles, &T::Minutes) => todo!(),
            (&D::Miles, &T::Seconds) => todo!(),
            (&D::Miles, &T::Milliseconds) => todo!(),
            (&D::Inches, &T::Hours) => todo!(),
            (&D::Inches, &T::Minutes) => todo!(),
            (&D::Inches, &T::Seconds) => todo!(),
            (&D::Inches, &T::Milliseconds) => todo!(),
            (&D::Feet, &T::Hours) => todo!(),
            (&D::Feet, &T::Minutes) => todo!(),
            (&D::Feet, &T::Seconds) => todo!(),
            (&D::Feet, &T::Milliseconds) => todo!(),
        }
    }
}

impl SpeedUnit {
    /// provides the numerator unit for some speed unit
    pub fn associated_time_unit(&self) -> TimeUnit {
        use SpeedUnit as S;
        match self {
            S::KilometersPerHour => TimeUnit::Hours,
            S::MilesPerHour => TimeUnit::Hours,
            S::MetersPerSecond => TimeUnit::Seconds,
        }
    }

    /// provides the denomenator unit for some speed unit
    pub fn associated_distance_unit(&self) -> DistanceUnit {
        use SpeedUnit as S;
        match self {
            S::KilometersPerHour => DistanceUnit::Kilometers,
            S::MilesPerHour => DistanceUnit::Miles,
            S::MetersPerSecond => DistanceUnit::Meters,
        }
    }

    /// use as a soft "max" value for certain calculations
    pub fn max_american_highway_speed(&self) -> Speed {
        use SpeedUnit as S;
        match self {
            S::KilometersPerHour => Speed::from(120.675),
            S::MilesPerHour => Speed::from(75.0),
            S::MetersPerSecond => Speed::from(33.528),
        }
    }
}

#[cfg(test)]
mod test {

    use super::{SpeedUnit as S, *};

    fn assert_approx_eq(a: Speed, b: Speed, error: f64) {
        let result = match (a, b) {
            (c, d) if c < d => (d - c).to_f64() < error,
            (c, d) if c > d => (c - d).to_f64() < error,
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
            S::KilometersPerHour.convert(&Speed::ONE, &S::KilometersPerHour),
            Speed::ONE,
            0.001,
        );
        assert_approx_eq(
            S::KilometersPerHour.convert(&Speed::ONE, &S::MilesPerHour),
            Speed::from(0.6215040398),
            0.001,
        );
        assert_approx_eq(
            S::KilometersPerHour.convert(&Speed::ONE, &S::MetersPerSecond),
            Speed::from(0.277778),
            0.001,
        );
        assert_approx_eq(
            S::MilesPerHour.convert(&Speed::ONE, &S::KilometersPerHour),
            Speed::from(1.60934),
            0.001,
        );
        assert_approx_eq(
            S::MilesPerHour.convert(&Speed::ONE, &S::MilesPerHour),
            Speed::ONE,
            0.001,
        );
        assert_approx_eq(
            S::MilesPerHour.convert(&Speed::ONE, &S::MetersPerSecond),
            Speed::from(0.44704),
            0.001,
        );
        assert_approx_eq(
            S::MetersPerSecond.convert(&Speed::ONE, &S::KilometersPerHour),
            Speed::from(3.6),
            0.001,
        );
        assert_approx_eq(
            S::MetersPerSecond.convert(&Speed::ONE, &S::MilesPerHour),
            Speed::from(2.23694),
            0.001,
        );
        assert_approx_eq(
            S::MetersPerSecond.convert(&Speed::ONE, &S::MetersPerSecond),
            Speed::ONE,
            0.001,
        );
    }
}
