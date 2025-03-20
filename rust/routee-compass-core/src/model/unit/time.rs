use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cmp::Ordering, fmt::Display};

use crate::model::state::StateVariable;

use super::{
    internal_float::InternalFloat, AsF64, Convert, Distance, DistanceUnit, Speed, SpeedUnit,
    TimeUnit, UnitError,
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
    Allocative,
)]
pub struct Time(pub InternalFloat);

impl AsF64 for Time {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

// impl From<(Distance, Speed)> for Time {
//     fn from(value: (Distance, Speed)) -> Self {
//         let (distance, speed) = value;
//         let time = distance.as_f64() / speed.as_f64();
//         Time::from(time)
//     }
// }
impl From<StateVariable> for Time {
    fn from(value: StateVariable) -> Self {
        Time::from(value.0)
    }
}
impl From<&StateVariable> for Time {
    fn from(value: &StateVariable) -> Self {
        Time::from(value.0)
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
        write!(f, "{:?}", self.0)
    }
}

impl From<f64> for Time {
    fn from(value: f64) -> Time {
        Time(InternalFloat::new(value))
    }
}

impl Time {
    /// calculates a time value based on the incoming speed/distance values
    /// in their unit types. performs the division operation to get time in the
    /// time unit associated with the SpeedUnit.
    pub fn create(
        distance: (&Distance, &DistanceUnit),
        speed: (&Speed, &SpeedUnit),
    ) -> Result<(Time, TimeUnit), UnitError> {
        let (d, du) = distance;
        let (s, su) = speed;
        let mut d_su = Cow::Borrowed(d);
        du.convert(&mut d_su, &su.associated_distance_unit())?;

        if s <= &Speed::ZERO || d <= &Distance::ZERO {
            Err(UnitError::TimeFromSpeedAndDistanceError(*s, *su, *d, *du))
        } else {
            let time = Time::from(d_su.as_ref().as_f64() / s.as_f64());
            Ok((time, su.associated_time_unit()))
        }
    }
    // pub fn to_f64(&self) -> f64 {
    //     (self.0).0
    // }
    pub const ZERO: Time = Time(InternalFloat::ZERO);
    pub const ONE: Time = Time(InternalFloat::ONE);
}

#[cfg(test)]
mod test {

    use crate::model::unit::*;

    fn assert_approx_eq(a: Time, b: Time, error: f64) {
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
    fn fails_with_invalid_speed() {
        let failure = Time::create(
            (&Distance::ONE, &DistanceUnit::Meters),
            (&Speed::ZERO, &SpeedUnit::KPH),
        );
        assert!(failure.is_err());
    }

    #[test]
    fn one_times_one() {
        let (one_sec, tu) = Time::create(
            (&Distance::ONE, &DistanceUnit::Meters),
            (&Speed::ONE, &SpeedUnit::MPS),
        )
        .unwrap();
        assert_eq!(Time::ONE, one_sec);
        assert_eq!(tu, TimeUnit::Seconds);
    }

    #[test]
    fn sixty_miles_at_60mph() {
        let (t, tu) = Time::create(
            (&Distance::from(60.0), &DistanceUnit::Miles),
            (&Speed::from(60.0), &SpeedUnit::MPH),
        )
        .unwrap();
        assert_approx_eq(t, Time::ONE, 0.0001);
        assert_eq!(tu, TimeUnit::Hours);
    }

    #[test]
    fn walking_100_meters() {
        let (walk_speed, walk_unit) = (Speed::from(5.0), SpeedUnit::KPH);
        let (t, tu) = Time::create(
            (&Distance::from(100.0), &DistanceUnit::Meters),
            (&walk_speed, &walk_unit),
        )
        .unwrap();
        let expected = Time::from(0.1 / 5.0); // should convert to km internally
        assert_approx_eq(t, expected, 0.0001);
        assert_eq!(tu, TimeUnit::Hours);
    }
}
