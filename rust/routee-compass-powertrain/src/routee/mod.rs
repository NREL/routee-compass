pub mod model_type;
pub mod prediction_model;
pub mod smartcore;
pub mod speed_grade_generic_model;
pub mod speed_grade_model_ops;
pub mod speed_grade_model_service;
pub mod speed_grade_phev_model;

#[cfg(feature = "onnx")]
pub mod onnx;
