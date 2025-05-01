use super::{
    vehicle_parameters::VehicleParameters,
    vehicle_restriction_model::VehicleRestrictionFrontierModel,
};
use routee_compass_core::{
    model::{
        frontier::{FrontierModel, FrontierModelError, FrontierModelService},
        network::edge_id::EdgeId,
        state::StateModel,
    },
    util::compact_ordered_hash_map::CompactOrderedHashMap,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct VehicleRestrictionFrontierService {
    pub vehicle_restriction_lookup:
        Arc<HashMap<EdgeId, CompactOrderedHashMap<String, VehicleParameters>>>,
}

impl FrontierModelService for VehicleRestrictionFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let service: Arc<VehicleRestrictionFrontierService> = Arc::new(self.clone());
        let vp_json = query.get("vehicle_parameters").ok_or_else(|| {
            FrontierModelError::BuildError(
                "Missing field `vehicle_parameters` in query".to_string(),
            )
        })?;
        let vehicle_parameters: Vec<VehicleParameters> = serde_json::from_value(vp_json.clone())
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to deserialize `vehicle_parameters` key: {}",
                    e
                ))
            })?;
        let model = VehicleRestrictionFrontierModel {
            service,
            vehicle_parameters,
        };

        Ok(Arc::new(model))
    }
}
