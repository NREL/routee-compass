use super::{Distance, Speed, Time};
use super::{DistanceUnit, TimeUnit, UnitError, BASE_DISTANCE, BASE_SPEED, BASE_TIME};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SpeedUnit {
    KilometersPerHour,
    MilesPerHour,
    MetersPerSecond,
}

impl SpeedUnit {
    /// converts a value from the current speed unit to some target speed unit.
    pub fn convert(&self, value: Speed, target: SpeedUnit) -> Speed {
        use SpeedUnit as S;
        match (self, target) {
            (S::KilometersPerHour, S::KilometersPerHour) => value,
            (S::KilometersPerHour, S::MilesPerHour) => value * 0.621371,
            (S::KilometersPerHour, S::MetersPerSecond) => value * 0.2777777778,
            (S::MilesPerHour, S::KilometersPerHour) => value * 1.60934,
            (S::MilesPerHour, S::MilesPerHour) => value,
            (S::MilesPerHour, S::MetersPerSecond) => value * 0.44704,
            (S::MetersPerSecond, S::KilometersPerHour) => value * 3.6,
            (S::MetersPerSecond, S::MilesPerHour) => value * 2.237,
            (S::MetersPerSecond, S::MetersPerSecond) => value,
        }
    }

    /// calculates a speed value based on the SpeedUnit and incoming time/distance values
    /// in their unit types. First converts both Time and Distance values to the Compass
    /// base units. performs the division operation to get speed and converts to the target
    /// speed unit.
    pub fn calculate_speed(
        &self,
        time: Time,
        time_unit: TimeUnit,
        distance: Distance,
        distance_unit: DistanceUnit,
    ) -> Result<Speed, UnitError> {
        let d = distance_unit.convert(distance, BASE_DISTANCE);
        let t = time_unit.convert(time, BASE_TIME);
        if t <= Time::ZERO {
            return Err(UnitError::SpeedFromTimeAndDistanceError(time, distance));
        } else {
            let s = Speed::new(d.to_f64() / t.to_f64());
            Ok(BASE_SPEED.convert(s, self.clone()))
        }
    }
}

#[cfg(test)]
mod test {

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
    fn test_conversions() {
        assert_approx_eq(
            S::KilometersPerHour.convert(Speed::ONE, S::KilometersPerHour),
            Speed::ONE,
            0.001,
        );
        assert_approx_eq(
            S::KilometersPerHour.convert(Speed::ONE, S::MilesPerHour),
            Speed::new(0.6215040398),
            0.001,
        );
        assert_approx_eq(
            S::KilometersPerHour.convert(Speed::ONE, S::MetersPerSecond),
            Speed::new(0.277778),
            0.001,
        );
        assert_approx_eq(
            S::MilesPerHour.convert(Speed::ONE, S::KilometersPerHour),
            Speed::new(1.60934),
            0.001,
        );
        assert_approx_eq(
            S::MilesPerHour.convert(Speed::ONE, S::MilesPerHour),
            Speed::ONE,
            0.001,
        );
        assert_approx_eq(
            S::MilesPerHour.convert(Speed::ONE, S::MetersPerSecond),
            Speed::new(0.44704),
            0.001,
        );
        assert_approx_eq(
            S::MetersPerSecond.convert(Speed::ONE, S::KilometersPerHour),
            Speed::new(3.6),
            0.001,
        );
        assert_approx_eq(
            S::MetersPerSecond.convert(Speed::ONE, S::MilesPerHour),
            Speed::new(2.23694),
            0.001,
        );
        assert_approx_eq(
            S::MetersPerSecond.convert(Speed::ONE, S::MetersPerSecond),
            Speed::ONE,
            0.001,
        );
    }

    #[test]
    fn test_calculate_fails() {
        let failure = S::MetersPerSecond.calculate_speed(
            Time::ZERO,
            TimeUnit::Seconds,
            Distance::ONE,
            DistanceUnit::Meters,
        );
        assert!(failure.is_err());
    }

    #[test]
    fn test_calculate_idempotent() {
        let one_mps = S::MetersPerSecond
            .calculate_speed(
                Time::ONE,
                TimeUnit::Seconds,
                Distance::ONE,
                DistanceUnit::Meters,
            )
            .unwrap();
        assert_eq!(Speed::ONE, one_mps);
    }

    #[test]
    fn test_calculate_imperial_to_si() {
        let speed_kph = S::KilometersPerHour
            .calculate_speed(
                Time::ONE,
                TimeUnit::Hours,
                Distance::ONE,
                DistanceUnit::Miles,
            )
            .unwrap();
        assert_approx_eq(Speed::new(1.60934), speed_kph, 0.001);
    }

    #[test]
    fn test_calculate_kph_to_base() {
        let speed_kph = BASE_SPEED
            .calculate_speed(
                Time::ONE,
                TimeUnit::Hours,
                Distance::ONE,
                DistanceUnit::Kilometers,
            )
            .unwrap();
        let expected = S::KilometersPerHour.convert(Speed::ONE, BASE_SPEED);
        assert_approx_eq(speed_kph, expected, 0.001);
    }

    #[test]
    fn test_calculate_base_to_kph() {
        let speed_kph = S::KilometersPerHour
            .calculate_speed(Time::ONE, BASE_TIME, Distance::ONE, BASE_DISTANCE)
            .unwrap();
        let expected = S::MetersPerSecond.convert(Speed::ONE, S::KilometersPerHour);
        assert_approx_eq(speed_kph, expected, 0.001);
    }

    #[test]
    fn test_calculate_mph_to_base() {
        let speed_kph = BASE_SPEED
            .calculate_speed(
                Time::ONE,
                TimeUnit::Hours,
                Distance::ONE,
                DistanceUnit::Miles,
            )
            .unwrap();
        let expected = S::MilesPerHour.convert(Speed::ONE, BASE_SPEED);
        assert_approx_eq(speed_kph, expected, 0.001);
    }

    #[test]
    fn test_calculate_base_to_mph() {
        let speed_kph = S::MilesPerHour
            .calculate_speed(Time::ONE, BASE_TIME, Distance::ONE, BASE_DISTANCE)
            .unwrap();
        let expected = S::MetersPerSecond.convert(Speed::ONE, S::MilesPerHour);
        assert_approx_eq(speed_kph, expected, 0.001);
    }
}
