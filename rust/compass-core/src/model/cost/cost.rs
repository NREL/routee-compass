use std::fmt::Display;

use derive_more::{Add, Div, Mul, Neg, Sum};
use ordered_float::OrderedFloat;

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Add, Mul, Div, Sum, Neg)]
pub struct Cost(pub OrderedFloat<f64>);

impl Cost {
    pub const ZERO: Cost = Cost(OrderedFloat(0.0));
    pub const INFINITY: Cost = Cost(OrderedFloat(f64::MAX));
}

impl Cost {
    pub fn from_f64(f: f64) -> Cost {
        Cost(OrderedFloat(f))
    }
    pub fn from_f32(f: f32) -> Cost {
        Cost(OrderedFloat(f64::from(f)))
    }
    pub fn into_f64(self) -> f64 {
        self.0.into_inner()
    }
    pub fn into_i64(self) -> i64 {
        self.0.into_inner().round() as i64
    }
}

impl Display for Cost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
