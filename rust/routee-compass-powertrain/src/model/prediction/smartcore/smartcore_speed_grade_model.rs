use crate::model::prediction::prediction_model::PredictionModel;
use routee_compass_core::model::{
    traversal::TraversalModelError,
    unit::{AsF64, Convert, EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit},
};
use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};
use std::{borrow::Cow, fs::File, path::Path};

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
        let mut speed_value = Cow::Owned(speed);
        let mut grade_value = Cow::Owned(grade);
        speed_unit.convert(&mut speed_value, &self.speed_unit)?;
        grade_unit.convert(&mut grade_value, &self.grade_unit)?;
        let x = DenseMatrix::from_2d_vec(&vec![vec![speed_value.as_f64(), grade_value.as_f64()]])
            .map_err(|e| {
            TraversalModelError::TraversalModelFailure(format!(
                "unable to set up prediction input vector: {}",
                e
            ))
        })?;
        let y = self.rf.predict(&x).map_err(|e| {
            TraversalModelError::TraversalModelFailure(format!(
                "failure running underlying Smartcore random forest energy prediction: {}",
                e
            ))
        })?;

        let energy_rate = EnergyRate::from(y[0]);
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
        Ok(SmartcoreSpeedGradeModel {
            rf,
            speed_unit,
            grade_unit,
            energy_rate_unit,
        })
    }
}
