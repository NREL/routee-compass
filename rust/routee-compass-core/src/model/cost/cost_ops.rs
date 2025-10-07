use std::{collections::HashMap, sync::Arc};

use super::VehicleCostRate;

/// validates cost feature configuration. catches invalid combinations of found and missing
/// configuration arguments.
pub fn describe_cost_feature_configuration(
    name: &str,
    weights_mapping: Arc<HashMap<String, f64>>,
    vehicle_rate_mapping: Arc<HashMap<String, VehicleCostRate>>,
) -> String {
    let has_weight = weights_mapping.get(name);
    let has_rate = vehicle_rate_mapping.get(name);

    match (has_weight, has_rate) {
        (None, None) => format!("Feature '{name}' will not contribute to cost model has it has no weight or cost rate."),
        (None, Some(r)) => format!("Feature '{name}' was provided cost rate '{r}' but no weight, so it will not contribute to the cost model."),
        (Some(w), None) => format!("Feature '{name}' was provided weight {w} but no cost rate. The default rate is zero, so this feature will be zeroed out and will not contribute to the cost model."),
        (Some(w), Some(r)) => format!("Feature '{name}' was provided weight {w} and rate {r} and will contribute to the cost model."),
    }
}
