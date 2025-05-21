use crate::model::prediction::prediction_model::PredictionModel;
use routee_compass_core::model::{
    state::{InputFeature, StateModel, StateVariable},
    traversal::TraversalModelError,
    unit::{AsF64, Convert, EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit},
};
use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};
use std::{borrow::Cow, fs::File, path::Path};

pub struct SmartcoreModel {
    rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>>,
    input_features: Vec<(String, InputFeature)>,
    energy_rate_unit: EnergyRateUnit,
}

impl PredictionModel for SmartcoreModel {
    fn predict(
        &self,
        input_features: &[(String, InputFeature)],
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(EnergyRate, EnergyRateUnit), TraversalModelError> {
        let mut feature_vector: Vec<f64> = Vec::new();
        for (name, input_feature) in input_features {
            let state_variable_f64: f64 = match input_feature {
                InputFeature::Speed(unit) => {
                    let (speed, _speed_unit) = state_model.get_speed(state, name, unit.as_ref())?;
                    speed.as_f64()
                }
                InputFeature::Grade(unit) => {
                    let (grade, _grade_unit) = state_model.get_grade(state, name, unit.as_ref())?;
                    grade.as_f64()
                }
                InputFeature::Custom { r#type: _, unit: _ } => {
                    state_model.get_custom_f64(state, name)?
                }
                _ => {
                    return Err(TraversalModelError::TraversalModelFailure(format!(
                        "got an unexpected input feature in the smartcore model prediction {}",
                        input_feature
                    )))
                }
            };
            feature_vector.push(state_variable_f64);
        }
        let x = DenseMatrix::from_2d_vec(&vec![feature_vector]).map_err(|e| {
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

impl SmartcoreModel {
    pub fn new<P: AsRef<Path>>(
        routee_model_path: &P,
        input_features: Vec<(String, InputFeature)>,
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
            input_features,
            energy_rate_unit,
        })
    }
}
