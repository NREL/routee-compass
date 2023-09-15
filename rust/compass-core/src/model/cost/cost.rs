use std::fmt::Display;

use derive_more::{Add, Div, Mul, Neg, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::util::unit::*;

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
    pub const ZERO: Cost = Cost(OrderedFloat(0.0));
    pub const INFINITY: Cost = Cost(OrderedFloat(f64::MAX));
    pub fn new(value: f64) -> Cost {
        return Cost(OrderedFloat(value));
    }
}

impl From<Distance> for Cost {
    fn from(value: Distance) -> Self {
        Cost::new(value.to_f64())
    }
}
impl From<Time> for Cost {
    fn from(value: Time) -> Self {
        Cost::new(value.to_f64())
    }
}
impl From<EnergyRate> for Cost {
    fn from(value: EnergyRate) -> Self {
        Cost::new(value.to_f64())
    }
}
impl From<Speed> for Cost {
    fn from(value: Speed) -> Self {
        Cost::new(value.to_f64())
    }
}

impl From<f64> for Cost {
    fn from(f: f64) -> Self {
        Cost(OrderedFloat(f))
    }
}

impl Into<f64> for Cost {
    fn into(self) -> f64 {
        self.0.into_inner()
    }
}

impl Display for Cost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
