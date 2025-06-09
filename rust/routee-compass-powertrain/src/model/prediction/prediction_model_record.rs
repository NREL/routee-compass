use crate::model::fieldname;

use super::{
    interpolation::InterpolationModel, model_type::ModelType, prediction_model_ops,
    smartcore::SmartcoreModel, PredictionModel, PredictionModelConfig,
};
use routee_compass_core::model::{
    state::{InputFeature, StateModel, StateVariable},
    traversal::TraversalModelError,
    unit::{AsF64, DistanceUnit, Energy, EnergyRate, EnergyRateUnit, EnergyUnit},
};
use std::sync::Arc;

/// A struct to hold the prediction model and associated metadata
pub struct PredictionModelRecord {
    pub name: String,
    pub prediction_model: Arc<dyn PredictionModel>,
    pub model_type: ModelType,
    pub input_features: Vec<(String, InputFeature)>,
    pub distance_unit: DistanceUnit,
    pub energy_rate_unit: EnergyRateUnit,
    pub ideal_energy_rate: EnergyRate,
    pub real_world_energy_adjustment: f64,
}

impl TryFrom<&PredictionModelConfig> for PredictionModelRecord {
    type Error = TraversalModelError;

    fn try_from(config: &PredictionModelConfig) -> Result<Self, Self::Error> {
        if config.input_features.is_empty() {
            return Err(TraversalModelError::BuildError(format!(
                "You must supply at least one input feature for vehicle model {}",
                config.name
            )));
        }
        let prediction_model: Arc<dyn PredictionModel> = match &config.model_type {
            ModelType::Smartcore => {
                let model = SmartcoreModel::new(&config.model_input_file, config.energy_rate_unit)?;
                Arc::new(model)
            }
            ModelType::Interpolate {
                underlying_model_type: underlying_model,
                feature_bounds,
            } => {
                let model = InterpolationModel::new(
                    &config.model_input_file,
                    *underlying_model.clone(),
                    config.input_features.clone(),
                    feature_bounds.clone(),
                    config.energy_rate_unit,
                )?;
                Arc::new(model)
            }
        };
        let ideal_energy_rate = prediction_model_ops::find_min_energy_rate(
            &prediction_model,
            &config.input_features,
            &config.energy_rate_unit,
        )?;

        let real_world_energy_adjustment = config.real_world_energy_adjustment.unwrap_or(1.0);

        Ok(PredictionModelRecord {
            name: config.name.clone(),
            prediction_model,
            model_type: config.model_type.clone(),
            input_features: config.input_features.clone(),
            distance_unit: config.distance_unit,
            energy_rate_unit: config.energy_rate_unit,
            ideal_energy_rate,
            real_world_energy_adjustment,
        })
    }
}

impl PredictionModelRecord {
    pub fn predict(
        &self,
        state: &mut [StateVariable],
        state_model: &StateModel,
    ) -> Result<(Energy, EnergyUnit), TraversalModelError> {
        let (distance, distance_unit) =
            state_model.get_distance(state, fieldname::EDGE_DISTANCE, Some(&self.distance_unit))?;
        let mut feature_vector: Vec<f64> = Vec::new();
        for (name, input_feature) in &self.input_features {
            let state_variable_f64: f64 = match input_feature {
                InputFeature::Speed(unit) => {
                    if unit.is_none() {
                        return Err(TraversalModelError::TraversalModelFailure(format!(
                            "Unit must be set for speed input feature {} but got None",
                            input_feature
                        )));
                    }
                    let (speed, _speed_unit) = state_model.get_speed(state, name, unit.as_ref())?;
                    speed.as_f64()
                }
                InputFeature::Grade(unit) => {
                    if unit.is_none() {
                        return Err(TraversalModelError::TraversalModelFailure(format!(
                            "Unit must be set for grade input feature {} but got None",
                            input_feature
                        )));
                    }
                    let (grade, _grade_unit) = state_model.get_grade(state, name, unit.as_ref())?;
                    grade.as_f64()
                }
                InputFeature::Distance(unit) => {
                    if unit.is_none() {
                        return Err(TraversalModelError::TraversalModelFailure(format!(
                            "Unit must be set for distance input feature {} but got None",
                            input_feature
                        )));
                    }
                    let (distance, _distance_unit) =
                        state_model.get_distance(state, name, unit.as_ref())?;
                    distance.as_f64()
                }
                InputFeature::Time(unit) => {
                    if unit.is_none() {
                        return Err(TraversalModelError::TraversalModelFailure(format!(
                            "Unit must be set for time input feature {} but got None",
                            input_feature
                        )));
                    }
                    let (time, _time_unit) = state_model.get_time(state, name, unit.as_ref())?;
                    time.as_f64()
                }
                InputFeature::Custom { name, unit: _ } => {
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

        let (energy_rate, _energy_rate_unit) = self.prediction_model.predict(&feature_vector)?;

        let energy_rate_real_world = energy_rate * self.real_world_energy_adjustment;

        let (energy, energy_unit) = Energy::create(
            (&distance, distance_unit),
            (&energy_rate_real_world, &self.energy_rate_unit),
        )?;

        Ok((energy, energy_unit))
    }
}
