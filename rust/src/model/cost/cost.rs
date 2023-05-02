#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Cost(i64);

impl Cost {
    pub const ZERO: Cost = Cost(0);
}
