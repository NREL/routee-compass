pub mod model_type;
pub mod prediction_model;
pub mod smartcore;

#[cfg(feature = "onnx")]
pub mod onnx;

pub use prediction_model::{load_prediction_model, PredictionModel, PredictionModelRecord};
