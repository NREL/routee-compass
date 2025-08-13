use super::custom_weight_type::CustomWeightType;
use crate::plugin::{input::InputJsonExtensions, input::InputPluginError};
use routee_compass_core::util::geo::haversine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum WeightHeuristic {
    /// computes a weight directly as the haversine distance estimation between
    /// trip origin and destination, in kilometers.
    Haversine,
    /// user provides a field of some custom weight type that is used directly
    /// for weight estimates.
    Custom {
        custom_weight_type: CustomWeightType,
    },
}

impl WeightHeuristic {
    pub fn estimate_weight(&self, query: &serde_json::Value) -> Result<f64, InputPluginError> {
        match self {
            WeightHeuristic::Haversine => {
                let o = query.get_origin_coordinate()?;
                let d_option = query.get_destination_coordinate()?;
                match d_option {
                    None => Err(InputPluginError::InputPluginFailed(String::from(
                        "cannot estimate search size without destination coordinate",
                    ))),
                    Some(d) => haversine::coord_distance(&o, &d)
                        .map(|d| d.get::<uom::si::length::kilometer>())
                        .map_err(|s| {
                            InputPluginError::InputPluginFailed(format!(
                                "failed calculating load balancing weight value due to {s}"
                            ))
                        }),
                }
            }
            WeightHeuristic::Custom { custom_weight_type } => custom_weight_type.get_weight(query),
        }
    }
}
