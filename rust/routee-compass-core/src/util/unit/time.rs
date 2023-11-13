use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

use super::{
    as_f64::AsF64, builders, Distance, DistanceUnit, Speed, SpeedUnit, TimeUnit, UnitError,
};

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
)]
pub struct Time(pub OrderedFloat<f64>);

impl AsF64 for Time {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl From<(Distance, Speed)> for Time {
    fn from(value: (Distance, Speed)) -> Self {
        let (distance, speed) = value;
        let time = distance.as_f64() / speed.as_f64();
        Time::new(time)
    }
}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

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
    pub fn new(value: f64) -> Time {
        Time(OrderedFloat(value))
    }
    pub fn create(
        speed: Speed,
        speed_unit: SpeedUnit,
        distance: Distance,
        distance_unit: DistanceUnit,
        time_unit: TimeUnit,
    ) -> Result<Time, UnitError> {
        builders::create_time(speed, speed_unit, distance, distance_unit, time_unit)
    }
    pub fn to_f64(&self) -> f64 {
        (self.0).0
    }
    pub const ZERO: Time = Time(OrderedFloat(0.0));
    pub const ONE: Time = Time(OrderedFloat(1.0));
}
