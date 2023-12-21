use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

use super::as_f64::AsF64;

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
)]
pub struct EnergyRate(pub OrderedFloat<f64>);

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
        write!(f, "{}", self.0)
    }
}

impl EnergyRate {
    pub fn new(value: f64) -> EnergyRate {
        EnergyRate(OrderedFloat(value))
    }
    pub const ZERO: EnergyRate = EnergyRate(OrderedFloat(0.0));
    pub const ONE: EnergyRate = EnergyRate(OrderedFloat(1.0));
}
