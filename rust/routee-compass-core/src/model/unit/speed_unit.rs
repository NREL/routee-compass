use super::{baseunit, AsF64, Convert, Distance, DistanceUnit, Speed, Time, TimeUnit, UnitError};
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize};
use std::{borrow::Cow, str::FromStr};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpeedUnit(pub DistanceUnit, pub TimeUnit);

impl std::fmt::Display for SpeedUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (du, tu) = (self.0, self.1);
        write!(f, "{}/{}", du, tu)
    }
}

impl SpeedUnit {
    pub const KPH: SpeedUnit = SpeedUnit(DistanceUnit::Kilometers, TimeUnit::Hours);
    pub const MPH: SpeedUnit = SpeedUnit(DistanceUnit::Miles, TimeUnit::Hours);
    pub const MPS: SpeedUnit = SpeedUnit(DistanceUnit::Meters, TimeUnit::Seconds);
}

impl FromStr for SpeedUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split("/").collect_vec()[..] {
            [du_str, tu_str] => {
                let du = DistanceUnit::from_str(du_str).map_err(|e| {
                    format!(
                        "speed unit has invalid distance unit value '{}', error: {}",
                        du_str, e
                    )
                })?;
                let tu = TimeUnit::from_str(du_str).map_err(|e| {
                    format!(
                        "speed unit has invalid time unit value '{}', error: {}",
                        tu_str, e
                    )
                })?;
                Ok(SpeedUnit(du, tu))
            }
            ["mph"] => Ok(SpeedUnit::MPH),
            ["kph"] => Ok(SpeedUnit::KPH),
            _ => Err(format!(
                "expected speed unit as 'kph', 'mph', or in the format '<distance unit>/<time unit>', found: {}",
                s
            )),
        }
    }
}

impl<'de> Deserialize<'de> for SpeedUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for SpeedUnit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl Convert<Speed> for SpeedUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Speed>, to: &Self) -> Result<(), UnitError> {
        if value.as_f64() <= 0.0 {
            return Ok(());
        }
        let (from_du, from_tu) = (self.0, self.1);
        let (to_du, to_tu) = (to.0, to.1);
        if from_du == to_du && from_tu == to_tu {
            return Ok(());
        }

        let mut dist_convert = Cow::Owned(Distance::from(value.as_f64()));
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
        let (speed, _) =
            Speed::from_distance_and_time((&dist_convert, &to_du), (&time_convert, &to_tu))?;
        *value.to_mut() = speed;
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
        (Speed::from(75.0), SpeedUnit::MPH)
    }
}

#[cfg(test)]
mod test {

    use std::borrow::Cow;

    use super::{DistanceUnit as D, SpeedUnit as S, TimeUnit as T, *};

    fn assert_approx_eq(a: Speed, b: Speed, error: f64) {
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
