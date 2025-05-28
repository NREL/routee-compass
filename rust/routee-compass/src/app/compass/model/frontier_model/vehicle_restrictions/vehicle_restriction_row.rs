use super::ComparisonOperation;
use routee_compass_core::model::network::edge_id::EdgeId;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RestrictionRow {
    pub edge_id: EdgeId,
    pub name: String,
    pub value: f64,
    pub operation: ComparisonOperation,
    pub unit: String,
}
