use std::collections::HashSet;

use super::graph::{Link, NodeId};

use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct StopCosts {
    pub stop_signs: HashSet<NodeId>,
    pub traffic_lights: HashSet<NodeId>,
    pub stop_cost_gallons_diesel: f64,
}

#[pymethods]
impl StopCosts {
    #[new]
    pub fn new(
        stop_signs: HashSet<NodeId>,
        traffic_lights: HashSet<NodeId>,
        stop_cost_gallons_diesel: f64,
    ) -> Self {
        StopCosts {
            stop_signs,
            traffic_lights,
            stop_cost_gallons_diesel,
        }
    }

    pub fn energy_stop_cost(&self, link: &Link) -> f64 {
        if self.stop_signs.contains(&link.end_node) {
            self.stop_cost_gallons_diesel
        } else if self.traffic_lights.contains(&link.end_node) {
            // assume the vehicle only stops at a traffic light half the time
            self.stop_cost_gallons_diesel * 0.5
        } else {
            0.0
        }
    }
}
