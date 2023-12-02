use crate::plugin::{input::input_json_extensions::InputJsonExtensions, plugin_error::PluginError};
use routee_compass_core::util::{geo::haversine, unit::as_f64::AsF64};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum WeightHeuristic {
    /// computes a weight directly as the haversine distance estimation between
    /// trip origin and destination, in meters.
    ///
    /// # Arguments
    ///
    /// * `default` - fill value if no destination is provided
    Haversine { default: f64 },
}

impl WeightHeuristic {
    pub fn estimate_weight(&self, query: serde_json::Value) -> Result<f64, PluginError> {
        match self {
            WeightHeuristic::Haversine { default } => {
                let o = query.get_origin_coordinate()?;
                let d_option = query.get_destination_coordinate()?;
                match d_option {
                    None => Ok(*default),
                    Some(d) => haversine::coord_distance_meters(o, d)
                        .map(|d| d.as_f64())
                        .map_err(|s| {
                            PluginError::PluginFailed(format!(
                                "failed calculating load balancing weight value due to {}",
                                s
                            ))
                        }),
                }
            }
        }
    }
}
