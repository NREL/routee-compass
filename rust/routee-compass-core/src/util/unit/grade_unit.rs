use super::Grade;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum GradeUnit {
    Percent,
    Decimal,
    Millis,
}

impl GradeUnit {
    pub fn convert(&self, value: Grade, target: GradeUnit) -> Grade {
        use GradeUnit as G;
        match (self, target) {
            (G::Percent, G::Percent) => value,
            (G::Decimal, G::Decimal) => value,
            (G::Millis, G::Millis) => value,
            (G::Percent, G::Decimal) => value / 100.0,
            (G::Percent, G::Millis) => value * 10.0,
            (G::Decimal, G::Percent) => value * 100.0,
            (G::Decimal, G::Millis) => value * 1000.0,
            (G::Millis, G::Percent) => value / 10.0,
            (G::Millis, G::Decimal) => value / 1000.0,
        }
    }
}

impl std::fmt::Display for GradeUnit {
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

    use super::Grade;
    use super::GradeUnit as G;

    fn assert_approx_eq(a: Grade, b: Grade, error: f64) {
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
            G::Percent.convert(Grade::new(10.0), G::Decimal),
            Grade::new(0.1),
            0.001,
        );
        assert_approx_eq(
            G::Percent.convert(Grade::new(10.0), G::Millis),
            Grade::new(100.0),
            0.001,
        );
        assert_approx_eq(
            G::Decimal.convert(Grade::new(0.1), G::Percent),
            Grade::new(10.0),
            0.001,
        );
        assert_approx_eq(
            G::Decimal.convert(Grade::new(0.1), G::Millis),
            Grade::new(100.0),
            0.001,
        );
        assert_approx_eq(
            G::Millis.convert(Grade::new(100.0), G::Percent),
            Grade::new(10.0),
            0.001,
        );
        assert_approx_eq(
            G::Millis.convert(Grade::new(100.0), G::Decimal),
            Grade::new(0.1),
            0.001,
        );
    }
}
