use std::collections::{HashMap, HashSet};

use routee_compass_core::model::cost::{
    cost_aggregation::CostAggregation, network::network_cost_mapping::NetworkUtilityMapping,
    vehicle::vehicle_cost_mapping::VehicleUtilityMapping,
};

use crate::app::compass::config::{
    compass_configuration_error::CompassConfigurationError,
    config_json_extension::ConfigJsonExtensions,
};

use super::cost_model_service::CostModelService;

pub struct CostModelBuilder {}

impl CostModelBuilder {
    pub fn build(
        &self,
        config: &serde_json::Value,
    ) -> Result<CostModelService, CompassConfigurationError> {
        let vehicle_mapping: Option<HashMap<String, VehicleUtilityMapping>> =
            config.get_config_serde_optional(&"vehicle_mapping", &"utility")?;
        let network_mapping: Option<HashMap<String, NetworkUtilityMapping>> =
            config.get_config_serde_optional(&"network_mapping", &"utility")?;
        let default_vehicle_dimensions: Option<HashSet<String>> =
            config.get_config_serde_optional(&"default_vehicle_dimensions", &"utility")?;
        let default_cost_aggregation: Option<CostAggregation> =
            config.get_config_serde_optional(&"default_cost_aggregation", &"utility")?;
        let model = CostModelService::new(
            vehicle_mapping,
            network_mapping,
            default_vehicle_dimensions,
            default_cost_aggregation,
        )?;
        Ok(model)
    }
}
