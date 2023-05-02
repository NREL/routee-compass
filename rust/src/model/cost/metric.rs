#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Metric(i64);

impl Metric {
    pub const ZERO: Metric = Metric(0);
}
