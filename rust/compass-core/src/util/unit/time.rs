use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

use super::{
    Distance, DistanceUnit, Speed, SpeedUnit, UnitError, BASE_DISTANCE, BASE_SPEED, BASE_TIME,
};

#[derive(
    Copy,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    PartialOrd,
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
)]
pub struct Time(pub OrderedFloat<f64>);

impl Ord for Time {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Time {
    /// calculates a time value based on the TimeUnit and incoming speed/distance values
    /// in their unit types. First converts both Speed and Distance values to the Compass
    /// base units. performs the division operation to get time and converts to the target
    /// time unit.
    pub fn calculate_time(
        speed: Speed,
        speed_unit: SpeedUnit,
        distance: Distance,
        distance_unit: DistanceUnit,
    ) -> Result<Time, UnitError> {
        let d = distance_unit.convert(distance, BASE_DISTANCE);
        let s = speed_unit.convert(speed, BASE_SPEED);
        let time_unit = speed_unit.associated_time_unit();
        if s <= Speed::ZERO {
            return Err(UnitError::TimeFromSpeedAndDistanceError(speed, distance));
        } else {
            let t = Time::new(d.to_f64() / s.to_f64());
            Ok(BASE_TIME.convert(t, time_unit))
        }
    }

    pub fn new(value: f64) -> Time {
        Time(OrderedFloat(value))
    }
    pub fn to_f64(&self) -> f64 {
        (self.0).0
    }
    pub const ZERO: Time = Time(OrderedFloat(0.0));
    pub const ONE: Time = Time(OrderedFloat(1.0));
}

#[cfg(test)]
mod test {
    use crate::util::unit::{
        Distance, DistanceUnit, Speed, SpeedUnit, Time, TimeUnit, BASE_DISTANCE, BASE_SPEED,
        BASE_TIME,
    };

    fn assert_approx_eq(a: Time, b: Time, error: f64) {
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
    fn test_calculate_fails() {
        let failure = Time::calculate_time(
            Speed::ZERO,
            SpeedUnit::KilometersPerHour,
            Distance::ONE,
            DistanceUnit::Meters,
        );
        assert!(failure.is_err());
    }

    #[test]
    fn test_calculate_idempotent() {
        let one_sec = Time::calculate_time(
            Speed::ONE,
            SpeedUnit::MetersPerSecond,
            Distance::ONE,
            DistanceUnit::Meters,
        )
        .unwrap();
        assert_eq!(Time::ONE, one_sec);
    }

    #[test]
    fn test_calculate_kph_to_base() {
        let time = Time::calculate_time(
            Speed::ONE,
            SpeedUnit::KilometersPerHour,
            Distance::ONE,
            DistanceUnit::Kilometers,
        )
        .unwrap();
        let expected = TimeUnit::Hours.convert(Time::ONE, BASE_TIME);
        assert_approx_eq(time, expected, 0.001);
    }

    #[test]
    fn test_calculate_base_to_kph() {
        let speed_kph =
            Time::calculate_time(Speed::ONE, BASE_SPEED, Distance::ONE, BASE_DISTANCE).unwrap();
        let expected = BASE_TIME.convert(Time::ONE, TimeUnit::Hours);
        assert_approx_eq(speed_kph, expected, 0.001);
    }

    #[test]
    fn test_calculate_mph_to_base() {
        let time = Time::calculate_time(
            Speed::ONE,
            SpeedUnit::MilesPerHour,
            Distance::ONE,
            DistanceUnit::Miles,
        )
        .unwrap();
        let expected = TimeUnit::Hours.convert(Time::ONE, BASE_TIME);
        assert_approx_eq(time, expected, 0.01);
    }

    #[test]
    fn test_calculate_base_to_mph() {
        let time =
            Time::calculate_time(Speed::ONE, BASE_SPEED, Distance::ONE, BASE_DISTANCE).unwrap();
        let expected = BASE_TIME.convert(Time::ONE, TimeUnit::Hours);
        assert_approx_eq(time, expected, 0.001);
    }
}
