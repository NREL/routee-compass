use routee_compass_core::model::unit::{as_f64::AsF64, Energy};

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
    let percent_remaining_bounded = percent_remaining.max(0.0).min(100.0);
    100.0 - percent_remaining_bounded
}
