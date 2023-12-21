use super::cost_model_service::CostModelService;
use crate::app::compass::config::{
    compass_configuration_error::CompassConfigurationError,
    compass_configuration_field::CompassConfigurationField,
    config_json_extension::ConfigJsonExtensions,
};
use routee_compass_core::model::cost::{
    cost_aggregation::CostAggregation, network::network_cost_mapping::NetworkCostMapping,
    vehicle::vehicle_cost_mapping::VehicleCostMapping,
};
use std::collections::{HashMap, HashSet};

pub struct CostModelBuilder {}

impl CostModelBuilder {
    pub fn build(
        &self,
        config: &serde_json::Value,
    ) -> Result<CostModelService, CompassConfigurationError> {
        let parent_key = CompassConfigurationField::Cost.to_string();
        let vehicle_mapping: Option<HashMap<String, VehicleCostMapping>> =
            config.get_config_serde_optional(&"vehicle_mapping", &parent_key)?;
        let network_mapping: Option<HashMap<String, NetworkCostMapping>> =
            config.get_config_serde_optional(&"network_mapping", &parent_key)?;

        // collect the complete set of potential state variable names from the keys of these mapping collections
        let default_state_variable_names: Option<HashSet<String>> =
            match (&vehicle_mapping, &network_mapping) {
                (None, None) => None,
                (None, Some(nm)) => Some(nm.keys().cloned().collect::<HashSet<_>>()),
                (Some(vm), None) => Some(vm.keys().cloned().collect::<HashSet<_>>()),
                (Some(vm), Some(nm)) => {
                    let key_set = vm
                        .keys()
                        .cloned()
                        .chain(nm.keys().cloned())
                        .collect::<HashSet<_>>();
                    Some(key_set)
                }
            };

        let default_cost_aggregation: Option<CostAggregation> =
            config.get_config_serde_optional(&"default_cost_aggregation", &parent_key)?;
        let model = CostModelService::new(
            vehicle_mapping,
            network_mapping,
            default_state_variable_names,
            default_cost_aggregation,
        )?;
        Ok(model)
    }
}
