use std::fmt::Display;

use derive_more::{Add, Div, Mul, Neg, Sum};

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Add, Mul, Div, Sum, Neg)]
pub struct Cost(pub i64);

impl Cost {
    pub const ZERO: Cost = Cost(0);
    pub const INFINITY: Cost = Cost(i64::MAX);
}

impl Display for Cost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
