use super::{map_error::MapError, matching_type::MatchingType};
use crate::model::unit::{Distance, DistanceUnit};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum MapModelConfig {
    #[serde(rename = "vertex")]
    VertexMapModelConfig {
        // #[serde(deserialize_with = "de_tolerance")]
        tolerance: Option<DistanceTolerance>,
        geometry_input_file: Option<String>,
        queries_without_destinations: bool,
        matching_type: Option<Vec<String>>,
    },
    #[serde(rename = "edge")]
    EdgeMapModelConfig {
        // #[serde(deserialize_with = "de_tolerance")]
        tolerance: Option<DistanceTolerance>,
        geometry_input_file: String,
        queries_without_destinations: bool,
        matching_type: Option<Vec<String>>,
    },
}

impl MapModelConfig {
    pub fn get_matching_type(&self) -> Result<MatchingType, MapError> {
        let matching_type = match self {
            MapModelConfig::VertexMapModelConfig {
                tolerance: _,
                geometry_input_file: _,
                queries_without_destinations: _,
                matching_type,
            } => matching_type,
            MapModelConfig::EdgeMapModelConfig {
                tolerance: _,
                geometry_input_file: _,
                queries_without_destinations: _,
                matching_type,
            } => matching_type,
        };
        match matching_type {
            None => Ok(MatchingType::default()),
            Some(string_list) => {
                let deserialized = string_list
                    .iter()
                    .map(|s| MatchingType::from_str(s.as_str()))
                    .collect::<Result<Vec<_>, _>>()?;
                match deserialized[..] {
                    [MatchingType::Point] => Ok(MatchingType::Point),
                    [MatchingType::VertexId] => Ok(MatchingType::VertexId),
                    [MatchingType::EdgeId] => Ok(MatchingType::EdgeId),
                    _ => Ok(MatchingType::Combined(deserialized)),
                }
            }
        }
    }
}

impl Default for MapModelConfig {
    fn default() -> Self {
        MapModelConfig::VertexMapModelConfig {
            tolerance: None,
            geometry_input_file: None,
            queries_without_destinations: true,
            matching_type: Some(MatchingType::names()),
        }
    }
}

impl TryFrom<Option<&Value>> for MapModelConfig {
    type Error = String;

    fn try_from(value: Option<&Value>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(MapModelConfig::default()),
            Some(json) => {
                let map_model_str = serde_json::to_string_pretty(&json).unwrap_or_default();
                let map_model_config: MapModelConfig =
                    serde_json::from_value(json.clone()).map_err(|e| {
                        format!(
                            "unable to deserialize map model configuration section due to '{}'. input data: \n{}",
                            e,
                            map_model_str
                        )
                    })?;
                Ok(map_model_config)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DistanceTolerance {
    pub distance: Distance,
    pub unit: DistanceUnit,
}

impl DistanceTolerance {
    pub fn unpack(&self) -> (Distance, DistanceUnit) {
        (self.distance, self.unit)
    }
}
