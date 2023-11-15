use super::Time;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
}

impl TimeUnit {
    pub fn convert(&self, value: Time, target: TimeUnit) -> Time {
        use TimeUnit as T;
        match (self, target) {
            (T::Hours, T::Hours) => value,
            (T::Minutes, T::Minutes) => value,
            (T::Seconds, T::Seconds) => value,
            (T::Milliseconds, T::Milliseconds) => value,
            (T::Hours, T::Minutes) => value * 60.0,
            (T::Hours, T::Seconds) => value * 3600.0,
            (T::Hours, T::Milliseconds) => value * 3600000.0,
            (T::Minutes, T::Hours) => value * 0.01666666667,
            (T::Minutes, T::Seconds) => value * 60.0,
            (T::Minutes, T::Milliseconds) => value * 60000.0,
            (T::Seconds, T::Hours) => value * 0.0002777777778,
            (T::Seconds, T::Minutes) => value * 0.01666666667,
            (T::Seconds, T::Milliseconds) => value * 1000.0,
            (T::Milliseconds, T::Hours) => value * 0.000000277777778,
            (T::Milliseconds, T::Minutes) => value * 0.00001666666667,
            (T::Milliseconds, T::Seconds) => value * 0.001,
        }
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

#[cfg(test)]
mod test {

    use super::{TimeUnit as T, *};

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
    fn test_conversions() {
        assert_approx_eq(T::Hours.convert(Time::ONE, T::Hours), Time::ONE, 0.001);
        assert_approx_eq(
            T::Hours.convert(Time::ONE, T::Minutes),
            Time::new(60.0),
            0.001,
        );
        assert_approx_eq(
            T::Hours.convert(Time::ONE, T::Seconds),
            Time::new(3600.0),
            0.001,
        );
        assert_approx_eq(
            T::Hours.convert(Time::ONE, T::Milliseconds),
            Time::new(3600000.0),
            0.001,
        );
        assert_approx_eq(
            T::Minutes.convert(Time::new(60.0), T::Hours),
            Time::ONE,
            0.001,
        );
        assert_approx_eq(
            T::Minutes.convert(Time::new(60.0), T::Minutes),
            Time::new(60.0),
            0.001,
        );
        assert_approx_eq(
            T::Minutes.convert(Time::new(60.0), T::Seconds),
            Time::new(3600.0),
            0.001,
        );
        assert_approx_eq(
            T::Minutes.convert(Time::new(60.0), T::Milliseconds),
            Time::new(3600000.0),
            0.001,
        );
        assert_approx_eq(
            T::Seconds.convert(Time::new(3600.0), T::Hours),
            Time::ONE,
            0.001,
        );
        assert_approx_eq(
            T::Seconds.convert(Time::new(3600.0), T::Minutes),
            Time::new(60.0),
            0.001,
        );
        assert_approx_eq(
            T::Seconds.convert(Time::new(3600.0), T::Seconds),
            Time::new(3600.0),
            0.001,
        );
        assert_approx_eq(
            T::Seconds.convert(Time::new(3600.0), T::Milliseconds),
            Time::new(3600000.0),
            0.001,
        );
        assert_approx_eq(
            T::Milliseconds.convert(Time::new(3600000.0), T::Hours),
            Time::ONE,
            0.001,
        );
        assert_approx_eq(
            T::Milliseconds.convert(Time::new(3600000.0), T::Minutes),
            Time::new(60.0),
            0.001,
        );
        assert_approx_eq(
            T::Milliseconds.convert(Time::new(3600000.0), T::Seconds),
            Time::new(3600.0),
            0.001,
        );
        assert_approx_eq(
            T::Milliseconds.convert(Time::new(3600000.0), T::Milliseconds),
            Time::new(3600000.0),
            0.001,
        );
    }
}
