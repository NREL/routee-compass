pub mod model_type;
pub mod prediction_model;
pub mod prediction_model_ops;
pub mod prediction_model_record;
pub mod smartcore;

#[cfg(feature = "onnx")]
pub mod onnx;

pub use prediction_model::PredictionModel;
pub use prediction_model_ops::load_prediction_model;
pub use prediction_model_record::PredictionModelRecord;
