// imports all f64 SI units at the module level
// like this:
// use compass_core::model::units::Velocity;
use serde::Deserialize;
pub use uom::si::f64::*;

#[derive(Debug, Deserialize, Clone)]
pub enum TimeUnit {
    Seconds,
    Milliseconds,
}
