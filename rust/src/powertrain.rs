use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};

use anyhow::Result;

use crate::graph::Link;

// scale the energy by this factor to make it an integer
pub const ROUTEE_SCALE_FACTOR: f64 = 100000.0;

/// a function that loads the routee-powertrain random forest model
/// and then returns a closure that takes a link and returns the energy over that link
pub fn build_routee_cost_function(model_file_path: &str) -> Result<impl Fn(&Link) -> u32> {
    let rf_binary = std::fs::read(model_file_path)?;

    let rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
        bincode::deserialize(&rf_binary)?;

    Ok(move |link: &Link| {
        let distance_miles = link.distance as f64 * 0.0006213712;
        let time_hours = link.time as f64 / 3600.0;
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
