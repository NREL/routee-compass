use std::path::Path;

use crate::model::prediction::prediction_model::PredictionModel;
use routee_compass_core::{
    model::traversal::TraversalModelError,
    model::unit::{as_f64::AsF64, EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit},
};
use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};

pub struct SmartcoreSpeedGradeModel {
    rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>>,
    speed_unit: SpeedUnit,
    grade_unit: GradeUnit,
    energy_rate_unit: EnergyRateUnit,
}

impl PredictionModel for SmartcoreSpeedGradeModel {
    fn predict(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
    ) -> Result<(EnergyRate, EnergyRateUnit), TraversalModelError> {
        let (speed, speed_unit) = speed;
        let (grade, grade_unit) = grade;
        let speed_value = speed_unit.convert(&speed, &self.speed_unit).as_f64();
        let grade_value = grade_unit.convert(&grade, &self.grade_unit).as_f64();
        let x = DenseMatrix::from_2d_vec(&vec![vec![speed_value, grade_value]]);
        let y = self.rf.predict(&x).map_err(|e| {
            TraversalModelError::TraversalModelFailure(format!(
                "failure running underlying Smartcore random forest energy prediction: {}",
                e
            ))
        })?;

        let energy_rate = EnergyRate::new(y[0]);
        Ok((energy_rate, self.energy_rate_unit))
    }
}

impl SmartcoreSpeedGradeModel {
    pub fn new<P: AsRef<Path>>(
        routee_model_path: &P,
        speed_unit: SpeedUnit,
        grade_unit: GradeUnit,
        energy_rate_unit: EnergyRateUnit,
    ) -> Result<Self, TraversalModelError> {
        // Load random forest binary file
        let rf_binary = std::fs::read(routee_model_path).map_err(|e| {
            TraversalModelError::BuildError(format!(
                "failure reading smartcore binary text file {} due to {}",
                routee_model_path.as_ref().to_str().unwrap_or_default(),
                e
            ))
        })?;
        let rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
            bincode::deserialize(&rf_binary).map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failure deserializing smartcore model {} due to {}",
                    routee_model_path.as_ref().to_str().unwrap_or_default(),
                    e
                ))
            })?;
        Ok(SmartcoreSpeedGradeModel {
            rf,
            speed_unit,
            grade_unit,
            energy_rate_unit,
        })
    }
}
