use derive_more::{Add, Mul, Sum, Div};

#[derive(Copy, Clone, Eq, PartialEq, Add, Mul, Sum, Div)]
pub struct CmPerSecond(pub u32);
