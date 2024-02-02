use allocative::Allocative;
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
struct FloatWrapper(OrderedFloat<f64>);

impl Allocative for FloatWrapper {
    fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
        visitor.visit_simple_sized::<Self>()
    }
}

impl FloatWrapper {
    pub fn to_f64(&self) -> f64 {
        (self.0).0
    }
}

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
pub struct Distance(pub FloatWrapper);

impl AsF64 for Distance {
    fn as_f64(&self) -> f64 {
        self.0.to_f64()
    }
}

impl PartialOrd for Distance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.0.cmp(&other.0.0))
    }
}

impl Ord for Distance {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.0.cmp(&other.0.0)
    }
}

impl Display for Distance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.0)
    }
}

impl Distance {
    pub fn new(value: f64) -> Distance {
        Distance(FloatWrapper(OrderedFloat(value)))
    }
    pub fn to_ordered_float(&self) -> OrderedFloat<f64> {
        self.0.0
    }
    pub const ZERO: Distance = Distance(FloatWrapper(OrderedFloat(0.0)));
    pub const ONE: Distance = Distance(FloatWrapper(OrderedFloat(1.0)));
}
