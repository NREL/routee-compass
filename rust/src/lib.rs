pub mod algorithm;
pub mod graph;
pub mod map;
pub mod powertrain;
pub mod time_of_day_speed;

use algorithm::extract_largest_scc;
use graph::{Graph, Link, Node};
use map::RustMap;
use pyo3::prelude::*;
use time_of_day_speed::TimeOfDaySpeeds;
use powertrain::VehicleParameters;

#[pymodule]
fn compass_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Link>()?;
    m.add_class::<Node>()?;
    m.add_class::<Graph>()?;
    m.add_class::<RustMap>()?;
    m.add_class::<TimeOfDaySpeeds>()?;
    m.add_class::<VehicleParameters>()?;
    m.add_function(wrap_pyfunction!(extract_largest_scc, m)?)?;

    Ok(())
}
