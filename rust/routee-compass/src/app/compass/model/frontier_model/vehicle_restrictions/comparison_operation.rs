use serde::{Deserialize, Serialize};

use super::VehicleParameter;

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
    pub fn compare_parameters(&self, a: &VehicleParameter, b: &VehicleParameter) -> bool {
        println!("Comparing {:?} {} {:?}", a, self, b);
        match (self, a, b) {
            (ComparisonOperation::LessThan, a, b) => a < b,
            (ComparisonOperation::GreaterThan, a, b) => a > b,
            (ComparisonOperation::Equal, a, b) => a == b,
            (ComparisonOperation::LessThanOrEqual, a, b) => a <= b,
            (ComparisonOperation::GreaterThanOrEqual, a, b) => a >= b,
        }
    }
}
