use super::{
    network::network_cost_rate::NetworkCostRate, vehicle::vehicle_cost_rate::VehicleCostRate,
};
use crate::model::{
    cost::cost_error::CostError, property::edge::Edge, traversal::state::state_variable::StateVar,
    unit::Cost,
};
use std::{collections::HashMap, sync::Arc};

pub fn calculate_vehicle_costs<'a>(
    prev_state: &'a [StateVar],
    next_state: &'a [StateVar],
    state_variable_indices: &'a [(String, usize)],
    state_variable_coefficients: Arc<HashMap<String, f64>>,
    rates: Arc<HashMap<String, VehicleCostRate>>,
) -> Result<Vec<(&'a String, Cost)>, CostError> {
    let costs = state_variable_indices
        .iter()
        .map(|(name, idx)| {
            let prev_state_var = prev_state
                .get(*idx)
                .ok_or_else(|| CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
            let next_state_var = next_state
                .get(*idx)
                .ok_or_else(|| CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
            let delta: StateVar = *next_state_var - *prev_state_var;
            let mapping = rates
                .get(name)
                .ok_or_else(|| CostError::StateVariableNotFound(name.clone()))?;
            let coefficient = state_variable_coefficients.get(name).unwrap_or(&1.0);
            let delta_cost = mapping.map_value(delta);
            let cost = delta_cost * coefficient;
            Ok((name, cost))
        })
        .collect::<Result<Vec<_>, CostError>>()?;
    Ok(costs)
}

pub fn calculate_network_traversal_costs<'a>(
    prev_state: &'a [StateVar],
    next_state: &'a [StateVar],
    edge: &'a Edge,
    state_variable_indices: &'a [(String, usize)],
    state_variable_coefficients: Arc<HashMap<String, f64>>,
    rates: Arc<HashMap<String, NetworkCostRate>>,
) -> Result<Vec<(&'a String, Cost)>, CostError> {
    let costs = state_variable_indices
        .iter()
        .map(|(name, idx)| match rates.get(name) {
            None => Ok((name, Cost::ZERO)),
            Some(m) => {
                let prev_state_var = prev_state
                    .get(*idx)
                    .ok_or_else(|| CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let next_state_var = next_state
                    .get(*idx)
                    .ok_or_else(|| CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let coefficient = state_variable_coefficients.get(name).unwrap_or(&1.0);
                let traversal_cost = m.traversal_cost(*prev_state_var, *next_state_var, edge)?;
                let cost = traversal_cost * coefficient;
                Ok((name, cost))
            }
        })
        .collect::<Result<Vec<_>, CostError>>()?;
    Ok(costs)
}

pub fn calculate_network_access_costs<'a>(
    prev_state: &'a [StateVar],
    next_state: &'a [StateVar],
    prev_edge: &'a Edge,
    next_edge: &'a Edge,
    state_variable_indices: &'a [(String, usize)],
    state_variable_coefficients: Arc<HashMap<String, f64>>,
    rates: Arc<HashMap<String, NetworkCostRate>>,
) -> Result<Vec<(&'a String, Cost)>, CostError> {
    let costs = state_variable_indices
        .iter()
        .map(|(name, idx)| match rates.get(name) {
            None => Ok((name, Cost::ZERO)),
            Some(m) => {
                let prev_state_var = prev_state
                    .get(*idx)
                    .ok_or_else(|| CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let next_state_var = next_state
                    .get(*idx)
                    .ok_or_else(|| CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let access_cost =
                    m.access_cost(*prev_state_var, *next_state_var, prev_edge, next_edge)?;
                let coefficient = state_variable_coefficients.get(name).unwrap_or(&1.0);
                let cost = access_cost * coefficient;
                Ok((name, cost))
            }
        })
        .collect::<Result<Vec<_>, CostError>>()?;
    Ok(costs)
}
