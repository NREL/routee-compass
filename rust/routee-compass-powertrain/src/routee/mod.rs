pub mod default_vehicles;
pub mod energy_model_ops;
pub mod energy_model_service;
pub mod energy_traversal_model;
pub mod model_type;
pub mod prediction_model;
pub mod smartcore;
pub mod vehicle;

#[cfg(feature = "onnx")]
pub mod onnx;
