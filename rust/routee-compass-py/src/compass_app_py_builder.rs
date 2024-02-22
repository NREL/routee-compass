use pyo3::pyclass;
use routee_compass::app::compass::config::builder::{
    compass_app_build_context::CompassAppBuildContext, compass_app_builder::CompassAppBuilder,
};

#[pyclass]
pub struct CompassAppPyBuilder {}

impl CompassAppBuildContext for CompassAppPyBuilder {
    fn init(&self) -> CompassAppBuilder {
        CompassAppBuilder::default()
    }
}
