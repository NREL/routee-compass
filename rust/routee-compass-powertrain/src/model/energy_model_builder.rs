use crate::model::energy_model_service::EnergyModelService;
use crate::model::BevEnergyModel;
use crate::model::IceEnergyModel;
use crate::model::PhevEnergyModel;
use routee_compass_core::config::ConfigJsonExtensions;
use routee_compass_core::model::traversal::TraversalModelBuilder;
use routee_compass_core::model::traversal::TraversalModelError;
use routee_compass_core::model::traversal::TraversalModelService;
use std::collections::HashMap;
use std::sync::Arc;

pub struct EnergyModelBuilder {}

impl TraversalModelBuilder for EnergyModelBuilder {
    fn build(
        &self,
        params: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let parent_key = String::from("energy traversal model");

        let vehicle_configs = params
            .get_config_array(&"vehicles", &parent_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        // read all vehicle configurations
        let mut vehicle_library = HashMap::new();
        for vehicle_config in vehicle_configs {
            let model_name = vehicle_config
                .get_config_string(&"name", &parent_key)
                .map_err(|e| {
                    TraversalModelError::BuildError(format!(
                        "vehicle model missing 'name' field: {e}"
                    ))
                })?;
            let vehicle_type = vehicle_config
                .get_config_string(&"type", &parent_key)
                .map_err(|e| {
                    TraversalModelError::BuildError(format!(
                        "vehicle model missing 'type' field: {e}"
                    ))
                })?;
            let service: Arc<dyn TraversalModelService> = match vehicle_type.as_str() {
                "ice" => Arc::new(IceEnergyModel::try_from(&vehicle_config)?),
                "bev" => Arc::new(BevEnergyModel::try_from(&vehicle_config)?),
                "phev" => Arc::new(PhevEnergyModel::try_from(&vehicle_config)?),
                _ => {
                    return Err(TraversalModelError::BuildError(format!(
                        "unknown vehicle model type: {vehicle_type}"
                    )));
                }
            };

            vehicle_library.insert(model_name, service);
        }

        let service = EnergyModelService::new(vehicle_library)?;

        Ok(Arc::new(service))
    }
}
