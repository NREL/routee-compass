use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};

use anyhow::Result;
use pyo3::prelude::*;

use crate::prototype::graph::Link;

// scale the energy by this factor to make it an integer
pub const ROUTEE_SCALE_FACTOR: f64 = 100000.0;

pub const CENTIMETERS_TO_MILES: f64 = 6.213712e-6;

#[pyclass]
#[derive(Clone, Copy, Debug)]
pub struct VehicleParameters {
    pub weight_lbs: u32,
    pub height_inches: u16,
    pub width_inches: u16,
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
pub fn build_routee_cost_function(model_file_path: &str) -> Result<impl Fn(&Link) -> u32> {
    let rf_binary = std::fs::read(model_file_path)?;

    let rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
        bincode::deserialize(&rf_binary)?;

    Ok(move |link: &Link| {
        let distance_miles = link.distance_centimeters as f64 * CENTIMETERS_TO_MILES;
        let time_hours = link.time_seconds as f64 / 3600.0;
        let speed_mph = distance_miles / time_hours;
        let grade = link.grade as f64;

        let features = vec![vec![speed_mph, grade]];

        let x = DenseMatrix::from_2d_vec(&features);
        let energy_per_mile = rf.predict(&x).unwrap()[0];

        let energy = energy_per_mile * distance_miles;
        let scaled_energy = energy * ROUTEE_SCALE_FACTOR;
        scaled_energy as u32
    })
}
