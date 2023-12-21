use super::{
    network::network_utility_mapping::NetworkUtilityMapping,
    vehicle::vehicle_utility_mapping::VehicleUtilityMapping,
};
use crate::model::{
    property::edge::Edge, traversal::state::state_variable::StateVar, unit::Cost,
    utility::utility_error::UtilityError,
};
use std::{collections::HashMap, sync::Arc};

pub fn calculate_vehicle_costs(
    prev_state: &[StateVar],
    next_state: &[StateVar],
    dimensions: &[(String, usize)],
    mappings: Arc<HashMap<String, VehicleUtilityMapping>>,
) -> Result<Vec<Cost>, UtilityError> {
    let costs = dimensions
        .iter()
        .map(|(name, idx)| {
            let prev_state_var = prev_state
                .get(*idx)
                .ok_or(UtilityError::StateIndexOutOfBounds(*idx, name.clone()))?;
            let next_state_var = next_state
                .get(*idx)
                .ok_or(UtilityError::StateIndexOutOfBounds(*idx, name.clone()))?;
            let delta: StateVar = *next_state_var - *prev_state_var;
            let mapping = mappings
                .get(name)
                .ok_or(UtilityError::StateDimensionNotFound(name.clone()))?;
            let cost = mapping.map_value(delta);
            Ok(cost)
        })
        .collect::<Result<Vec<_>, UtilityError>>()?;
    Ok(costs)
}

pub fn calculate_network_traversal_costs(
    prev_state: &[StateVar],
    next_state: &[StateVar],
    edge: &Edge,
    dimensions: &[(String, usize)],
    mappings: Arc<HashMap<String, NetworkUtilityMapping>>,
) -> Result<Vec<Cost>, UtilityError> {
    let costs = dimensions
        .iter()
        .map(|(name, idx)| match mappings.get(name) {
            None => Ok(Cost::ZERO),
            Some(m) => {
                let prev_state_var = prev_state
                    .get(*idx)
                    .ok_or(UtilityError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let next_state_var = next_state
                    .get(*idx)
                    .ok_or(UtilityError::StateIndexOutOfBounds(*idx, name.clone()))?;
                m.traversal_cost(*prev_state_var, *next_state_var, edge)
            }
        })
        .collect::<Result<Vec<_>, UtilityError>>()?;
    Ok(costs)
}

pub fn calculate_network_access_costs(
    prev_state: &[StateVar],
    next_state: &[StateVar],
    prev_edge: &Edge,
    next_edge: &Edge,
    dimensions: &[(String, usize)],
    mappings: Arc<HashMap<String, NetworkUtilityMapping>>,
) -> Result<Vec<Cost>, UtilityError> {
    let costs = dimensions
        .iter()
        .map(|(name, idx)| match mappings.get(name) {
            None => Ok(Cost::ZERO),
            Some(m) => {
                let prev_state_var = prev_state
                    .get(*idx)
                    .ok_or(UtilityError::StateIndexOutOfBounds(*idx, name.clone()))?;
                let next_state_var = next_state
                    .get(*idx)
                    .ok_or(UtilityError::StateIndexOutOfBounds(*idx, name.clone()))?;
                m.access_cost(*prev_state_var, *next_state_var, prev_edge, next_edge)
            }
        })
        .collect::<Result<Vec<_>, UtilityError>>()?;
    Ok(costs)
}
