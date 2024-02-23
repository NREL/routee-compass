use pyo3::prelude::*;
use super::compass_app_builder::CompassAppBuilder;

#[derive(Clone)]
#[pyclass]
pub struct CompassAppPyBuilder {
    pub builder: CompassAppBuilder,
}

