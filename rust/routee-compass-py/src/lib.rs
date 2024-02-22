#![doc = include_str!("doc.md")]

pub mod app_graph_ops;
pub mod compass_app_py;
mod compass_app_py_builder;

use compass_app_py::CompassAppPy;
use pyo3::prelude::*;

#[pymodule]
fn routee_compass_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<CompassAppPy>()?;

    Ok(())
}
