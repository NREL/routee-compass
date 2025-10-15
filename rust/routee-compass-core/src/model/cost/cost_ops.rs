use std::{collections::HashMap, sync::Arc};

use crate::model::cost::network::NetworkCostRate;

use super::VehicleCostRate;

/// validates cost feature configuration. catches invalid combinations of found and missing
/// configuration arguments.
pub fn describe_cost_feature_configuration(
    name: &str,
    weights_mapping: Arc<HashMap<String, f64>>,
    vehicle_rate_mapping: Arc<HashMap<String, VehicleCostRate>>,
    network_rate_mapping: Arc<HashMap<String, NetworkCostRate>>,
) -> String {
    let has_weight = weights_mapping.get(name);
    let has_rate = vehicle_rate_mapping.get(name);
    let has_nrate = network_rate_mapping.get(name);

    match (has_weight, has_rate, has_nrate) {
        (None, None, None) => format!("Feature '{name}' will not contribute to cost model has it has no weight or cost rate."),
        (None, Some(v), None) => format!("Feature '{name}' was provided cost rate '{v}' but no weight, so it will not contribute to the cost model."),
        (Some(w), None, None) => format!("Feature '{name}' was provided weight {w} but no cost rate. The default rate is zero, so this feature will be zeroed out and will not contribute to the cost model."),
        (Some(w), Some(v), None) => format!("Feature '{name}' was provided weight {w} and vehicle rate {v} and will contribute to the cost model. No network costs were provided for this feature."),
        (None, None, Some(_)) => format!("Feature '{name}' with network rate will not contribute to cost model because it has no weight."),
        (None, Some(v), Some(_)) => format!("Feature '{name}' was provided vehicle cost rate '{v}' and network rate, but has no weight value, so it will not contribute to the cost model."),
        (Some(w), None, Some(_)) => format!("Feature '{name}' was provided weight {w} and network rate input, will contribute to the cost model."),
        (Some(w), Some(v), Some(_)) => format!("Feature '{name}' was provided weight {w}, vehicle rate {v} and network rate. It will contribute to the cost model."),
    }
}
