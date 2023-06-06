use algorithm::extract_largest_scc;
use graph::{Graph, Link, Node};
use map::{RustMap, SearchInput, SearchResult, SearchType};
use powertrain::VehicleParameters;
use time_of_day_speed::TimeOfDaySpeeds;

use pyo3::prelude::*;

mod algorithm;

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
