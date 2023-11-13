use std::fmt::Display;

use derive_more::{Add, Div, Mul, Neg, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::util::unit::{as_f64::AsF64, *};

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
pub struct Cost(pub OrderedFloat<f64>);

impl Cost {
    /// represents zero cost
    pub const ZERO: Cost = Cost(OrderedFloat(0.0));
    /// represents the maximum possible cost
    pub const INFINITY: Cost = Cost(OrderedFloat(f64::MAX));
    pub fn new(value: f64) -> Cost {
        Cost(OrderedFloat(value))
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
