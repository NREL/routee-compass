use super::Weight;
use crate::util::serde::serde_ops::string_deserialize;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WeightUnit {
    Pounds,
    Tons,
    Kg,
}

impl WeightUnit {
    pub fn convert(&self, value: &Weight, target: &WeightUnit) -> Weight {
        use WeightUnit as S;
        match (self, target) {
            (S::Pounds, S::Pounds) => *value,
            (S::Pounds, S::Tons) => *value / 2000.0,
            (S::Pounds, S::Kg) => *value / 2.20462,
            (S::Tons, S::Pounds) => *value * 2000.0,
            (S::Tons, S::Tons) => *value,
            (S::Tons, S::Kg) => *value * 907.185,
            (S::Kg, S::Pounds) => *value * 2.20462,
            (S::Kg, S::Tons) => *value / 907.185,
            (S::Kg, S::Kg) => *value,
        }
    }
}

impl std::fmt::Display for WeightUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

impl FromStr for WeightUnit {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        string_deserialize(s)
    }
}

#[cfg(test)]
mod test {

    use crate::model::unit::AsF64;

    use super::Weight;
    use super::WeightUnit as D;

    fn assert_approx_eq(a: Weight, b: Weight, error: f64) {
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
            D::Pounds.convert(&Weight::new(1.0), &D::Pounds),
            Weight::new(1.0),
            0.0001,
        );
        assert_approx_eq(
            D::Pounds.convert(&Weight::new(1.0), &D::Tons),
            Weight::new(0.0005),
            0.0001,
        );
        assert_approx_eq(
            D::Pounds.convert(&Weight::new(1.0), &D::Kg),
            Weight::new(0.453592),
            0.0001,
        );
        assert_approx_eq(
            D::Tons.convert(&Weight::new(1.0), &D::Pounds),
            Weight::new(2000.0),
            0.0001,
        );
        assert_approx_eq(
            D::Tons.convert(&Weight::new(1.0), &D::Tons),
            Weight::new(1.0),
            0.0001,
        );
        assert_approx_eq(
            D::Tons.convert(&Weight::new(1.0), &D::Kg),
            Weight::new(907.185),
            0.0001,
        );
        assert_approx_eq(
            D::Kg.convert(&Weight::new(1.0), &D::Pounds),
            Weight::new(2.20462),
            0.0001,
        );
        assert_approx_eq(
            D::Kg.convert(&Weight::new(1.0), &D::Tons),
            Weight::new(0.00110231),
            0.0001,
        );
        assert_approx_eq(
            D::Kg.convert(&Weight::new(1.0), &D::Kg),
            Weight::new(1.0),
            0.0001,
        );
    }
}
