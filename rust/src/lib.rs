pub mod prototype;

use prototype::algorithm::largest_scc;
use prototype::graph::{Graph, Link, Node};
use prototype::map::RustMap;
use pyo3::prelude::*;

mod algorithm;
mod model;
mod util;

#[pymodule]
fn compass_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Link>()?;
    m.add_class::<Node>()?;
    m.add_class::<Graph>()?;
    m.add_class::<RustMap>()?;
    m.add_function(wrap_pyfunction!(largest_scc, m)?)?;

    Ok(())
}
