use self::internal_float::InternalFloat;
use crate::model::unit::{AsF64, *};
use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

/// Represents the cost for traversing a graph edge.
/// A cost does not carry any units but can be built from a unit type like [`Time`] or [`Energy`]  

#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    Neg,
    Serialize,
    Deserialize,
    Allocative,
)]
pub struct Cost(InternalFloat);

impl Cost {
    /// represents zero cost, unit of addition operation
    pub const ZERO: Cost = Cost(InternalFloat::ZERO);

    /// represents one cost, unit of multiplication operation
    pub const ONE: Cost = Cost(InternalFloat::ONE);

    /// represents the maximum possible cost
    pub const INFINITY: Cost = Cost(InternalFloat::INFINITY);

    /// when path search costs must be strictly positive, this value
    /// is used as a sentinel in place of non-positive costs
    pub const MIN_COST: Cost = Cost(InternalFloat::MIN);

    /// helper to construct a Cost from an f64
    pub fn new(value: f64) -> Cost {
        Cost(InternalFloat::new(value))
    }

    /// helper to enforce costs that are strictly positive
    pub fn enforce_strictly_positive(cost: Cost) -> Cost {
        if cost <= Cost::ZERO {
            Cost::MIN_COST
        } else {
            cost
        }
    }

    /// helper to enforce costs that are zero or greater
    pub fn enforce_non_negative(cost: Cost) -> Cost {
        if cost < Cost::ZERO {
            Cost::ZERO
        } else {
            cost
        }
    }
}

impl From<f64> for Cost {
    fn from(f: f64) -> Self {
        Cost(InternalFloat::new(f))
    }
}

impl AsF64 for Cost {
    fn as_f64(&self) -> f64 {
        self.0 .0
    }
}

impl From<Cost> for f64 {
    fn from(val: Cost) -> Self {
        val.0.into_inner()
    }
}

impl Display for Cost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReverseCost(std::cmp::Reverse<Cost>);

impl From<Cost> for ReverseCost {
    fn from(cost: Cost) -> Self {
        ReverseCost(std::cmp::Reverse(cost))
    }
}

impl Allocative for ReverseCost {
    fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
        let _visitor = visitor.enter_self_sized::<Self>();
    }
}

impl Deref for ReverseCost {
    type Target = std::cmp::Reverse<Cost>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReverseCost {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
