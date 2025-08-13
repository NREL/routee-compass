use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(
    Copy,
    Clone,
    PartialEq,
    PartialOrd,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    Neg,
    Debug,
    Deserialize,
    Serialize,
    Allocative,
)]
pub struct StateVariable(pub f64);

impl StateVariable {
    pub const ZERO: StateVariable = StateVariable(0.0);
    pub const ONE: StateVariable = StateVariable(1.0);
    pub const ONE_HUNDRED: StateVariable = StateVariable(100.0);
    pub const MAX: StateVariable = StateVariable(f64::MAX);
}

impl Display for StateVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<StateVariable> for f64 {
    fn from(val: StateVariable) -> Self {
        val.0
    }
}
impl From<&StateVariable> for f64 {
    fn from(val: &StateVariable) -> Self {
        val.0
    }
}

impl FromStr for StateVariable {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<f64>().map(StateVariable).map_err(|e| {
            let msg = format!("failure decoding state variable {s} due to: {e:}");
            std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
        })
    }
}
