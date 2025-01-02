use routee_compass_core::model::{frontier::FrontierModelError, network::edge_id::EdgeId};
use serde::Deserialize;

use super::vehicle_restriction::VehicleRestriction;

#[derive(Debug, Clone, Deserialize)]
pub struct RestrictionRow {
    pub edge_id: EdgeId,
    pub restriction_name: String,
    pub restriction_value: f64,
    pub restriction_unit: String,
}

impl RestrictionRow {
    pub fn to_restriction(&self) -> Result<VehicleRestriction, FrontierModelError> {
        // use serde to deserialize the restriction value
        let json = serde_json::json!({
            self.restriction_name.clone(): (self.restriction_value, self.restriction_unit.clone())
        });
        let restriction: VehicleRestriction = serde_json::from_value(json).map_err(|e| {
            FrontierModelError::BuildError(format!(
                "Unable to deserialize restriction {:?} due to: {}",
                self, e
            ))
        })?;
        Ok(restriction)
    }
}
