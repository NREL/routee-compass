use super::{baseunit, Convert, UnitError, Weight};
use crate::{model::unit::AsF64, util::serde::serde_ops::string_deserialize};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WeightUnit {
    Pounds,
    Tons,
    Kg,
}

impl Convert<Weight> for WeightUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Weight>, to: &Self) -> Result<(), UnitError> {
        use WeightUnit as S;
        let conversion_factor = match (self, to) {
            (S::Pounds, S::Pounds) => None,
            (S::Pounds, S::Tons) => Some(0.0005),
            (S::Pounds, S::Kg) => Some(0.45359291),
            (S::Tons, S::Pounds) => Some(2000.0),
            (S::Tons, S::Tons) => None,
            (S::Tons, S::Kg) => Some(907.185),
            (S::Kg, S::Pounds) => Some(2.20462),
            (S::Kg, S::Tons) => Some(0.00110231),
            (S::Kg, S::Kg) => None,
        };
        if let Some(factor) = conversion_factor {
            let updated = Weight::from(value.as_ref().as_f64() * factor);
            *value.to_mut() = updated;
        }
        Ok(())
    }

    fn convert_to_base(&self, value: &mut std::borrow::Cow<Weight>) -> Result<(), UnitError> {
        self.convert(value, &baseunit::WEIGHT_UNIT)
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

    use super::WeightUnit as D;
    use crate::model::unit::*;
    use std::borrow::Cow;

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
    fn test_pounds_pounds() {
        let mut value = Cow::Owned(Weight::from(1.0));
        D::Pounds.convert(&mut value, &D::Pounds).unwrap();
        assert_approx_eq(*value, Weight::from(1.0), 0.0001);
    }
    #[test]
    fn test_pounds_tons() {
        let mut value = Cow::Owned(Weight::from(1.0));
        D::Pounds.convert(&mut value, &D::Tons).unwrap();
        assert_approx_eq(*value, Weight::from(0.0005), 0.0001);
    }
    #[test]
    fn test_pounds_kg() {
        let mut value = Cow::Owned(Weight::from(1.0));
        D::Pounds.convert(&mut value, &D::Kg).unwrap();
        assert_approx_eq(*value, Weight::from(0.453592), 0.0001);
    }
    #[test]
    fn test_tons_pounds() {
        let mut value = Cow::Owned(Weight::from(1.0));
        D::Tons.convert(&mut value, &D::Pounds).unwrap();
        assert_approx_eq(*value, Weight::from(2000.0), 0.0001);
    }
    #[test]
    fn test_tons_tons() {
        let mut value = Cow::Owned(Weight::from(1.0));
        D::Tons.convert(&mut value, &D::Tons).unwrap();
        assert_approx_eq(*value, Weight::from(1.0), 0.0001);
    }
    #[test]
    fn test_tons_kg() {
        let mut value = Cow::Owned(Weight::from(1.0));
        D::Tons.convert(&mut value, &D::Kg).unwrap();
        assert_approx_eq(*value, Weight::from(907.185), 0.0001);
    }
    #[test]
    fn test_kg_pounds() {
        let mut value = Cow::Owned(Weight::from(1.0));
        D::Kg.convert(&mut value, &D::Pounds).unwrap();
        assert_approx_eq(*value, Weight::from(2.20462), 0.0001);
    }
    #[test]
    fn test_kg_tons() {
        let mut value = Cow::Owned(Weight::from(1.0));
        D::Kg.convert(&mut value, &D::Tons).unwrap();
        assert_approx_eq(*value, Weight::from(0.00110231), 0.0001);
    }
    #[test]
    fn test_kg_kg() {
        let mut value = Cow::Owned(Weight::from(1.0));
        D::Kg.convert(&mut value, &D::Kg).unwrap();
        assert_approx_eq(*value, Weight::from(1.0), 0.0001);
    }
}
