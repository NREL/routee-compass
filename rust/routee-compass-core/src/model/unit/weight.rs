use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

use crate::model::state::StateVariable;

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
pub struct Weight(InternalFloat);

impl From<f64> for Weight {
    fn from(value: f64) -> Self {
        Weight(InternalFloat::new(value))
    }
}

impl AsF64 for Weight {
    fn as_f64(&self) -> f64 {
        **self.0
    }
}

impl AsF64 for &Weight {
    fn as_f64(&self) -> f64 {
        **self.0
    }
}

impl PartialOrd for Weight {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}
impl Ord for Weight {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
impl Display for Weight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl From<StateVariable> for Weight {
    fn from(value: StateVariable) -> Self {
        Weight::from(value.0)
    }
}

impl Weight {
    pub fn to_ordered_float(&self) -> OrderedFloat<f64> {
        *self.0
    }
    pub const ZERO: Weight = Weight(InternalFloat::ZERO);
    pub const ONE: Weight = Weight(InternalFloat::ONE);
}
