use derive_more::{Add, Mul, Sum};

/// represents time in milliseconds. can be positive or negative.
#[derive(Copy, Clone, Eq, PartialEq, Add, Mul, Sum)]
pub struct Milliseconds(pub i64);
