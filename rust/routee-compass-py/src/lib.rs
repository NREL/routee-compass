#![doc = include_str!("doc.md")]

pub mod app_wrapper;

use app_wrapper::CompassAppWrapper;
use pyo3::prelude::*;

#[pymodule]
fn routee_compass_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<CompassAppWrapper>()?;

    Ok(())
}
