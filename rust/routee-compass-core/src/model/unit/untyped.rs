use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

use super::internal_float::InternalFloat;

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
pub struct Untyped(pub InternalFloat);

impl PartialOrd for Untyped {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for Untyped {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for Untyped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
