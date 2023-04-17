pub mod algorithm;
pub mod graph;
pub mod map;
pub mod powertrain;

use algorithm::largest_scc;
use graph::{Graph, Link, Node};
use map::RustMap;
use pyo3::prelude::*;

#[pymodule]
fn compass_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Link>()?;
    m.add_class::<Node>()?;
    m.add_class::<Graph>()?;
    m.add_class::<RustMap>()?;
    m.add_function(wrap_pyfunction!(largest_scc, m)?)?;

    Ok(())
}
