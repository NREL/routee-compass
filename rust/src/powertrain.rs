use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};

use anyhow::Result;
use pyo3::prelude::*;

use crate::{
    graph::Link,
    time_of_day_speed::{DayOfWeek, SecondOfDay, TimeOfDaySpeeds},
};

// scale the energy by this factor to make it an integer
pub const ROUTEE_SCALE_FACTOR: f64 = 100000.0;

pub const CENTIMETERS_TO_MILES: f64 = 6.213712e-6;

#[pyclass]
#[derive(Clone, Copy, Debug)]
pub struct VehicleParameters {
    #[pyo3(get)]
    pub weight_lbs: u32,
    #[pyo3(get)]
    pub height_inches: u16,
    #[pyo3(get)]
    pub width_inches: u16,
    #[pyo3(get)]
    pub length_inches: u16,
}

#[pymethods]
impl VehicleParameters {
    #[new]
    pub fn new(weight_lbs: u32, height_inches: u16, width_inches: u16, length_inches: u16) -> Self {
        VehicleParameters {
            weight_lbs,
            height_inches,
            width_inches,
            length_inches,
        }
    }
}

/// a function that loads the routee-powertrain random forest model
/// and then returns a closure that takes a link and returns the energy over that link
pub fn build_routee_cost_function(
    model_file_path: &str,
    vehicle_params: Option<VehicleParameters>,
) -> Result<impl Fn(&Link) -> u32> {
    let rf_binary = std::fs::read(model_file_path)?;

    let rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
        bincode::deserialize(&rf_binary)?;

    Ok(move |link: &Link| {
        let distance_miles = link.distance_centimeters as f64 * CENTIMETERS_TO_MILES;
        let time_hours = link.time_seconds() as f64 / 3600.0;
        let speed_mph = distance_miles / time_hours;
        let grade = link.grade as f64;

        let features = match vehicle_params {
            Some(params) => {
                let mass_kg = params.weight_lbs as f64 * 0.453592;
                vec![vec![speed_mph, grade, mass_kg]]
            }
            None => vec![vec![speed_mph, grade]],
        };

        let x = DenseMatrix::from_2d_vec(&features);
        let energy_per_mile = rf.predict(&x).unwrap()[0];

        let energy = energy_per_mile * distance_miles;
        let scaled_energy = energy * ROUTEE_SCALE_FACTOR;
        scaled_energy as u32
    })
}

pub fn build_routee_cost_function_with_tods(
    model_file_path: &str,
    tods: TimeOfDaySpeeds,
    sod: SecondOfDay,
    dow: DayOfWeek,
    vehicle_params: Option<VehicleParameters>,
) -> Result<impl Fn(&Link) -> u32> {
    let rf_binary = std::fs::read(model_file_path)?;

    let rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
        bincode::deserialize(&rf_binary)?;

    Ok(move |link: &Link| {
        let distance_miles = link.distance_centimeters as f64 * CENTIMETERS_TO_MILES;
        let mut modifier = 1.0;
        if let Some(profile_id) = link.week_profile_ids[dow] {
            modifier = tods.get_modifier_by_second_of_day(profile_id, sod);
        }

        let time_hours = link.time_seconds() as f64 / 3600.0;
        let speed_mph = (distance_miles / time_hours) * modifier;
        let grade = link.grade as f64;

        let features = match vehicle_params {
            Some(params) => vec![vec![speed_mph, grade, params.weight_lbs as f64 * 0.453592]],
            None => vec![vec![speed_mph, grade]],
        };

        let x = DenseMatrix::from_2d_vec(&features);
        let energy_per_mile = rf.predict(&x).unwrap()[0];

        let energy = energy_per_mile * distance_miles;
        let scaled_energy = energy * ROUTEE_SCALE_FACTOR;
        scaled_energy as u32
    })
}
