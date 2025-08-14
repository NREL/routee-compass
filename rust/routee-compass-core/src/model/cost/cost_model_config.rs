use std::collections::HashMap;

use crate::model::cost::{network::NetworkCostRate, CostAggregation, VehicleCostRate};
use serde::{Deserialize, Serialize};

/// configuration for a cost model set at app initialization time.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CostModelConfig {
    pub vehicle_rates: Option<HashMap<String, VehicleCostRate>>,
    pub network_rates: Option<HashMap<String, NetworkCostRate>>,
    pub weights: Option<HashMap<String, f64>>,
    pub cost_aggregation: Option<CostAggregation>,
    pub ignore_unknown_user_provided_weights: Option<bool>,
}

impl CostModelConfig {
    pub fn get_vehicle_rates(&self) -> HashMap<String, VehicleCostRate> {
        self.vehicle_rates.clone().unwrap_or_default()
    }
    pub fn get_network_rates(&self) -> HashMap<String, NetworkCostRate> {
        self.network_rates.clone().unwrap_or_default()
    }
    pub fn get_weights(&self) -> HashMap<String, f64> {
        self.weights.clone().unwrap_or_default()
    }
    pub fn get_cost_aggregation(&self) -> CostAggregation {
        self.cost_aggregation.unwrap_or_default()
    }
    pub fn get_ignore_policy(&self) -> bool {
        self.ignore_unknown_user_provided_weights.unwrap_or(true)
    }
}
