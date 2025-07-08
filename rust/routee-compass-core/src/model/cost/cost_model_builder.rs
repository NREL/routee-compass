use super::cost_model_service::CostModelService;
use crate::config::{CompassConfigurationError, CompassConfigurationField, ConfigJsonExtensions};
use crate::model::cost::{network::NetworkCostRate, CostAggregation, VehicleCostRate};
use std::{collections::HashMap, sync::Arc};

pub struct CostModelBuilder {}

impl CostModelBuilder {
    pub fn build(
        &self,
        config: &serde_json::Value,
    ) -> Result<CostModelService, CompassConfigurationError> {
        let parent_key = CompassConfigurationField::Cost.to_string();
        let vehicle_rates: HashMap<String, VehicleCostRate> = config
            .get_config_serde_optional(&"vehicle_rates", &parent_key)?
            .unwrap_or_default();
        let network_rates: HashMap<String, NetworkCostRate> = config
            .get_config_serde_optional(&"network_rates", &parent_key)?
            .unwrap_or_default();

        let weights: HashMap<String, f64> = config
            .get_config_serde_optional(&"weights", &parent_key)?
            .unwrap_or_default();
        let cost_aggregation: CostAggregation = config
            .get_config_serde_optional(&"cost_aggregation", &parent_key)?
            .unwrap_or_default();

        let ignore_unknown_weights = config
            .get_config_serde_optional(&"ignore_unknown_user_provided_weights", &parent_key)?
            .unwrap_or(true);

        let model = CostModelService {
            vehicle_rates: Arc::new(vehicle_rates),
            network_rates: Arc::new(network_rates),
            weights: Arc::new(weights),
            cost_aggregation,
            ignore_unknown_weights,
        };
        Ok(model)
    }
}
