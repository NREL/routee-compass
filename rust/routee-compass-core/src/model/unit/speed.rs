use super::{
    internal_float::InternalFloat, AsF64, Convert, Distance, DistanceUnit, SpeedUnit, Time,
    TimeUnit, UnitError,
};
use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cmp::Ordering, fmt::Display, str::FromStr};

#[derive(
    Copy,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Default,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    Neg,
    Allocative,
)]
pub struct Speed(pub InternalFloat);

impl AsF64 for Speed {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl AsF64 for &Speed {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl PartialOrd for Speed {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for Speed {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for Speed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<f64> for Speed {
    fn from(value: f64) -> Self {
        Speed(InternalFloat::new(value))
    }
}

impl FromStr for Speed {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s
            .parse::<f64>()
            .map_err(|_| format!("could not parse {} as a number", s))?;
        if value < 0.0 {
            Err(format!(
                "speed value {} invalid, must be strictly positive (0, +inf]",
                value
            ))
        } else {
            Ok(Speed::from(value))
        }
    }
}

impl Speed {
    /// calculates a speed value based on the incoming time/distance values
    /// performs the division operation to get speed in the implied speed unit
    /// (based on the input distance and time units).
    pub fn from_distance_and_time(
        distance: (&Distance, &DistanceUnit),
        time: (&Time, &TimeUnit),
    ) -> Result<(Speed, SpeedUnit), UnitError> {
        let (d, du) = distance;
        let (t, tu) = time;
        if t <= &Time::ZERO {
            Err(UnitError::SpeedFromTimeAndDistanceError(*t, *d))
        } else {
            let s = Speed::from(d.as_f64() / t.as_f64());
            let su = SpeedUnit::from((du, tu));
            Ok((s, su))
        }
    }

    pub fn to_base_unit(&self, current_speed_unit: &SpeedUnit) -> Result<(), UnitError> {
        let mut s = Cow::Borrowed(self);
        current_speed_unit.convert_to_base(&mut s)
    }
    pub const ZERO: Speed = Speed(InternalFloat::ZERO);
    pub const ONE: Speed = Speed(InternalFloat::ONE);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::unit::{baseunit, AsF64};

    fn approx_eq_speed(a: Speed, b: Speed, error: f64) {
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
    fn test_speed_calculate_fails() {
        let failure = Speed::from_distance_and_time(
            (&Distance::ONE, &DistanceUnit::Meters),
            (&Time::ZERO, &TimeUnit::Seconds),
        );
        assert!(failure.is_err());
    }

    #[test]
    fn test_speed_calculate_mps() {
        let (speed, speed_unit) = Speed::from_distance_and_time(
            (&Distance::ONE, &DistanceUnit::Meters),
            (&Time::ONE, &TimeUnit::Seconds),
        )
        .unwrap();
        assert_eq!(speed, Speed::ONE);
        assert_eq!(speed_unit, SpeedUnit::MPS);
    }

    #[test]
    fn test_speed_calculate_mph() {
        let (speed, speed_unit) = Speed::from_distance_and_time(
            (&Distance::ONE, &DistanceUnit::Miles),
            (&Time::ONE, &TimeUnit::Hours),
        )
        .unwrap();
        approx_eq_speed(Speed::ONE, speed, 0.001);
        assert_eq!(speed_unit, SpeedUnit::MPH);
    }

    #[test]
    fn test_speed_calculate_kph() {
        let (speed, speed_unit) = Speed::from_distance_and_time(
            (&Distance::ONE, &DistanceUnit::Kilometers),
            (&Time::ONE, &TimeUnit::Hours),
        )
        .unwrap();
        approx_eq_speed(speed, Speed::ONE, 0.001);
        assert_eq!(speed_unit, SpeedUnit::KPH);
    }

    #[test]
    fn test_speed_calculate_base() {
        let (speed, speed_unit) = Speed::from_distance_and_time(
            (&Distance::ONE, &baseunit::DISTANCE_UNIT),
            (&Time::ONE, &baseunit::TIME_UNIT),
        )
        .unwrap();
        approx_eq_speed(speed, Speed::ONE, 0.001);
        assert_eq!(speed_unit, baseunit::SPEED_UNIT);
    }
}
