mod algorithm;
mod graph;
mod map;
mod powertrain;
mod time_of_day_speed;

use crate::graph::{Graph, Link, Node};
use crate::map::{RustMap, SearchInput, SearchResult, SearchType};
use crate::powertrain::VehicleParameters;
use crate::time_of_day_speed::TimeOfDaySpeeds;
use algorithm::extract_largest_scc;

use pyo3::prelude::*;

#[pymodule]
fn compass_prototype(_py: Python, m: &PyModule) -> PyResult<()> {
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
