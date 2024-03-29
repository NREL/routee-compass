use super::cost_model_service::CostModelService;
use crate::app::compass::config::{
    compass_configuration_error::CompassConfigurationError,
    compass_configuration_field::CompassConfigurationField,
    config_json_extension::ConfigJsonExtensions,
};
use routee_compass_core::model::cost::{
    cost_aggregation::CostAggregation, network::network_cost_rate::NetworkCostRate,
    vehicle::vehicle_cost_rate::VehicleCostRate,
};
use std::{collections::HashMap, sync::Arc};

pub struct CostModelBuilder {}

impl CostModelBuilder {
    pub fn build(
        &self,
        config: &serde_json::Value,
    ) -> Result<CostModelService, CompassConfigurationError> {
        let parent_key = CompassConfigurationField::Cost.to_string();
        let vehicle_rates: HashMap<String, VehicleCostRate> =
            config.get_config_serde(&"vehicle_rates", &parent_key)?;
        let network_rates: HashMap<String, NetworkCostRate> =
            config.get_config_serde(&"network_rates", &parent_key)?;

        let coefficients: HashMap<String, f64> =
            config.get_config_serde(&"weights", &parent_key)?;
        let cost_aggregation: CostAggregation =
            config.get_config_serde(&"cost_aggregation", &parent_key)?;

        let model = CostModelService {
            vehicle_rates: Arc::new(vehicle_rates),
            network_rates: Arc::new(network_rates),
            weights: Arc::new(coefficients),
            cost_aggregation,
        };
        Ok(model)
    }
}
