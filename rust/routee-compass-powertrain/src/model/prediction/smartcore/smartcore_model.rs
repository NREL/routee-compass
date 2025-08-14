use crate::model::prediction::prediction_model::PredictionModel;
use routee_compass_core::model::{traversal::TraversalModelError, unit::EnergyRateUnit};
use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};
use std::{fs::File, path::Path};

pub struct SmartcoreModel {
    rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>>,
    energy_rate_unit: EnergyRateUnit,
}

impl PredictionModel for SmartcoreModel {
    fn predict(
        &self,
        feature_vector: &[f64],
    ) -> Result<(f64, EnergyRateUnit), TraversalModelError> {
        let x = DenseMatrix::from_2d_vec(&vec![feature_vector.to_vec()]).map_err(|e| {
            TraversalModelError::TraversalModelFailure(format!(
                "unable to set up prediction input vector: {e}"
            ))
        })?;
        let y = self.rf.predict(&x).map_err(|e| {
            TraversalModelError::TraversalModelFailure(format!(
                "failure running underlying Smartcore random forest energy prediction: {e}"
            ))
        })?;

        let energy_rate = y[0];
        Ok((energy_rate, self.energy_rate_unit))
    }
}

impl SmartcoreModel {
    pub fn new<P: AsRef<Path>>(
        routee_model_path: &P,
        energy_rate_unit: EnergyRateUnit,
    ) -> Result<Self, TraversalModelError> {
        let mut file = File::open(routee_model_path).map_err(|e| {
            TraversalModelError::BuildError(format!(
                "failure opening file {}: {}",
                routee_model_path.as_ref().to_string_lossy(),
                e
            ))
        })?;
        let rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
            bincode::serde::decode_from_std_read(&mut file, bincode::config::legacy()).map_err(
                |e| {
                    TraversalModelError::BuildError(format!(
                        "failure deserializing smartcore model {} due to {}",
                        routee_model_path.as_ref().to_str().unwrap_or_default(),
                        e
                    ))
                },
            )?;
        Ok(SmartcoreModel {
            rf,
            energy_rate_unit,
        })
    }
}
