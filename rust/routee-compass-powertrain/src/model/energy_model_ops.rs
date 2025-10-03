use routee_compass_core::model::{
    network::EdgeId,
    traversal::{default::turn_delays::EdgeHeading, TraversalModelError},
    unit::UnitError,
};
use uom::{
    si::f64::{Energy, Ratio},
    ConstZero,
};

/// used as a replacement for zero in energy calculations
/// where zero is not a valid value.
pub const SAFE_MIN_ENERGY: f64 = 1e-9;
pub const MIN_SOC_PERCENT: f64 = 0.0;
pub const MAX_SOC_PERCENT: f64 = 100.0;

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
    start_soc: Ratio,
    energy_consumption: Energy,
    maximum_energy: Energy,
) -> Result<Ratio, UnitError> {
    let start_battery: Energy = maximum_energy * start_soc;
    let current_energy: Energy = start_battery - energy_consumption;
    let percent_remaining: Ratio = soc_from_energy(current_energy, maximum_energy)?;
    Ok(percent_remaining)
}

pub fn soc_from_energy(energy: Energy, maximum_energy: Energy) -> Result<Ratio, UnitError> {
    if maximum_energy == Energy::ZERO {
        return Err(UnitError::ZeroDivisionError(
            "maximum energy cannot be zero".to_string(),
        ));
    }
    let soc = energy / maximum_energy;
    let min = Ratio::new::<uom::si::ratio::percent>(MIN_SOC_PERCENT);
    let max = Ratio::new::<uom::si::ratio::percent>(MAX_SOC_PERCENT);
    if soc < min {
        Ok(min)
    } else if soc > max {
        Ok(max)
    } else {
        Ok(soc)
    }
}

pub fn get_query_start_soc(
    query: &serde_json::Value,
) -> Result<Option<Ratio>, TraversalModelError> {
    let starting_soc_percent = match query.get("starting_soc_percent".to_string()) {
        Some(soc_string) => soc_string.as_f64().ok_or_else(|| {
            TraversalModelError::BuildError(
                "Expected 'starting_soc_percent' value to be numeric".to_string(),
            )
        })?,
        None => return Ok(None),
    };
    if !(0.0..=100.0).contains(&starting_soc_percent) {
        return Err(TraversalModelError::BuildError(
            "Expected 'starting_soc_percent' value to be between 0 and 100".to_string(),
        ));
    }
    let starting_soc = Ratio::new::<uom::si::ratio::percent>(starting_soc_percent);
    Ok(Some(starting_soc))
}

/// inspect the user query for a starting_soc_percent value. if provided, compute the
/// energy value to use as the starting energy for the vehicle. if not provided, return None.
pub fn get_query_start_energy(
    query: &serde_json::Value,
    capacity: Energy,
) -> Result<Option<Energy>, TraversalModelError> {
    let starting_soc = match get_query_start_soc(query)? {
        Some(soc) => soc,
        None => return Ok(None),
    };
    let starting_battery_energy = starting_soc * capacity;
    Ok(Some(starting_battery_energy))
}

/// look up the grade from the grade table
pub fn get_grade(
    grade_table: &Option<Box<[Ratio]>>,
    edge_id: EdgeId,
) -> Result<Ratio, TraversalModelError> {
    match grade_table {
        None => Ok(Ratio::ZERO),
        Some(gt) => {
            let grade: &Ratio = gt.get(edge_id.as_usize()).ok_or_else(|| {
                TraversalModelError::TraversalModelFailure(format!(
                    "missing index {edge_id} from grade table"
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
            "missing index {edge_id} from headings table"
        ))
    })?;
    Ok(*heading)
}

#[cfg(test)]
mod test {
    use super::update_soc_percent;

    use uom::si::f64::{Energy, Ratio};

    #[test]
    fn test_update_soc_percent() {
        let start_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        let maximum_energy = Energy::new::<uom::si::energy::kilowatt_hour>(100.0);
        let energy_consumption = Energy::new::<uom::si::energy::kilowatt_hour>(20.0);
        let result = update_soc_percent(start_soc, energy_consumption, maximum_energy)
            .expect("failed to update");
        let expected_soc = Ratio::new::<uom::si::ratio::percent>(80.0);
        assert_eq!(
            result, expected_soc,
            "should have used 20/100 = 20% of the soc"
        )
    }

    #[test]
    fn test_update_soc_no_underflow() {
        let start_soc = Ratio::new::<uom::si::ratio::percent>(50.0);
        let maximum_energy = Energy::new::<uom::si::energy::kilowatt_hour>(100.0);
        let energy_consumption = Energy::new::<uom::si::energy::kilowatt_hour>(70.0);
        let result = update_soc_percent(start_soc, energy_consumption, maximum_energy)
            .expect("failed to update");
        let expected_soc = Ratio::new::<uom::si::ratio::percent>(0.0);
        assert_eq!(result, expected_soc, "should prevent soc underflow")
    }

    #[test]
    fn test_update_soc_no_overflow() {
        let start_soc = Ratio::new::<uom::si::ratio::percent>(100.0);
        let maximum_energy = Energy::new::<uom::si::energy::kilowatt_hour>(100.0);
        let energy_consumption = Energy::new::<uom::si::energy::kilowatt_hour>(-70.0);
        let result = update_soc_percent(start_soc, energy_consumption, maximum_energy)
            .expect("failed to update");
        assert_eq!(result, start_soc, "should prevent soc overflow")
    }
}
