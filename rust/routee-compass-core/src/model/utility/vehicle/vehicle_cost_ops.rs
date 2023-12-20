use std::{collections::HashMap, sync::Arc};

use crate::model::{
    traversal::state::state_variable::StateVar,
    utility::{cost::Cost, utility_error::UtilityError},
};

use super::vehicle_utility_mapping::VehicleUtilityMapping;

pub fn calculate_vehicle_cost(
    prev_state: &[StateVar],
    next_state: &[StateVar],
    dimensions: &[(String, usize)],
    vehicle_mapping: Arc<HashMap<String, VehicleUtilityMapping>>,
) -> Result<Vec<Cost>, UtilityError> {
    let vehicle_costs = dimensions
        .iter()
        .map(|(name, idx)| {
            let prev_state_var = prev_state
                .get(*idx)
                .ok_or(UtilityError::StateIndexOutOfBounds(*idx, name.clone()))?;
            let next_state_var = next_state
                .get(*idx)
                .ok_or(UtilityError::StateIndexOutOfBounds(*idx, name.clone()))?;
            let delta: StateVar = *next_state_var - *prev_state_var;
            let mapping = vehicle_mapping
                .get(name)
                .ok_or(UtilityError::StateDimensionNotFound(name.clone()))?;
            let cost = mapping.map_value(delta);
            Ok(cost)
        })
        .collect::<Result<Vec<_>, UtilityError>>()?;
    Ok(vehicle_costs)
}
