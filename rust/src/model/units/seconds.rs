use derive_more::{Add, Mul, Sum, Div};

/// represents time in seconds. can be positive or negative.
#[derive(Copy, Clone, Eq, PartialEq, Add, Mul, Sum, Div)]
pub struct Seconds(pub i64);
