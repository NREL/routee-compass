use super::VehicleParameter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ComparisonOperation {
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = "=")]
    Equal,
    #[serde(rename = "<=")]
    LessThanOrEqual,
    #[serde(rename = ">=")]
    GreaterThanOrEqual,
}

impl std::fmt::Display for ComparisonOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ComparisonOperation::LessThan => "<".to_string(),
            ComparisonOperation::GreaterThan => ">".to_string(),
            ComparisonOperation::Equal => "=".to_string(),
            ComparisonOperation::LessThanOrEqual => "<=".to_string(),
            ComparisonOperation::GreaterThanOrEqual => ">=".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl ComparisonOperation {
    /// leverage the PartialCmp implementation for VehicleParameter to test if a particular comparison is true.
    pub fn compare_parameters(
        &self,
        query: &VehicleParameter,
        restriction: &VehicleParameter,
    ) -> bool {
        match (self, query, restriction) {
            (ComparisonOperation::LessThan, q, r) => q < r,
            (ComparisonOperation::GreaterThan, q, r) => q > r,
            (ComparisonOperation::Equal, q, r) => q == r,
            (ComparisonOperation::LessThanOrEqual, q, r) => q <= r,
            (ComparisonOperation::GreaterThanOrEqual, q, r) => q >= r,
        }
    }
}
