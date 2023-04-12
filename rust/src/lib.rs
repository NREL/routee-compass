pub mod algorithm;
pub mod graph;

use algorithm::py_time_shortest_path;
use graph::{Graph, Link, Node};
use pyo3::prelude::*;

#[pymodule]
fn compass_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Link>()?;
    m.add_class::<Node>()?;
    m.add_class::<Graph>()?;
    m.add_function(wrap_pyfunction!(py_time_shortest_path, m)?)?;

    Ok(())
}
