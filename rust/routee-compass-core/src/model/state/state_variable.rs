use crate::model::unit::{AsF64, Distance, Energy, Time};
use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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
impl From<Distance> for StateVariable {
    fn from(value: Distance) -> Self {
        StateVariable(value.as_f64())
    }
}
impl From<Time> for StateVariable {
    fn from(value: Time) -> Self {
        StateVariable(value.as_f64())
    }
}
impl From<Energy> for StateVariable {
    fn from(value: Energy) -> Self {
        StateVariable(value.as_f64())
    }
}

impl From<&StateVariable> for f64 {
    fn from(val: &StateVariable) -> Self {
        val.0
    }
}
