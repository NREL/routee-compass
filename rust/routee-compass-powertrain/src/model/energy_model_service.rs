use routee_compass_core::model::traversal::{
    TraversalModel, TraversalModelError, TraversalModelService,
};
use std::collections::HashMap;
use std::sync::Arc;

/// holds a library of vehicle models as TraversalModelServices and selects one
/// based on the model_name field of the incoming query.
#[derive(Clone)]
pub struct EnergyModelService {
    pub vehicle_library: HashMap<String, Arc<dyn TraversalModelService>>,
}

impl EnergyModelService {
    pub fn new(
        vehicle_library: HashMap<String, Arc<dyn TraversalModelService>>,
    ) -> Result<Self, TraversalModelError> {
        Ok(EnergyModelService { vehicle_library })
    }
}

impl TraversalModelService for EnergyModelService {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let model_name = parameters
            .get("model_name")
            .ok_or_else(|| {
                TraversalModelError::BuildError("query missing 'model_name' field".to_string())
            })?
            .as_str()
            .ok_or_else(|| {
                TraversalModelError::BuildError("query 'model_name' is not a string".to_string())
            })?;

        let service = self.vehicle_library.get(model_name).ok_or_else(|| {
            TraversalModelError::BuildError(format!(
                "unknown vehicle model {}, must be one of [{}]",
                model_name,
                self.vehicle_library
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(",")
            ))
        })?;
        let model = service.build(parameters)?;
        Ok(model)
    }
}
