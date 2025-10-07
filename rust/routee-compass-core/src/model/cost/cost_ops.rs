use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use super::{network::NetworkCostRate, CostAggregation, CostModelError, VehicleCostRate};
use crate::model::{network::Edge, state::StateVariable, unit::Cost};

/// validates cost feature configuration. catches invalid combinations of found and missing
/// configuration arguments.
pub fn describe_cost_feature_configuration(
    name: &str,
    displayed_costs: &HashSet<String>,
    weights_mapping: Arc<HashMap<String, f64>>,
    vehicle_rate_mapping: Arc<HashMap<String, VehicleCostRate>>,
) -> String {
    let has_weight = weights_mapping.get(name);
    let has_rate = vehicle_rate_mapping.get(name);
    let has_display = displayed_costs.contains(name);

    match (has_weight, has_rate, has_display) {
        (None, None, true) => format!("Feature '{name}' will not contribute to cost model but its cost will be displayed in cost output."),
        (None, None, false) => format!("Feature '{name}' will not contribute to cost model and its cost will not be displayed in cost output."),
        (None, Some(r), true) => format!("Feature '{name}' was provided cost rate '{r}' but no weight, so it will not contribute to the cost model, but its cost will be displayed in cost output"),
        (None, Some(r), false) => format!("Feature '{name}' was provided cost rate '{r}' but no weight, so it will not contribute to the cost model. It will not be displayed in cost output"),
        (Some(w), None, true) => format!("Feature '{name}' was provided weight {w} but no cost rate. The default rate is zero, so this feature will be zeroed out. It has been configured for display in the cost model."),
        (Some(w), None, false) => format!("Feature '{name}' was provided weight {w} but no cost rate. The default rate is zero, so this feature will be zeroed out. It has been configured to not display in the cost model."),
        (Some(w), Some(r), true) => format!("Feature '{name}' was provided weight {w} and rate {r} and is configured to display in the cost model output"),
        (Some(w), Some(r), false) => format!("Feature '{name}' was provided weight {w} and rate {r}. It was configured not to display in the cost model output but this will be overriden."),
    }
}

/// steps through each state variable and assigns vehicle costs related to that variable
/// due to an edge access + traversal event.
///
/// # Arguments
/// * `prev_state` - the state before beginning the traversal
/// * `next_state` - the state after traversal
/// * `indices`    - feature names and corresponding state indices
pub fn calculate_vehicle_costs(
    state_sequence: (&[StateVariable], &[StateVariable]),
    indices: &[(String, usize)],
    weights: &[f64],
    rates: &[VehicleCostRate],
    cost_aggregation: &CostAggregation,
) -> Result<Cost, CostModelError> {
    let (prev_state, next_state) = state_sequence;
    let mut costs: Vec<(&str, Cost)> = vec![];
    for (name, state_idx) in indices.iter() {
        // compute delta
        let prev_state_var = prev_state
            .get(*state_idx)
            .ok_or_else(|| CostModelError::StateIndexOutOfBounds(*state_idx, name.clone()))?;
        let next_state_var = next_state
            .get(*state_idx)
            .ok_or_else(|| CostModelError::StateIndexOutOfBounds(*state_idx, name.clone()))?;
        let delta: StateVariable = *next_state_var - *prev_state_var;

        // collect weight and vehicle cost rate
        let mapping = rates.get(*state_idx).ok_or_else(|| {
            CostModelError::CostVectorOutOfBounds(*state_idx, String::from("vehicle_rates"))
        })?;
        let weight = weights.get(*state_idx).ok_or_else(|| {
            CostModelError::CostVectorOutOfBounds(*state_idx, String::from("weights"))
        })?;

        // compute cost

        if let Some(delta_cost) = mapping.map_value(delta) {
            costs.push((name, delta_cost * weight));
        }
    }

    cost_aggregation.aggregate(&costs)
}

pub fn calculate_network_traversal_costs(
    state_sequence: (&[StateVariable], &[StateVariable]),
    edge: &Edge,
    indices: &[(String, usize)],
    weights: &[f64],
    rates: &[NetworkCostRate],
    cost_aggregation: &CostAggregation,
) -> Result<Cost, CostModelError> {
    let (prev_state, next_state) = state_sequence;
    let mut costs: Vec<(&str, Cost)> = vec![];
    for (name, state_idx) in indices.iter() {
        let prev_state_var = prev_state
            .get(*state_idx)
            .ok_or_else(|| CostModelError::StateIndexOutOfBounds(*state_idx, name.clone()))?;
        let next_state_var = next_state
            .get(*state_idx)
            .ok_or_else(|| CostModelError::StateIndexOutOfBounds(*state_idx, name.clone()))?;

        // determine weight and access cost for this state feature
        let weight = weights.get(*state_idx).ok_or_else(|| {
            CostModelError::CostVectorOutOfBounds(*state_idx, String::from("weights"))
        })?;
        let rate = rates.get(*state_idx).ok_or_else(|| {
            CostModelError::CostVectorOutOfBounds(*state_idx, String::from("network_cost_rate"))
        })?;
        let access_cost = rate.traversal_cost(*prev_state_var, *next_state_var, edge)?;
        let cost = access_cost * weight;
        costs.push((name, cost));
    }

    cost_aggregation.aggregate(&costs)
}
