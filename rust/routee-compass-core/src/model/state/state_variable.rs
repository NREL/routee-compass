use crate::model::unit::{as_f64::AsF64, Distance, Energy, Time};
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
pub struct StateVar(pub f64);

impl StateVar {
    pub const ZERO: StateVar = StateVar(0.0);
    pub const ONE: StateVar = StateVar(1.0);
    pub const ONE_HUNDRED: StateVar = StateVar(100.0);
    pub const MAX: StateVar = StateVar(f64::MAX);
}

impl Display for StateVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<StateVar> for f64 {
    fn from(val: StateVar) -> Self {
        val.0
    }
}
impl From<Distance> for StateVar {
    fn from(value: Distance) -> Self {
        StateVar(value.as_f64())
    }
}
impl From<Time> for StateVar {
    fn from(value: Time) -> Self {
        StateVar(value.as_f64())
    }
}
impl From<Energy> for StateVar {
    fn from(value: Energy) -> Self {
        StateVar(value.as_f64())
    }
}
