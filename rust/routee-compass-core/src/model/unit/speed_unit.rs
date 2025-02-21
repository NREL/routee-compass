use super::{baseunit, AsF64, Convert, Distance, DistanceUnit, Speed, Time, TimeUnit, UnitError};
use crate::util::serde::serde_ops::string_deserialize;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, str::FromStr};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub struct SpeedUnit(pub DistanceUnit, pub TimeUnit);

impl std::fmt::Display for SpeedUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (du, tu) = (self.0, self.1);
        write!(f, "{}/{}", du, tu)
    }
}

impl FromStr for SpeedUnit {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        string_deserialize(s)
    }
}

impl Convert<Speed> for SpeedUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Speed>, to: &Self) -> Result<(), UnitError> {
        if value.to_f64() <= 0.0 {
            return Err(UnitError::InvalidSpeed(0.0));
        }
        let (from_du, from_tu) = (self.0, self.1);
        let (to_du, to_tu) = (to.0, to.1);
        if from_du == to_du && from_tu == to_tu {
            return Ok(());
        }

        let mut dist_convert = Cow::Owned(Distance::from(value.to_f64()));
        let mut time_convert = Cow::Owned(Time::ONE);
        from_du.convert(&mut dist_convert, &to_du)?;
        from_tu.convert(&mut time_convert, &to_tu)?;
        if dist_convert.as_ref() <= &Distance::ZERO {
            return Err(UnitError::PrecisionError(format!(
                "while converting from {} {} to {}, the distance value went to {} due to numeric precision, which is invalid.",
                value.clone(),
                self,
                to,
                dist_convert.as_f64()
            )));
        }
        if time_convert.as_ref() <= &Time::ZERO {
            return Err(UnitError::PrecisionError(format!(
                "while converting from {} {} to {}, the time value went to {} due to numeric precision, which is invalid.",
                value.clone(),
                self,
                to,
                time_convert.as_f64()
            )));
        }
        let (mut speed, _) =
            Speed::from_distance_and_time((&dist_convert, &to_du), (&time_convert, &to_tu))?;
        let mut v = value.to_mut();
        std::mem::swap(&mut v, &mut &mut speed);

        // let conversion_factor = match (self, to) {
        //     (S(D::Kilometers, T::Hours), S(D::Kilometers, T::Hours)) => None,
        //     (S(D::Kilometers, T::Hours), S(D::Miles, T::Hours)) => Some(0.621371),
        //     (S(D::Kilometers, T::Hours), S(D::Meters, T::Seconds)) => Some(0.2777777778),
        //     (S(D::Kilometers, T::Hours), S(D::Meters, T::Milliseconds)) => Some(0.2777777778),
        //     (S(D::Miles, T::Hours), S(D::Kilometers, T::Hours)) => Some(1.60934),
        //     (S(D::Miles, T::Hours), S(D::Miles, T::Hours)) => None,
        //     (S(D::Miles, T::Hours), S(D::Meters, T::Seconds)) => Some(0.44704),
        //     (S(D::Miles, T::Hours), S(D::Meters, T::Milliseconds)) => Some(0.44704),
        //     (S(D::Meters, T::Seconds), S(D::Kilometers, T::Hours)) => Some(3.6),
        //     (S(D::Meters, T::Seconds), S(D::Miles, T::Hours)) => Some(2.237),
        //     (S(D::Meters, T::Seconds), S(D::Meters, T::Seconds)) => None,
        //     (S(D::Meters, T::Seconds), S(D::Meters, T::Milliseconds)) => None,
        // };
        // if let Some(factor) = conversion_factor {
        //     let mut updated = Speed::from(value.as_ref().as_f64() * factor);
        //     let value_mut = value.to_mut();
        //     std::mem::swap(value_mut, &mut updated);
        // }
        Ok(())
    }

    fn convert_to_base(&self, value: &mut std::borrow::Cow<Speed>) -> Result<(), UnitError> {
        self.convert(value, &baseunit::SPEED_UNIT)
    }
}

