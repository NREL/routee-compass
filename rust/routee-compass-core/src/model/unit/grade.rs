use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display, str::FromStr};

use super::{internal_float::InternalFloat, AsF64};

#[derive(
    Copy,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Default,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    Neg,
    Allocative,
)]
pub struct Grade(pub InternalFloat);

impl From<f64> for Grade {
    fn from(value: f64) -> Self {
        Grade(InternalFloat::new(value))
    }
}

impl AsF64 for Grade {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl AsF64 for &Grade {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl PartialOrd for Grade {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for Grade {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl FromStr for Grade {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = str::parse::<f64>(s)
            .map_err(|e| format!("failure reading grade value {}: {}", s, e))?;
        Ok(Grade::from(f))
    }
}

impl Grade {
    pub const ZERO: Grade = Grade(InternalFloat::ZERO);
}
