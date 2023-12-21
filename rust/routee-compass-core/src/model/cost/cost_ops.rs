use super::{
    network::network_cost_mapping::NetworkUtilityMapping,
    vehicle::vehicle_cost_mapping::VehicleUtilityMapping,
};
use crate::model::{
    cost::cost_error::CostError, property::edge::Edge, traversal::state::state_variable::StateVar,
    unit::Cost,
};
use std::{collections::HashMap, sync::Arc};

pub fn calculate_vehicle_costs(
    prev_state: &[StateVar],
    next_state: &[StateVar],
    dimensions: &[(String, usize)],
    mappings: Arc<HashMap<String, VehicleUtilityMapping>>,
) -> Result<Vec<Cost>, CostError> {
    let costs = dimensions
        .iter()
        .map(|(name, idx)| {
            let prev_state_var = prev_state
                .get(*idx)
                .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
            let next_state_var = next_state
                .get(*idx)
                .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
            let delta: StateVar = *next_state_var - *prev_state_var;
            let mapping = mappings
                .get(name)
                .ok_or(CostError::StateDimensionNotFound(name.clone()))?;
            let cost = mapping.map_value(delta);
            Ok(cost)
        })
        .collect::<Result<Vec<_>, CostError>>()?;
    Ok(costs)
}

pub fn calculate_network_traversal_costs(
    prev_state: &[StateVar],
    next_state: &[StateVar],
    edge: &Edge,
    dimensions: &[(String, usize)],
    mappings: Arc<HashMap<String, NetworkUtilityMapping>>,
) -> Result<Vec<Cost>, CostError> {
    let costs = dimensions
        .iter()
        .map(|(name, idx)| match mappings.get(name) {
            None => Ok(Cost::ZERO),
            Some(m) => {
                let prev_state_var = prev_state
                    .get(*idx)
                    .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let next_state_var = next_state
                    .get(*idx)
                    .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                m.traversal_cost(*prev_state_var, *next_state_var, edge)
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
    dimensions: &[(String, usize)],
    mappings: Arc<HashMap<String, NetworkUtilityMapping>>,
) -> Result<Vec<Cost>, CostError> {
    let costs = dimensions
        .iter()
        .map(|(name, idx)| match mappings.get(name) {
            None => Ok(Cost::ZERO),
            Some(m) => {
                let prev_state_var = prev_state
                    .get(*idx)
                    .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let next_state_var = next_state
                    .get(*idx)
                    .ok_or(CostError::StateIndexOutOfBounds(*idx, name.clone()))?;
                m.access_cost(*prev_state_var, *next_state_var, prev_edge, next_edge)
            }
        })
        .collect::<Result<Vec<_>, CostError>>()?;
    Ok(costs)
}
