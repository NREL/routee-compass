use indexmap::IndexMap;

use super::{
    vehicle_restriction_model::VehicleRestrictionConstraintModel, VehicleParameter,
    VehicleParameterConfig, VehicleParameterType, VehicleRestriction,
};
use crate::model::{
    constraint::{ConstraintModel, ConstraintModelError, ConstraintModelService},
    network::EdgeId,
    state::StateModel,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct VehicleRestrictionFrontierService {
    pub vehicle_restriction_lookup:
        Arc<HashMap<EdgeId, IndexMap<VehicleParameterType, VehicleRestriction>>>,
}

impl ConstraintModelService for VehicleRestrictionFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn ConstraintModel>, ConstraintModelError> {
        let service: Arc<VehicleRestrictionFrontierService> = Arc::new(self.clone());
        let vp_json = query.get("vehicle_parameters").ok_or_else(|| {
            ConstraintModelError::BuildError(
                "Missing field `vehicle_parameters` in query".to_string(),
            )
        })?;
        let vehicle_parameter_configs: Vec<VehicleParameterConfig> =
            serde_json::from_value(vp_json.clone()).map_err(|e| {
                ConstraintModelError::BuildError(format!(
                    "Unable to deserialize `vehicle_parameters` key: {e}"
                ))
            })?;
        let vehicle_parameters: Vec<VehicleParameter> = vehicle_parameter_configs
            .into_iter()
            .map(|vpc| vpc.into())
            .collect();
        let model = VehicleRestrictionConstraintModel {
            service,
            vehicle_parameters,
        };

        Ok(Arc::new(model))
    }
}
