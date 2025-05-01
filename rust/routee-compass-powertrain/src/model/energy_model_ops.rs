use std::borrow::Cow;

use routee_compass_core::model::{
    access::default::turn_delays::EdgeHeading,
    network::edge_id::EdgeId,
    traversal::TraversalModelError,
    unit::{AsF64, Convert, Energy, EnergyUnit, Grade, UnitError},
};

/// used as a replacement for zero in energy calculations
/// where zero is not a valid value.
pub const SAFE_MIN_ENERGY: f64 = 1e-9;

/// updates the SOC feature for a vehicle type with a battery based on the
/// state, the energy delta, and max battery capacity.
///
/// note: if delta is negative, this is a regenerative braking event.
///
/// # Arguments
///
/// * `delta`        - change in energy
/// * `maximum_energy` - maximum energy for this vehicle
pub fn update_soc_percent(
    start_soc: &f64,
    energy_consumption: (&Energy, &EnergyUnit),
    maximum_energy: (&Energy, &EnergyUnit),
) -> Result<f64, UnitError> {
    let (delta_energy, delta_unit) = energy_consumption;
    let (max_energy, max_unit) = maximum_energy;
    let mut delta = Cow::Borrowed(delta_energy);
    delta_unit.convert(&mut delta, max_unit)?;
    let start_battery = Energy::from(max_energy.as_f64() * (start_soc / 100.0));
    let current_energy = start_battery - *delta_energy;
    let percent_remaining = (current_energy.as_f64() / max_energy.as_f64()) * 100.0;
    Ok(percent_remaining.clamp(0.0, 100.0))
}

pub fn soc_from_energy(
    energy: (&Energy, &EnergyUnit),
    maximum_energy: (&Energy, &EnergyUnit),
) -> Result<f64, String> {
    let (e, eu) = energy;
    let (me, meu) = maximum_energy;
    let mut e_mut = Cow::Borrowed(e);
    eu.convert(&mut e_mut, meu).map_err(|e| format!("while converting energy to soc, failed to match energy units of max value and current value: {}", e))?;
    let energy = e_mut.into_owned();
    if energy < Energy::ZERO {
        return Ok(0.0);
    }
    if me < &energy {
        return Err(format!(
            "vehicle energy {} is greater than battery capacity {}",
            energy, me
        ));
    }
    let soc = energy.as_f64() / me.as_f64();
    Ok(soc)
}

/// look up the grade from the grade table
pub fn get_grade(
    grade_table: &Option<Box<[Grade]>>,
    edge_id: EdgeId,
) -> Result<Grade, TraversalModelError> {
    match grade_table {
        None => Ok(Grade::ZERO),
        Some(gt) => {
            let grade: &Grade = gt.get(edge_id.as_usize()).ok_or_else(|| {
                TraversalModelError::TraversalModelFailure(format!(
                    "missing index {} from grade table",
                    edge_id
                ))
            })?;
            Ok(*grade)
        }
    }
}

/// lookup up the edge heading from the headings table
pub fn get_headings(
    headings_table: &[EdgeHeading],
    edge_id: EdgeId,
) -> Result<EdgeHeading, TraversalModelError> {
    let heading: &EdgeHeading = headings_table.get(edge_id.as_usize()).ok_or_else(|| {
        TraversalModelError::TraversalModelFailure(format!(
            "missing index {} from headings table",
            edge_id
        ))
    })?;
    Ok(*heading)
}
