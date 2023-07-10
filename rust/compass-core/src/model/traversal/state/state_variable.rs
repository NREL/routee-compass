use std::fmt::Display;

use derive_more::{Add, Div, Mul, Neg, Sum};

#[derive(Copy, Clone, PartialEq, PartialOrd, Add, Mul, Div, Sum, Neg)]
pub struct StateVar(pub f64);

impl StateVar {
    pub const ZERO: StateVar = StateVar(0.0);
    pub const MAX: StateVar = StateVar(f64::MAX);
}

impl Display for StateVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
