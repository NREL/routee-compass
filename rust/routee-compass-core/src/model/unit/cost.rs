use crate::model::unit::{as_f64::AsF64, *};
use derive_more::{Add, Div, Mul, Neg, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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
    Mul,
    Div,
    Sum,
    Neg,
    Serialize,
    Deserialize,
)]
pub struct Cost(OrderedFloat<f64>);

impl Cost {
    /// represents zero cost, unit of addition operation
    pub const ZERO: Cost = Cost(OrderedFloat(0.0));

    /// represents one cost, unit of multiplication operation
    pub const ONE: Cost = Cost(OrderedFloat(1.0));

    /// represents the maximum possible cost
    pub const INFINITY: Cost = Cost(OrderedFloat(f64::MAX));

    /// when path search costs must be strictly positive, this value
    /// is used as a sentinel in place of non-positive costs
    pub const MIN_COST: Cost = Cost(OrderedFloat(0.0000000001));

    /// helper to construct a Cost from an f64
    pub fn new(value: f64) -> Cost {
        Cost(OrderedFloat(value))
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

impl From<Distance> for Cost {
    fn from(value: Distance) -> Self {
        Cost::new(value.as_f64())
    }
}
impl From<Time> for Cost {
    fn from(value: Time) -> Self {
        Cost::new(value.as_f64())
    }
}
impl From<Energy> for Cost {
    fn from(value: Energy) -> Self {
        Cost::new(value.as_f64())
    }
}
impl From<Speed> for Cost {
    fn from(value: Speed) -> Self {
        Cost::new(value.as_f64())
    }
}

impl From<f64> for Cost {
    fn from(f: f64) -> Self {
        Cost(OrderedFloat(f))
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
        write!(f, "{}", self.0)
    }
}
