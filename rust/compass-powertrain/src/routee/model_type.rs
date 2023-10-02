use std::sync::Arc;

use compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{EnergyRateUnit, SpeedUnit, GradeUnit},
};
use serde::{Deserialize, Serialize};

use super::{
    onnx::onnx_speed_grade_model::OnnxSpeedGradeModel, prediction_model::SpeedGradePredictionModel,
    smartcore::smartcore_speed_grade_model::SmartcoreSpeedGradeModel,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    Smartcore,
    Onnx,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", s)
    }
}

impl ModelType {
    /// builds a speed grade energy prediction model
    pub fn build(
        &self,
        energy_model_path: String,
        energy_model_speed_unit: SpeedUnit,
        energy_model_grade_unit: GradeUnit,
        energy_model_energy_rate_unit: EnergyRateUnit,
    ) -> Result<Arc<dyn SpeedGradePredictionModel>, TraversalModelError> {
        // Load random forest binary file
        let model: Arc<dyn SpeedGradePredictionModel> = match self {
            ModelType::Smartcore => Arc::new(SmartcoreSpeedGradeModel::new(
                energy_model_path.clone(),
                energy_model_speed_unit.clone(),
                energy_model_grade_unit.clone(),
                energy_model_energy_rate_unit.clone(),
            )?),
            ModelType::Onnx => Arc::new(OnnxSpeedGradeModel::new(
                energy_model_path.clone(),
                energy_model_speed_unit.clone(),
                energy_model_grade_unit.clone(),
                energy_model_energy_rate_unit.clone(),
            )?),
        };
        Ok(model)
    }
}