impl From<(&DistanceUnit, &TimeUnit)> for SpeedUnit {
    fn from(value: (&DistanceUnit, &TimeUnit)) -> Self {
        Self(*value.0, *value.1)
    }
}

impl SpeedUnit {
    /// provides the numerator unit for some speed unit
    pub fn associated_time_unit(&self) -> TimeUnit {
        self.1
    }

    /// provides the denomenator unit for some speed unit
    pub fn associated_distance_unit(&self) -> DistanceUnit {
        self.0
    }

    /// use as a soft "max" value for certain calculations
    /// todo: should come from configuration, not hard-coded here
    pub fn max_american_highway_speed(&self) -> (Speed, SpeedUnit) {
        (
            Speed::from(75.0),
            SpeedUnit(DistanceUnit::Miles, TimeUnit::Hours),
        )
    }
}

#[cfg(test)]
mod test {

    use std::borrow::Cow;

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
    fn test_kph_kph() {
        let mut value = Cow::Owned(Speed::ONE);
        S(D::Kilometers, T::Hours)
            .convert(&mut value, &S(D::Kilometers, T::Hours))
            .unwrap();
        assert_approx_eq(value.into_owned(), Speed::ONE, 0.001);
    }
    #[test]
    fn test_kph_mph() {
        let mut value = Cow::Owned(Speed::ONE);
        S(D::Kilometers, T::Hours)
            .convert(&mut value, &S(D::Miles, T::Hours))
            .unwrap();
        assert_approx_eq(value.into_owned(), Speed::from(0.6215040398), 0.001);
    }
    #[test]
    fn test_kph_mps() {
        let mut value = Cow::Owned(Speed::ONE);
        S(D::Kilometers, T::Hours)
            .convert(&mut value, &S(D::Meters, T::Seconds))
            .unwrap();
        assert_approx_eq(value.into_owned(), Speed::from(0.277778), 0.001);
    }
    #[test]
    fn test_mph_kph() {
        let mut value = Cow::Owned(Speed::ONE);
        S(D::Miles, T::Hours)
            .convert(&mut value, &S(D::Kilometers, T::Hours))
            .unwrap();
        assert_approx_eq(value.into_owned(), Speed::from(1.60934), 0.001);
    }
    #[test]
    fn test_mph_mph() {
        let mut value = Cow::Owned(Speed::ONE);
        S(D::Miles, T::Hours)
            .convert(&mut value, &S(D::Miles, T::Hours))
            .unwrap();
        assert_approx_eq(value.into_owned(), Speed::ONE, 0.001);
    }
    #[test]
    fn test_mph_mps() {
        let mut value = Cow::Owned(Speed::ONE);
        S(D::Miles, T::Hours)
            .convert(&mut value, &S(D::Meters, T::Seconds))
            .unwrap();
        assert_approx_eq(value.into_owned(), Speed::from(0.44704), 0.001);
    }
    #[test]
    fn test_mps_kph() {
        let mut value = Cow::Owned(Speed::ONE);
        S(D::Meters, T::Seconds)
            .convert(&mut value, &S(D::Kilometers, T::Hours))
            .unwrap();
        assert_approx_eq(value.into_owned(), Speed::from(3.6), 0.001);
    }
    #[test]
    fn test_mps_mph() {
        let mut value = Cow::Owned(Speed::ONE);
        S(D::Meters, T::Seconds)
            .convert(&mut value, &S(D::Miles, T::Hours))
            .unwrap();
        assert_approx_eq(value.into_owned(), Speed::from(2.23694), 0.001);
    }
    #[test]
    fn test_mps_mps() {
        let mut value = Cow::Owned(Speed::ONE);
        S(D::Meters, T::Seconds)
            .convert(&mut value, &S(D::Meters, T::Seconds))
            .unwrap();
        assert_approx_eq(value.into_owned(), Speed::ONE, 0.001);
    }
}
