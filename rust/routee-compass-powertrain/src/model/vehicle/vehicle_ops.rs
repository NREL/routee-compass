use routee_compass_core::model::{
    state::{StateModel, StateModelError, StateVariable},
    unit::{AsF64, Energy},
};

/// updates the SOC feature for a vehicle type with a battery based on the
/// state, the energy delta, and max battery capacity.
///
/// note: if delta is negative, this is a regenerative braking event.
///
/// # Arguments
///
/// * `state`        - state to update
/// * `feature_name` - state feature for storing SOC value
/// * `delta`        - change in energy
/// * `max`          - maximum energy for this vehicle
/// * `state_model`  - provides API for interacting with state
pub fn update_soc_percent(
    state: &mut [StateVariable],
    feature_name: &str,
    delta: &Energy,
    max: &Energy,
    state_model: &StateModel,
) -> Result<(), StateModelError> {
    let start_soc = state_model.get_custom_f64(state, &feature_name.into())?;
    let start_battery = max.as_f64() * (start_soc / 100.0);
    let current_soc = soc_from_battery_and_delta(&Energy::from(start_battery), delta, max);
    state_model.set_custom_f64(state, &feature_name.into(), &current_soc)
}

/// a capacitated vehicle's state of charge (SOC) is the inverse of the
/// percent of fuel consumed with respect to the max energy. this function
/// allows scenarios where the current energy used exceeds the vehicle max
/// energy, for cases where route planning is not restricted by energy
/// capacity
///
/// # Arguments
///
/// * `remaining_battery` - amount of energy in battery
/// * `max_battery` - maximum energy storage capacity
///
/// # Returns
///
/// the remaining battery as a percentage [0, 100] %
pub fn as_soc_percent(remaining_battery: &Energy, max_battery: &Energy) -> f64 {
    let percent_remaining = (remaining_battery.as_f64() / max_battery.as_f64()) * 100.0;
    percent_remaining.clamp(0.0, 100.0)
}

/// a capacitated vehicle's state of charge (SOC) is the inverse of the
/// percent of fuel consumed with respect to the max energy. this function
/// allows scenarios where the current energy used exceeds the vehicle max
/// energy, for cases where route planning is not restricted by energy
/// capacity
///
/// # Arguments
///
/// * `start_battery` - amount of energy at start of edge
/// * `energy_used`   - energy used on edge. if negative, acts as regenerative braking event.
/// * `max_battery`   - maximum energy storage capacity
///
/// # Returns
///
/// the remaining battery as a percentage [0, 100] %
pub fn soc_from_battery_and_delta(
    start_battery: &Energy,
    energy_used: &Energy,
    max_battery: &Energy,
) -> f64 {
    let current_energy = *start_battery - *energy_used;
    let percent_remaining = (current_energy.as_f64() / max_battery.as_f64()) * 100.0;
    percent_remaining.clamp(0.0, 100.0)
}
