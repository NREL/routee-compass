use super::{baseunit, Convert, Grade, UnitError};
use crate::{model::unit::AsF64, util::serde::serde_ops::string_deserialize};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum GradeUnit {
    Percent,
    Decimal,
    Millis,
}

impl Convert<Grade> for GradeUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Grade>, to: &Self) -> Result<(), UnitError> {
        use GradeUnit as G;
        let conversion_factor = match (self, to) {
            (G::Percent, G::Percent) => None,
            (G::Decimal, G::Decimal) => None,
            (G::Millis, G::Millis) => None,
            (G::Percent, G::Decimal) => Some(0.01),
            (G::Percent, G::Millis) => Some(10.0),
            (G::Decimal, G::Percent) => Some(100.0),
            (G::Decimal, G::Millis) => Some(1000.0),
            (G::Millis, G::Percent) => Some(0.1),
            (G::Millis, G::Decimal) => Some(0.001),
        };
        if let Some(factor) = conversion_factor {
            let updated = Grade::from(value.as_ref().as_f64() * factor);
            *value.to_mut() = updated;
        }
        Ok(())
    }

    fn convert_to_base(&self, value: &mut std::borrow::Cow<Grade>) -> Result<(), UnitError> {
        self.convert(value, &baseunit::GRADE_UNIT)
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

impl FromStr for GradeUnit {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        string_deserialize(s)
    }
}

#[cfg(test)]
mod test {

    use crate::model::unit::{GradeUnit as G, *};
    use std::borrow::Cow;

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
    fn test_pct_dec() {
        let mut value = Cow::Owned(Grade::from(10.0));
        G::Percent.convert(&mut value, &G::Decimal).unwrap();
        assert_approx_eq(value.into_owned(), Grade::from(0.1), 0.001);
    }
    #[test]
    fn test_pct_mil() {
        let mut value = Cow::Owned(Grade::from(10.0));
        G::Percent.convert(&mut value, &G::Millis).unwrap();
        assert_approx_eq(value.into_owned(), Grade::from(100.0), 0.001);
    }
    #[test]
    fn test_dec_pct() {
        let mut value = Cow::Owned(Grade::from(0.1));
        G::Decimal.convert(&mut value, &G::Percent).unwrap();
        assert_approx_eq(value.into_owned(), Grade::from(10.0), 0.001);
    }
    #[test]
    fn test_dec_mil() {
        let mut value = Cow::Owned(Grade::from(0.1));
        G::Decimal.convert(&mut value, &G::Millis).unwrap();
        assert_approx_eq(value.into_owned(), Grade::from(100.0), 0.001);
    }
    #[test]
    fn test_mil_pct() {
        let mut value = Cow::Owned(Grade::from(100.0));
        G::Millis.convert(&mut value, &G::Percent).unwrap();
        assert_approx_eq(value.into_owned(), Grade::from(10.0), 0.001);
    }
    #[test]
    fn test_mil_dec() {
        let mut value = Cow::Owned(Grade::from(100.0));
        G::Millis.convert(&mut value, &G::Decimal).unwrap();
        assert_approx_eq(value.into_owned(), Grade::from(0.1), 0.001);
    }
}
