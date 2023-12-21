use super::{
    network::network_cost_rate::NetworkCostRate, vehicle::vehicle_cost_rate::VehicleCostRate,
};
use crate::model::{
    cost::cost_error::CostError, property::edge::Edge, traversal::state::state_variable::StateVar,
    unit::Cost,
};
use std::{collections::HashMap, sync::Arc};

pub fn calculate_vehicle_costs(
    prev_state: &[StateVar],
    next_state: &[StateVar],
    state_variable_indices: &[(String, usize)],
    state_variable_coefficients: Arc<HashMap<String, f64>>,
    rates: Arc<HashMap<String, VehicleCostRate>>,
) -> Result<Vec<Cost>, CostError> {
    let costs = state_variable_indices
        .iter()
        .map(|(name, idx)| {
            let prev_state_var = prev_state
                .get(*idx)
                .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
            let next_state_var = next_state
                .get(*idx)
                .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
            let delta: StateVar = *next_state_var - *prev_state_var;
            let mapping = rates
                .get(name)
                .ok_or(CostError::StateDimensionNotFound(name.clone()))?;
            let cost = mapping.map_value(delta);

            // apply coefficient if provided
            match state_variable_coefficients.get(name) {
                Some(coefficient) => Ok(cost * coefficient),
                None => Ok(cost),
            }
        })
        .collect::<Result<Vec<_>, CostError>>()?;
    Ok(costs)
}

pub fn calculate_network_traversal_costs(
    prev_state: &[StateVar],
    next_state: &[StateVar],
    edge: &Edge,
    state_variable_indices: &[(String, usize)],
    state_variable_coefficients: Arc<HashMap<String, f64>>,
    rates: Arc<HashMap<String, NetworkCostRate>>,
) -> Result<Vec<Cost>, CostError> {
    let costs = state_variable_indices
        .iter()
        .map(|(name, idx)| match rates.get(name) {
            None => Ok(Cost::ZERO),
            Some(m) => {
                let prev_state_var = prev_state
                    .get(*idx)
                    .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let next_state_var = next_state
                    .get(*idx)
                    .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let cost = m.traversal_cost(*prev_state_var, *next_state_var, edge)?;
                // apply coefficient if provided
                match state_variable_coefficients.get(name) {
                    Some(coefficient) => Ok(cost * coefficient),
                    None => Ok(cost),
                }
            }
        })
        .collect::<Result<Vec<_>, CostError>>()?;
    Ok(costs)
}

pub fn calculate_network_access_costs(
    prev_state: &[StateVar],
    next_state: &[StateVar],
    prev_edge: &Edge,
    next_edge: &Edge,
    state_variable_indices: &[(String, usize)],
    state_variable_coefficients: Arc<HashMap<String, f64>>,
    rates: Arc<HashMap<String, NetworkCostRate>>,
) -> Result<Vec<Cost>, CostError> {
    let costs = state_variable_indices
        .iter()
        .map(|(name, idx)| match rates.get(name) {
            None => Ok(Cost::ZERO),
            Some(m) => {
                let prev_state_var = prev_state
                    .get(*idx)
                    .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let next_state_var = next_state
                    .get(*idx)
                    .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let cost = m.access_cost(*prev_state_var, *next_state_var, prev_edge, next_edge)?;
                // apply coefficient if provided
                match state_variable_coefficients.get(name) {
                    Some(coefficient) => Ok(cost * coefficient),
                    None => Ok(cost),
                }
            }
        })
        .collect::<Result<Vec<_>, CostError>>()?;
    Ok(costs)
}
