use super::{
    as_f64::AsF64, builders, Distance, DistanceUnit, SpeedUnit, Time, TimeUnit, UnitError,
};
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display, str::FromStr};

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
pub struct Speed(pub OrderedFloat<f64>);

impl AsF64 for Speed {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl From<(Distance, Time)> for Speed {
    fn from(value: (Distance, Time)) -> Self {
        let (distance, time) = value;
        let speed = distance.as_f64() / time.as_f64();
        Speed::new(speed)
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
        write!(f, "{}", self.0)
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
            Ok(Speed::new(value))
        }
    }
}

impl Speed {
    pub fn new(value: f64) -> Speed {
        Speed(OrderedFloat(value))
    }
    pub fn create(
        time: Time,
        time_unit: TimeUnit,
        distance: Distance,
        distance_unit: DistanceUnit,
        speed_unit: SpeedUnit,
    ) -> Result<Speed, UnitError> {
        builders::create_speed(time, time_unit, distance, distance_unit, speed_unit)
    }
    pub fn to_f64(&self) -> f64 {
        (self.0).0
    }
    pub const ZERO: Speed = Speed(OrderedFloat(0.0));
    pub const ONE: Speed = Speed(OrderedFloat(1.0));
}
