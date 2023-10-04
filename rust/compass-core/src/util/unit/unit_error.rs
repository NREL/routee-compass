use super::{Distance, DistanceUnit, Speed, SpeedUnit, Time};

#[derive(thiserror::Error, Debug)]
pub enum UnitError {
    #[error("unable to parse {0} as a number")]
    NumericParsingError(String),
    #[error("{0} is an invalid speed, must be strictly positive (0, +inf]")]
    InvalidSpeed(f64),
    #[error("cannot create speed from time {0} and distance {0}")]
    SpeedFromTimeAndDistanceError(Time, Distance),
    #[error("cannot create time from speed {0} {1} and distance {2} {3}")]
    TimeFromSpeedAndDistanceError(Speed, SpeedUnit, Distance, DistanceUnit),
}
