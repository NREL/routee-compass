use super::Speed;
use super::{DistanceUnit, TimeUnit};
use serde::{Deserialize, Serialize};

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

impl From<(DistanceUnit, TimeUnit)> for SpeedUnit {
    fn from(value: (DistanceUnit, TimeUnit)) -> Self {
        use DistanceUnit as D;
        use SpeedUnit as S;
        use TimeUnit as T;
        match value {
            (D::Meters, T::Hours) => todo!(),
            (D::Meters, T::Minutes) => todo!(),
            (D::Meters, T::Seconds) => S::MetersPerSecond,
            (D::Meters, T::Milliseconds) => todo!(),
            (D::Kilometers, T::Hours) => S::KilometersPerHour,
            (D::Kilometers, T::Minutes) => todo!(),
            (D::Kilometers, T::Seconds) => todo!(),
            (D::Kilometers, T::Milliseconds) => todo!(),
            (D::Miles, T::Hours) => S::MilesPerHour,
            (D::Miles, T::Minutes) => todo!(),
            (D::Miles, T::Seconds) => todo!(),
            (D::Miles, T::Milliseconds) => todo!(),
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

    /// converts a value from the current speed unit to some target speed unit.
    pub fn convert(&self, value: Speed, target: SpeedUnit) -> Speed {
        use SpeedUnit as S;
        match (self, target) {
            (S::KilometersPerHour, S::KilometersPerHour) => value,
            (S::KilometersPerHour, S::MilesPerHour) => value * 0.621371,
            (S::KilometersPerHour, S::MetersPerSecond) => value * 0.2777777778,
            (S::MilesPerHour, S::KilometersPerHour) => value * 1.60934,
            (S::MilesPerHour, S::MilesPerHour) => value,
            (S::MilesPerHour, S::MetersPerSecond) => value * 0.44704,
            (S::MetersPerSecond, S::KilometersPerHour) => value * 3.6,
            (S::MetersPerSecond, S::MilesPerHour) => value * 2.237,
            (S::MetersPerSecond, S::MetersPerSecond) => value,
        }
    }

    /// use as a soft "max" value for certain calculations
    pub fn max_american_highway_speed(&self) -> Speed {
        use SpeedUnit as S;
        match self {
            S::KilometersPerHour => Speed::new(120.675),
            S::MilesPerHour => Speed::new(75.0),
            S::MetersPerSecond => Speed::new(33.528),
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
            S::KilometersPerHour.convert(Speed::ONE, S::KilometersPerHour),
            Speed::ONE,
            0.001,
        );
        assert_approx_eq(
            S::KilometersPerHour.convert(Speed::ONE, S::MilesPerHour),
            Speed::new(0.6215040398),
            0.001,
        );
        assert_approx_eq(
            S::KilometersPerHour.convert(Speed::ONE, S::MetersPerSecond),
            Speed::new(0.277778),
            0.001,
        );
        assert_approx_eq(
            S::MilesPerHour.convert(Speed::ONE, S::KilometersPerHour),
            Speed::new(1.60934),
            0.001,
        );
        assert_approx_eq(
            S::MilesPerHour.convert(Speed::ONE, S::MilesPerHour),
            Speed::ONE,
            0.001,
        );
        assert_approx_eq(
            S::MilesPerHour.convert(Speed::ONE, S::MetersPerSecond),
            Speed::new(0.44704),
            0.001,
        );
        assert_approx_eq(
            S::MetersPerSecond.convert(Speed::ONE, S::KilometersPerHour),
            Speed::new(3.6),
            0.001,
        );
        assert_approx_eq(
            S::MetersPerSecond.convert(Speed::ONE, S::MilesPerHour),
            Speed::new(2.23694),
            0.001,
        );
        assert_approx_eq(
            S::MetersPerSecond.convert(Speed::ONE, S::MetersPerSecond),
            Speed::ONE,
            0.001,
        );
    }
}
