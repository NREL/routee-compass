use crate::util::serde::serde_ops::string_deserialize;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uom::si::f64::Ratio;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GradeUnit {
    Percent,
    Decimal,
    Millis,
}

impl GradeUnit {
    pub fn to_uom(&self, value: f64) -> Ratio {
        use GradeUnit as G;
        match self {
            G::Percent => Ratio::new::<uom::si::ratio::percent>(value),
            G::Decimal => Ratio::new::<uom::si::ratio::ratio>(value),
            G::Millis => Ratio::new::<uom::si::ratio::per_mille>(value),
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

impl FromStr for GradeUnit {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        string_deserialize(s)
    }
}
