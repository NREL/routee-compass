use super::{map_error::MapError, matching_type::MatchingType};
use crate::{
    config::OneOrMany,
    model::{map::GeometryInput, unit::DistanceUnit},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use uom::si::f64::Length;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum MapModelConfig {
    #[serde(rename = "vertex")]
    VertexMapModelConfig {
        /// distance from coordinate to the nearest vertex required for map matching
        tolerance: Option<DistanceTolerance>,
        /// edge geometries. if not provided, edge geometries are created from vertex coordinates.
        geometry: Option<OneOrMany<GeometryInput>>,
        /// allow source-only queries for shortest path tree outputs
        queries_without_destinations: bool,
        /// the [`MatchingType`]s supported
        matching_type: Option<Vec<String>>,
    },
    #[serde(rename = "edge")]
    EdgeMapModelConfig {
        /// distance from coordinate to the nearest vertex required for map matching
        tolerance: Option<DistanceTolerance>,
        /// edge geometries for each [`EdgeList`]
        geometry: OneOrMany<GeometryInput>,
        /// allow source-only queries for shortest path tree outputs
        queries_without_destinations: bool,
        /// the [`MatchingType`]s supported
        matching_type: Option<Vec<String>>,
    },
}

impl MapModelConfig {
    pub fn get_matching_type(&self) -> Result<MatchingType, MapError> {
        let matching_type = match self {
            MapModelConfig::VertexMapModelConfig {
                tolerance: _,
                geometry: _,
                queries_without_destinations: _,
                matching_type,
            } => matching_type,
            MapModelConfig::EdgeMapModelConfig {
                tolerance: _,
                geometry: _,
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
            geometry: None,
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
                            "unable to deserialize map model configuration section due to '{e}'. input data: \n{map_model_str}"
                        )
                    })?;
                Ok(map_model_config)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DistanceTolerance {
    pub distance: f64,
    pub unit: DistanceUnit,
}

impl DistanceTolerance {
    pub fn to_uom(&self) -> Length {
        self.unit.to_uom(self.distance)
    }
}
