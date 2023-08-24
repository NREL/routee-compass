use compass_app::app::compass::compass_app::CompassApp;
use pyo3::prelude::*;

#[pyclass]
pub struct CompassAppWrapper {
    compass_app: CompassApp,
}


