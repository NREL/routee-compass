use super::{
    interpolation::InterpolationModel, model_type::ModelType, prediction_model_ops,
    smartcore::SmartcoreModel, PredictionModel, PredictionModelConfig,
};
#[cfg(feature = "onnx")]
use crate::model::prediction::onnx::OnnxSpeedGradeModel;
use routee_compass_core::model::{
    state::{InputFeature, StateModel, StateVariable},
    traversal::TraversalModelError,
    unit::{
        Distance, DistanceUnit, Energy, EnergyRate, EnergyRateUnit, EnergyUnit, Grade, GradeUnit,
        Speed, SpeedUnit,
    },
};
use std::sync::Arc;

/// A struct to hold the prediction model and associated metadata
pub struct PredictionModelRecord {
    pub name: String,
    pub prediction_model: Arc<dyn PredictionModel>,
    pub model_type: ModelType,
    pub input_features: Vec<(String, InputFeature)>,
    pub energy_rate_unit: EnergyRateUnit,
    pub ideal_energy_rate: EnergyRate,
    pub real_world_energy_adjustment: f64,
}

impl TryFrom<&PredictionModelConfig> for PredictionModelRecord {
    type Error = TraversalModelError;

    fn try_from(config: &PredictionModelConfig) -> Result<Self, Self::Error> {
        let prediction_model: Arc<dyn PredictionModel> = match &config.model_type {
            ModelType::Smartcore => {
                let model = SmartcoreModel::new(
                    &config.model_input_file,
                    config.input_features,
                    config.energy_rate_unit,
                )?;
                Arc::new(model)
            }
            ModelType::Onnx => {
                #[cfg(feature = "onnx")]
                {
                    let model = OnnxSpeedGradeModel::new(
                        &config.model_input_file,
                        config.speed_unit,
                        config.grade_unit,
                        config.energy_rate_unit,
                    )?;
                    Arc::new(model)
                }
                #[cfg(not(feature = "onnx"))]
                {
                    return Err(TraversalModelError::BuildError(
                        "Cannot build Onnx model without `onnx` feature enabled for compass-powertrain"
                            .to_string(),
                    ));
                }
            }
            ModelType::Interpolate {
                underlying_model_type: underlying_model,
                feature_bounds,
            } => {
                let model = InterpolationModel::new(
                    &config.model_input_file,
                    *underlying_model.clone(),
                    config.name.clone(),
                    config.speed_unit,
                    (*speed_lower_bound, *speed_upper_bound),
                    *speed_bin_size,
                    config.grade_unit,
                    (*grade_lower_bound, *grade_upper_bound),
                    *grade_bin_size,
                    config.energy_rate_unit,
                )?;
                Arc::new(model)
            }
        };
        let ideal_energy_rate = prediction_model_ops::find_min_energy_rate(
            &prediction_model,
            config.speed_unit,
            config.grade_unit,
            &config.energy_rate_unit,
        )?;

        let real_world_energy_adjustment = config.real_world_energy_adjustment.unwrap_or(1.0);

        Ok(PredictionModelRecord {
            name: config.name.clone(),
            prediction_model,
            model_type: config.model_type.clone(),
            input_features: config.input_features.clone(),
            energy_rate_unit: config.energy_rate_unit,
            ideal_energy_rate,
            real_world_energy_adjustment,
        })
    }
}

impl PredictionModelRecord {
    pub fn predict(
        &self,
        input_features: &[(String, InputFeature)],
        distance: (Distance, &DistanceUnit),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(Energy, EnergyUnit), TraversalModelError> {
        let (distance, distance_unit) = distance;

        let (energy_rate, _energy_rate_unit) =
            self.prediction_model
                .predict(input_features, state, state_model)?;

        let energy_rate_real_world = energy_rate * self.real_world_energy_adjustment;

        let (energy, energy_unit) = Energy::create(
            (&distance, distance_unit),
            (&energy_rate_real_world, &self.energy_rate_unit),
        )?;

        Ok((energy, energy_unit))
    }
}
