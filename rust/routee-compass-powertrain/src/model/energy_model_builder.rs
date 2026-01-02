use crate::model::energy_model_service::EnergyModelService;
use crate::model::BevEnergyModel;
use crate::model::IceEnergyModel;
use crate::model::PhevEnergyModel;
use config::Config;
use routee_compass_core::config::ConfigJsonExtensions;
use routee_compass_core::model::traversal::TraversalModelBuilder;
use routee_compass_core::model::traversal::TraversalModelError;
use routee_compass_core::model::traversal::TraversalModelService;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub struct EnergyModelBuilder {}

impl TraversalModelBuilder for EnergyModelBuilder {
    fn build(
        &self,
        params: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let parent_key = String::from("energy traversal model");

        let vehicle_files = params
            .get_config_array(&"vehicle_input_files", &parent_key)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        // read all vehicle configurations from files
        let mut vehicle_library = HashMap::new();
        for vehicle_file in vehicle_files {
            let file_path = vehicle_file.as_str().ok_or_else(|| {
                TraversalModelError::BuildError("vehicle file path must be a string".to_string())
            })?;

            let vehicle_config = Config::builder()
                .add_source(config::File::with_name(file_path))
                .build()
                .map_err(|e| {
                    TraversalModelError::BuildError(format!(
                        "failed to read vehicle config file '{}': {}",
                        file_path, e
                    ))
                })?;

            let vehicle_json = vehicle_config
                .try_deserialize::<serde_json::Value>()
                .map_err(|e| {
                    TraversalModelError::BuildError(format!(
                        "failed to parse vehicle config file '{}': {}",
                        file_path, e
                    ))
                })?
                .normalize_file_paths(Path::new(file_path), None)
                .map_err(|e| {
                    TraversalModelError::BuildError(format!(
                        "failed to normalize file paths in vehicle config file '{}': {}",
                        file_path, e
                    ))
                })?;

            let model_name = vehicle_json
                .get_config_string(&"name", &parent_key)
                .map_err(|e| {
                    TraversalModelError::BuildError(format!(
                        "vehicle model missing 'name' field in '{}': {}",
                        file_path, e
                    ))
                })?;
            let vehicle_type = vehicle_json
                .get_config_string(&"type", &parent_key)
                .map_err(|e| {
                    TraversalModelError::BuildError(format!(
                        "vehicle model missing 'type' field in '{}': {}",
                        file_path, e
                    ))
                })?;
            let service: Arc<dyn TraversalModelService> = match vehicle_type.as_str() {
                "ice" => Arc::new(IceEnergyModel::try_from(&vehicle_json)?),
                "bev" => Arc::new(BevEnergyModel::try_from(&vehicle_json)?),
                "phev" => Arc::new(PhevEnergyModel::try_from(&vehicle_json)?),
                _ => {
                    return Err(TraversalModelError::BuildError(format!(
                        "unknown vehicle model type in '{}': {}",
                        file_path, vehicle_type
                    )));
                }
            };

            vehicle_library.insert(model_name, service);
        }

        let service = EnergyModelService::new(vehicle_library)?;

        Ok(Arc::new(service))
    }
}
