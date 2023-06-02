use std::collections::HashSet;

use super::graph::NodeId;



pub struct StopCosts {
    pub stop_signs: HashSet<NodeId>,
    pub traffic_lights: HashSet<NodeId>,
}