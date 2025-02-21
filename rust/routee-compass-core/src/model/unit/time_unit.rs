use super::{baseunit, Convert, Time, UnitError};
use crate::{model::unit::AsF64, util::serde::serde_ops::string_deserialize};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
    Hours,
    #[default]
    Minutes,
    Seconds,
    Milliseconds,
}

impl Convert<Time> for TimeUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Time>, to: &Self) -> Result<(), UnitError> {
        /// converts a value from the current speed unit to some target speed unit.
        use TimeUnit as T;
        let conversion_factor = match (self, to) {
            (T::Hours, T::Hours) => None,
            (T::Minutes, T::Minutes) => None,
            (T::Seconds, T::Seconds) => None,
            (T::Milliseconds, T::Milliseconds) => None,
            (T::Hours, T::Minutes) => Some(60.0),
            (T::Hours, T::Seconds) => Some(3600.0),
            (T::Hours, T::Milliseconds) => Some(3600000.0),
            (T::Minutes, T::Hours) => Some(0.01666666667),
            (T::Minutes, T::Seconds) => Some(60.0),
            (T::Minutes, T::Milliseconds) => Some(60000.0),
            (T::Seconds, T::Hours) => Some(0.0002777777778),
            (T::Seconds, T::Minutes) => Some(0.01666666667),
            (T::Seconds, T::Milliseconds) => Some(1000.0),
            (T::Milliseconds, T::Hours) => Some(0.000000277777778),
            (T::Milliseconds, T::Minutes) => Some(0.00001666666667),
            (T::Milliseconds, T::Seconds) => Some(0.001),
        };
        if let Some(factor) = conversion_factor {
            let updated = Time::from(value.as_ref().as_f64() * factor);
            *value.to_mut() = updated;
        }
        Ok(())
    }

    fn convert_to_base(&self, value: &mut std::borrow::Cow<Time>) -> Result<(), UnitError> {
        self.convert(value, &baseunit::TIME_UNIT)
    }
}

impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

impl FromStr for TimeUnit {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        string_deserialize(s)
    }
}

#[cfg(test)]
mod test {

    use crate::model::unit::{TimeUnit as T, *};
    use std::borrow::Cow;

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
    fn convert_hr_to_hr() {
        let value = Time::ONE;
        let mut v_cow = Cow::Owned(value);
        T::Hours.convert(&mut v_cow, &T::Hours).unwrap();
        assert_approx_eq(*v_cow, Time::ONE, 0.001);
    }
    #[test]
    fn convert_hr_to_min() {
        let value = Time::ONE;
        let mut v_cow = Cow::Owned(value);
        T::Hours.convert(&mut v_cow, &T::Minutes).unwrap();
        assert_approx_eq(*v_cow, Time::from(60.0), 0.001);
    }
    #[test]
    fn convert_hr_to_sec() {
        let value = Time::ONE;
        let mut v_cow = Cow::Owned(value);
        T::Hours.convert(&mut v_cow, &T::Seconds).unwrap();
        assert_approx_eq(*v_cow, Time::from(3600.0), 0.001);
    }
    #[test]
    fn convert_hr_to_ms() {
        let value = Time::ONE;
        let mut v_cow = Cow::Owned(value);
        T::Hours.convert(&mut v_cow, &T::Milliseconds).unwrap();
        assert_approx_eq(*v_cow, Time::from(3600000.0), 0.001);
    }
    #[test]
    fn convert_min_to_hr() {
        let value = Time::from(60.0);
        let mut v_cow = Cow::Owned(value);
        T::Minutes.convert(&mut v_cow, &T::Hours).unwrap();
        assert_approx_eq(*v_cow, Time::ONE, 0.001);
    }
    #[test]
    fn convert_min_to_min() {
        let value = Time::from(60.0);
        let mut v_cow = Cow::Owned(value);
        T::Minutes.convert(&mut v_cow, &T::Minutes).unwrap();
        assert_approx_eq(*v_cow, Time::from(60.0), 0.001);
    }
    #[test]
    fn convert_min_to_sec() {
        let value = Time::from(60.0);
        let mut v_cow = Cow::Owned(value);
        T::Minutes.convert(&mut v_cow, &T::Seconds).unwrap();
        assert_approx_eq(*v_cow, Time::from(3600.0), 0.001);
    }
    #[test]
    fn convert_min_to_ms() {
        let value = Time::from(60.0);
        let mut v_cow = Cow::Owned(value);
        T::Minutes.convert(&mut v_cow, &T::Milliseconds).unwrap();
        assert_approx_eq(*v_cow, Time::from(3600000.0), 0.001);
    }
    #[test]
    fn convert_sec_to_hr() {
        let value = Time::from(3600.0);
        let mut v_cow = Cow::Owned(value);
        T::Seconds.convert(&mut v_cow, &T::Hours).unwrap();
        assert_approx_eq(*v_cow, Time::ONE, 0.001);
    }
    #[test]
    fn convert_sec_to_min() {
        let value = Time::from(3600.0);
        let mut v_cow = Cow::Owned(value);
        T::Seconds.convert(&mut v_cow, &T::Minutes).unwrap();
        assert_approx_eq(*v_cow, Time::from(60.0), 0.001);
    }
    #[test]
    fn convert_sec_to_sec() {
        let value = Time::from(3600.0);
        let mut v_cow = Cow::Owned(value);
        T::Seconds.convert(&mut v_cow, &T::Seconds).unwrap();
        assert_approx_eq(*v_cow, Time::from(3600.0), 0.001);
    }
    #[test]
    fn convert_sec_to_ms() {
        let value = Time::from(3600.0);
        let mut v_cow = Cow::Owned(value);
        T::Seconds.convert(&mut v_cow, &T::Milliseconds).unwrap();
        assert_approx_eq(*v_cow, Time::from(3600000.0), 0.001);
    }
    #[test]
    fn convert_ms_to_hr() {
        let value = Time::from(3600000.0);
        let mut v_cow = Cow::Owned(value);
        T::Milliseconds.convert(&mut v_cow, &T::Hours).unwrap();
        assert_approx_eq(*v_cow, Time::ONE, 0.001);
    }
    #[test]
    fn convert_ms_to_min() {
        let value = Time::from(3600000.0);
        let mut v_cow = Cow::Owned(value);
        T::Milliseconds.convert(&mut v_cow, &T::Minutes).unwrap();
        assert_approx_eq(*v_cow, Time::from(60.0), 0.001);
    }
    #[test]
    fn convert_ms_to_sec() {
        let value = Time::from(3600000.0);
        let mut v_cow = Cow::Owned(value);
        T::Milliseconds.convert(&mut v_cow, &T::Seconds).unwrap();
        assert_approx_eq(*v_cow, Time::from(3600.0), 0.001);
    }
    #[test]
    fn convert_ms_to_ms() {
        let value = Time::from(3600000.0);
        let mut v_cow = Cow::Owned(value);
        T::Milliseconds
            .convert(&mut v_cow, &T::Milliseconds)
            .unwrap();
        assert_approx_eq(*v_cow, Time::from(3600000.0), 0.001);
    }
}
