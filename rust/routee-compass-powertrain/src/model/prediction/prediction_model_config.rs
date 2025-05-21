use super::ModelType;
use routee_compass_core::{
    model::unit::{EnergyRateUnit, GradeUnit, SpeedUnit},
    util::cache_policy::float_cache_policy::FloatCachePolicyConfig,
};
use serde::{Deserialize, Serialize};

/// Configuration for a prediction model parsed from the Compass configuration file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictionModelConfig {
    pub name: String,
    pub model_input_file: String,
    pub model_type: ModelType,
    pub speed_unit: SpeedUnit,
    pub grade_unit: GradeUnit,
    pub energy_rate_unit: EnergyRateUnit,
    pub real_world_energy_adjustment: Option<f64>,
    pub float_cache_policy: Option<FloatCachePolicyConfig>,
}

impl PredictionModelConfig {
    pub fn new(
        name: String,
        model_input_file: String,
        model_type: ModelType,
        speed_unit: SpeedUnit,
        grade_unit: GradeUnit,
        energy_rate_unit: EnergyRateUnit,
        real_world_energy_adjustment: Option<f64>,
        float_cache_policy: Option<FloatCachePolicyConfig>,
    ) -> Self {
        Self {
            name,
            model_input_file,
            model_type,
            speed_unit,
            grade_unit,
            energy_rate_unit,
            real_world_energy_adjustment,
            float_cache_policy,
        }
    }
}
