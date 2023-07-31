use derive_more::{Add, Div, Mul, Neg, Sum};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(
    Copy, Clone, PartialEq, PartialOrd, Add, Mul, Div, Sum, Neg, Debug, Deserialize, Serialize,
)]
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
