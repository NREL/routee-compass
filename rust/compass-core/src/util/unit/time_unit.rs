use serde::{Deserialize, Serialize};

use super::{
    Distance, DistanceUnit, Speed, SpeedUnit, Time, UnitError, BASE_DISTANCE, BASE_SPEED, BASE_TIME,
};

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

    /// calculates a time value based on the TimeUnit and incoming speed/distance values
    /// in their unit types. First converts both Speed and Distance values to the Compass
    /// base units. performs the division operation to get time and converts to the target
    /// time unit.
    pub fn calculate_time(
        &self,
        speed: Speed,
        speed_unit: SpeedUnit,
        distance: Distance,
        distance_unit: DistanceUnit,
    ) -> Result<Time, UnitError> {
        let d = distance_unit.convert(distance, BASE_DISTANCE);
        let s = speed_unit.convert(speed, BASE_SPEED);
        if s <= Speed::ZERO {
            return Err(UnitError::TimeFromSpeedAndDistanceError(speed, distance));
        } else {
            let t = Time::new(d.to_f64() / s.to_f64());
            Ok(BASE_TIME.convert(t, self.clone()))
        }
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

    #[test]
    fn test_calculate_fails() {
        let failure = T::Seconds.calculate_time(
            Speed::ZERO,
            SpeedUnit::KilometersPerHour,
            Distance::ONE,
            DistanceUnit::Meters,
        );
        assert!(failure.is_err());
    }

    #[test]
    fn test_calculate_idempotent() {
        let one_sec = T::Seconds
            .calculate_time(
                Speed::ONE,
                SpeedUnit::MetersPerSecond,
                Distance::ONE,
                DistanceUnit::Meters,
            )
            .unwrap();
        assert_eq!(Time::ONE, one_sec);
    }

    #[test]
    fn test_calculate_kph_to_base() {
        let time = BASE_TIME
            .calculate_time(
                Speed::ONE,
                SpeedUnit::KilometersPerHour,
                Distance::ONE,
                DistanceUnit::Kilometers,
            )
            .unwrap();
        let expected = T::Hours.convert(Time::ONE, BASE_TIME);
        assert_approx_eq(time, expected, 0.001);
    }

    #[test]
    fn test_calculate_base_to_kph() {
        let speed_kph = TimeUnit::Hours
            .calculate_time(Speed::ONE, BASE_SPEED, Distance::ONE, BASE_DISTANCE)
            .unwrap();
        let expected = BASE_TIME.convert(Time::ONE, T::Hours);
        assert_approx_eq(speed_kph, expected, 0.001);
    }

    #[test]
    fn test_calculate_mph_to_base() {
        let time = BASE_TIME
            .calculate_time(
                Speed::ONE,
                SpeedUnit::MilesPerHour,
                Distance::ONE,
                DistanceUnit::Miles,
            )
            .unwrap();
        let expected = T::Hours.convert(Time::ONE, BASE_TIME);
        assert_approx_eq(time, expected, 0.01);
    }

    #[test]
    fn test_calculate_base_to_mph() {
        let time = T::Hours
            .calculate_time(Speed::ONE, BASE_SPEED, Distance::ONE, BASE_DISTANCE)
            .unwrap();
        let expected = BASE_TIME.convert(Time::ONE, T::Hours);
        assert_approx_eq(time, expected, 0.001);
    }
}
