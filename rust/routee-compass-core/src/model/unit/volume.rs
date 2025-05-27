use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use ordered_float::OrderedFloat;
use std::{cmp::Ordering, fmt::Display};

use crate::model::state::StateVariable;

use super::{internal_float::InternalFloat, AsF64};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    Neg,
    Allocative,
    derive_more::derive::From,
)]
pub struct Volume(InternalFloat);

impl AsF64 for &Volume {
    fn as_f64(&self) -> f64 {
        **self.0
    }
}
impl AsF64 for Volume {
    fn as_f64(&self) -> f64 {
        **self.0
    }
}

impl PartialOrd for Volume {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}
impl Ord for Volume {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
impl Display for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl From<StateVariable> for Volume {
    fn from(value: StateVariable) -> Self {
        Volume::from(value.0)
    }
}
impl From<&StateVariable> for Volume {
    fn from(value: &StateVariable) -> Self {
        Volume::from(value.0)
    }
}

impl From<f64> for Volume {
    fn from(value: f64) -> Self {
        Volume(InternalFloat::new(value))
    }
}

impl Volume {
    pub fn to_ordered_float(&self) -> OrderedFloat<f64> {
        *self.0
    }
    pub const ZERO: Volume = Volume(InternalFloat::ZERO);
    pub const ONE: Volume = Volume(InternalFloat::ONE);
}
