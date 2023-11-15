use super::Distance;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DistanceUnit {
    Meters,
    Kilometers,
    Miles,
}

impl DistanceUnit {
    pub fn convert(&self, value: Distance, target: DistanceUnit) -> Distance {
        use DistanceUnit as S;
        match (self, target) {
            (S::Meters, S::Meters) => value,
            (S::Meters, S::Kilometers) => value * 0.001,
            (S::Meters, S::Miles) => value * 0.0006215040398,
            (S::Kilometers, S::Meters) => value * 1000.0,
            (S::Kilometers, S::Kilometers) => value,
            (S::Kilometers, S::Miles) => value * 0.6215040398,
            (S::Miles, S::Meters) => value * 1609.34,
            (S::Miles, S::Kilometers) => value * 1.60934,
            (S::Miles, S::Miles) => value,
        }
    }
}

impl std::fmt::Display for DistanceUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod test {

    use crate::util::unit::as_f64::AsF64;

    use super::Distance;
    use super::DistanceUnit as D;

    fn assert_approx_eq(a: Distance, b: Distance, error: f64) {
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
    fn test_conversions() {
        assert_approx_eq(
            D::Meters.convert(Distance::ONE, D::Meters),
            Distance::ONE,
            0.001,
        );
        assert_approx_eq(
            D::Meters.convert(Distance::ONE, D::Kilometers),
            Distance::new(0.001),
            0.001,
        );
        assert_approx_eq(
            D::Meters.convert(Distance::ONE, D::Miles),
            Distance::new(0.000621371),
            0.001,
        );
        assert_approx_eq(
            D::Kilometers.convert(Distance::ONE, D::Meters),
            Distance::new(1000.0),
            0.001,
        );
        assert_approx_eq(
            D::Kilometers.convert(Distance::ONE, D::Kilometers),
            Distance::ONE,
            0.001,
        );
        assert_approx_eq(
            D::Kilometers.convert(Distance::ONE, D::Miles),
            Distance::new(0.621371),
            0.001,
        );
        assert_approx_eq(
            D::Miles.convert(Distance::ONE, D::Meters),
            Distance::new(1609.34),
            0.001,
        );
        assert_approx_eq(
            D::Miles.convert(Distance::ONE, D::Kilometers),
            Distance::new(1.60934),
            0.001,
        );
        assert_approx_eq(
            D::Miles.convert(Distance::ONE, D::Miles),
            Distance::ONE,
            0.001,
        );
    }
}
