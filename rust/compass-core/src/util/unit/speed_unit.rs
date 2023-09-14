use serde::{Deserialize, Serialize};

use super::Speed;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SpeedUnit {
    KilometersPerHour,
    MilesPerHour,
    MetersPerSecond,
}

impl SpeedUnit {
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
}

#[cfg(test)]
mod test {

    use super::Speed;
    use super::SpeedUnit as S;

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
}
