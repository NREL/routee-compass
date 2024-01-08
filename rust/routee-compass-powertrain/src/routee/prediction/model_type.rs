use routee_compass_core::model::unit::{Grade, Speed};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    Smartcore,
    Onnx,
    Interpolate {
        underlying_model_type: Box<ModelType>,
        speed_lower_bound: Speed,
        speed_upper_bound: Speed,
        speed_bins: usize,
        grade_lower_bound: Grade,
        grade_upper_bound: Grade,
        grade_bins: usize,
    },
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", s)
    }
}
