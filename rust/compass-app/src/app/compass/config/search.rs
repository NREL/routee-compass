use compass_core::model::{
    traversal::{
        default::{distance::DistanceModel, velocity_lookup::VelocityLookupModel},
        traversal_model::TraversalModel,
        traversal_model_error::TraversalModelError,
    },
    units::TimeUnit,
};
use compass_powertrain::routee::routee_random_forest::RouteERandomForestModel;
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
    #[serde(rename = "distance")]
    Distance,
    #[serde(rename = "velocity_table")]
    VelocityTable {
        filename: String,
        output_unit: TimeUnit,
    },
    #[serde(rename = "powertrain")]
    Powertrain {
        model: String,
        velocity_filename: String,
        velocity_output_unit: TimeUnit,
    },
}

impl TryFrom<TraversalModelConfig> for Box<dyn TraversalModel> {
    type Error = TraversalModelError;

    fn try_from(value: TraversalModelConfig) -> Result<Box<dyn TraversalModel>, Self::Error> {
        match value {
            TraversalModelConfig::Distance => {
                let model = DistanceModel {};
                Ok(Box::new(model))
            }
            TraversalModelConfig::VelocityTable {
                filename,
                output_unit,
            } => {
                let model = VelocityLookupModel::from_file(&filename, output_unit.clone())?;
                Ok(Box::new(model))
            }
            TraversalModelConfig::Powertrain {
                model,
                velocity_filename,
                velocity_output_unit,
            } => {
                let model = RouteERandomForestModel::new_w_speed_file(
                    model.clone(),
                    velocity_filename.clone(),
                    velocity_output_unit.clone(),
                )?;
                Ok(Box::new(model))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchConfig {
    pub algorithm: AlgorithmConfig,
    pub traversal_model: TraversalModelConfig,
}
