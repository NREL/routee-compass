use compass_core::model::units::TimeUnit;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum AlgorithmConfig {
    #[serde(rename = "astar")]
    AStar {
        heuristic: String,
        bidirectional: bool,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub enum TraversalModelConfig {
    Distance,
    VelocityTable {
        filename: String,
        output_unit: TimeUnit,
    },
    Powertrain {
        model: String,
        velocity_filename: String,
        velocity_output_unit: TimeUnit,
    },
}

#[derive(Debug, Deserialize)]
pub struct SearchConfig {
    pub algorithm: AlgorithmConfig,
    pub traversal_model: TraversalModelConfig,
}
