pub mod prototype;


use prototype::algorithm::extract_largest_scc;
use prototype::graph::{Graph, Link, Node};
use prototype::map::{RustMap, SearchInput, SearchResult, SearchType};
use prototype::powertrain::VehicleParameters;
use prototype::time_of_day_speed::TimeOfDaySpeeds;

use pyo3::prelude::*;

mod algorithm;
mod model;
mod util;
mod implementations;

#[pymodule]
fn compass_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Link>()?;
    m.add_class::<Node>()?;
    m.add_class::<Graph>()?;
    m.add_class::<RustMap>()?;
    m.add_class::<SearchInput>()?;
    m.add_class::<SearchResult>()?;
    m.add_class::<SearchType>()?;
    m.add_class::<TimeOfDaySpeeds>()?;
    m.add_class::<VehicleParameters>()?;
    m.add_function(wrap_pyfunction!(extract_largest_scc, m)?)?;

    Ok(())
}
