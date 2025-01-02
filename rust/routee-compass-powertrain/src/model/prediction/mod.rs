pub mod interpolation;
mod model_type;
mod prediction_model;
pub mod prediction_model_ops;
mod prediction_model_record;
pub mod smartcore;

#[cfg(feature = "onnx")]
pub mod onnx;

pub use model_type::ModelType;
pub use prediction_model::PredictionModel;
pub use prediction_model_ops::load_prediction_model;
pub use prediction_model_record::PredictionModelRecord;
