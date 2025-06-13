use super::ModelType;
use routee_compass_core::model::{state::InputFeature, unit::EnergyRateUnit};
use serde::{Deserialize, Serialize};

/// Configuration for a prediction model parsed from the Compass configuration file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictionModelConfig {
    pub name: String,
    pub model_input_file: String,
    pub model_type: ModelType,
    pub input_features: Vec<InputFeature>,
    pub energy_rate_unit: EnergyRateUnit,
    pub real_world_energy_adjustment: Option<f64>,
}

impl PredictionModelConfig {
    pub fn new(
        name: String,
        model_input_file: String,
        model_type: ModelType,
        input_features: Vec<InputFeature>,
        energy_rate_unit: EnergyRateUnit,
        real_world_energy_adjustment: Option<f64>,
    ) -> Self {
        Self {
            name,
            model_input_file,
            model_type,
            input_features,
            energy_rate_unit,
            real_world_energy_adjustment,
        }
    }
}
