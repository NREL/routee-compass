use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(
    Copy,
    Clone,
    Serialize,
    Deserialize,
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
    PartialEq,
    Ord,
    PartialOrd,
)]
pub struct InternalFloat(OrderedFloat<f64>);

impl InternalFloat {
    pub fn new(value: f64) -> InternalFloat {
        InternalFloat(OrderedFloat(value))
    }
    pub const ZERO: InternalFloat = InternalFloat(OrderedFloat(0.0)); 
    pub const ONE: InternalFloat = InternalFloat(OrderedFloat(1.0));
    pub const INFINITY: InternalFloat = InternalFloat(OrderedFloat(f64::INFINITY));
    pub const MIN: InternalFloat = InternalFloat(OrderedFloat(0.0000000001));
}

impl From<f64> for InternalFloat {
    fn from(value: f64) -> Self {
        InternalFloat(OrderedFloat(value))
    }
}

impl Allocative for InternalFloat {
    fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
        visitor.visit_simple_sized::<Self>()
    }
}

impl Deref for InternalFloat {
    type Target = OrderedFloat<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InternalFloat {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
