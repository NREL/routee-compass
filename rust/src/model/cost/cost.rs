use derive_more::{Add, Mul, Sum};

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Add, Mul, Sum)]
pub struct Cost(pub i64);

impl Cost {
    pub const ZERO: Cost = Cost(0);
}
