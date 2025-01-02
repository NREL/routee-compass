use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

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
pub struct EnergyRate(pub InternalFloat);

impl AsF64 for EnergyRate {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl PartialOrd for EnergyRate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for EnergyRate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for EnergyRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl EnergyRate {
    pub fn new(value: f64) -> EnergyRate {
        EnergyRate(InternalFloat::new(value))
    }
    pub const ZERO: EnergyRate = EnergyRate(InternalFloat::ZERO);
    pub const ONE: EnergyRate = EnergyRate(InternalFloat::ONE);
}
