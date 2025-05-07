use super::{
    interpolation::InterpolationSpeedGradeModel, model_type::ModelType, prediction_model_ops,
    smartcore::SmartcoreSpeedGradeModel, PredictionModel, PredictionModelConfig,
};
#[cfg(feature = "onnx")]
use crate::model::prediction::onnx::OnnxSpeedGradeModel;
use routee_compass_core::{
    model::{
        traversal::TraversalModelError,
        unit::{
            AsF64, Distance, DistanceUnit, Energy, EnergyRate, EnergyRateUnit, EnergyUnit, Grade,
            GradeUnit, Speed, SpeedUnit,
        },
    },
    util::cache_policy::float_cache_policy::FloatCachePolicy,
};
use std::sync::Arc;

/// A struct to hold the prediction model and associated metadata
pub struct PredictionModelRecord {
    pub name: String,
    pub prediction_model: Arc<dyn PredictionModel>,
    pub model_type: ModelType,
    pub speed_unit: SpeedUnit,
    pub grade_unit: GradeUnit,
    pub energy_rate_unit: EnergyRateUnit,
    pub ideal_energy_rate: EnergyRate,
    pub real_world_energy_adjustment: f64,
    pub cache: Option<FloatCachePolicy>,
}

impl TryFrom<&PredictionModelConfig> for PredictionModelRecord {
    type Error = TraversalModelError;

    fn try_from(config: &PredictionModelConfig) -> Result<Self, Self::Error> {
        let prediction_model: Arc<dyn PredictionModel> = match &config.model_type {
            ModelType::Smartcore => {
                let model = SmartcoreSpeedGradeModel::new(
                    &config.model_input_file,
                    config.speed_unit,
                    config.grade_unit,
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
                speed_lower_bound,
                speed_upper_bound,
                speed_bins: speed_bin_size,
                grade_lower_bound,
                grade_upper_bound,
                grade_bins: grade_bin_size,
            } => {
                let model = InterpolationSpeedGradeModel::new(
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
        let ideal_energy_rate = match config.ideal_energy_rate {
            None => prediction_model_ops::find_min_energy_rate(
                &prediction_model,
                config.speed_unit,
                config.grade_unit,
                &config.energy_rate_unit,
            )?,
            Some(ier) => ier,
        };

        let real_world_energy_adjustment = config.real_world_energy_adjustment.unwrap_or(1.0);
        let cache_policy = match &config.float_cache_policy {
            Some(cache_config) => Some(FloatCachePolicy::from_config(cache_config.clone())?),
            None => None,
        };

        Ok(PredictionModelRecord {
            name: config.name.clone(),
            prediction_model,
            model_type: config.model_type.clone(),
            speed_unit: config.speed_unit,
            grade_unit: config.grade_unit,
            energy_rate_unit: config.energy_rate_unit,
            ideal_energy_rate,
            real_world_energy_adjustment,
            cache: cache_policy,
        })
    }
}

impl PredictionModelRecord {
    pub fn predict(
        &self,
        speed: (Speed, &SpeedUnit),
        grade: (Grade, &GradeUnit),
        distance: (Distance, &DistanceUnit),
    ) -> Result<(Energy, EnergyUnit), TraversalModelError> {
        let (speed, speed_unit) = speed;
        let (distance, distance_unit) = distance;
        let (grade, grade_unit) = grade;

        let energy_rate = match &self.cache {
            Some(cache) => {
                let key = vec![speed.as_f64(), grade.as_f64()];
                match cache.get(&key)? {
                    Some(er) => EnergyRate::from(er),
                    None => {
                        let (energy_rate, _energy_rate_unit) = self
                            .prediction_model
                            .predict((speed, speed_unit), (grade, grade_unit))?;
                        cache.update(&key, energy_rate.as_f64())?;
                        energy_rate
                    }
                }
            }
            None => {
                let (energy_rate, _energy_rate_unit) = self
                    .prediction_model
                    .predict((speed, speed_unit), (grade, grade_unit))?;
                energy_rate
            }
        };

        let energy_rate_real_world = energy_rate * self.real_world_energy_adjustment;

        let (energy, energy_unit) = Energy::create(
            (&distance, &distance_unit),
            (&energy_rate_real_world, &self.energy_rate_unit),
        )?;

        Ok((energy, energy_unit))
    }
}
