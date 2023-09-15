use super::{Distance, Speed, Time};

#[derive(thiserror::Error, Debug)]
pub enum UnitError {
    #[error("cannot create speed from time {0} and distance {0}")]
    SpeedFromTimeAndDistanceError(Time, Distance),
    #[error("cannot create time from speed {0} and distance {0}")]
    TimeFromSpeedAndDistanceError(Speed, Distance),
}
