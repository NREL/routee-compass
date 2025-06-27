use super::{ComparisonOperation, VehicleParameterType};
use crate::model::network::edge_id::EdgeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RestrictionRow {
    pub edge_id: EdgeId,
    pub name: VehicleParameterType,
    pub value: f64,
    pub operation: ComparisonOperation,
    pub unit: String,
}
