use super::{
    vehicle_restriction_model::VehicleRestrictionFrontierModel, VehicleParameter,
    VehicleParameterConfig, VehicleParameterType, VehicleRestriction,
};
use crate::{
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
        Arc<HashMap<EdgeId, CompactOrderedHashMap<VehicleParameterType, VehicleRestriction>>>,
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
        let vehicle_parameter_configs: Vec<VehicleParameterConfig> =
            serde_json::from_value(vp_json.clone()).map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "Unable to deserialize `vehicle_parameters` key: {}",
                    e
                ))
            })?;
        let vehicle_parameters: Vec<VehicleParameter> = vehicle_parameter_configs
            .into_iter()
            .map(|vpc| vpc.into())
            .collect();
        let model = VehicleRestrictionFrontierModel {
            service,
            vehicle_parameters,
        };

        Ok(Arc::new(model))
    }
}
