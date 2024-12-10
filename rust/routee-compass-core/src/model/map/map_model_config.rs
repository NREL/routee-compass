use super::matching_type::MatchingType;
use crate::model::unit::{Distance, DistanceUnit};
// use serde::{de, Deserialize, Deserializer, Serialize};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum MapModelConfig {
    #[serde(rename = "vertex")]
    VertexMapModelConfig {
        // #[serde(deserialize_with = "de_tolerance")]
        tolerance: Option<DistanceTolerance>,
        geometry_input_file: Option<String>,
        queries_without_destinations: bool,
        matching_type: Option<MatchingType>,
    },
    #[serde(rename = "edge")]
    EdgeMapModelConfig {
        // #[serde(deserialize_with = "de_tolerance")]
        tolerance: Option<DistanceTolerance>,
        geometry_input_file: String,
        queries_without_destinations: bool,
        matching_type: Option<MatchingType>,
    },
}

impl Default for MapModelConfig {
    fn default() -> Self {
        MapModelConfig::VertexMapModelConfig {
            tolerance: None,
            geometry_input_file: None,
            queries_without_destinations: true,
            matching_type: None,
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

// fn de_tolerance<'de, D>(value: D) -> Result<Option<(Distance, DistanceUnit)>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     struct ToleranceVisitor;

//     impl<'de> de::Visitor<'de> for ToleranceVisitor {
//         type Value = Option<(Distance, DistanceUnit)>;

//         fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//             formatter.write_str("a vector of [Distance, DistanceUnit]")
//         }

//         fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//         where
//             A: de::SeqAccess<'de>,
//         {
//             let distance: Distance = match seq.next_element() {
//                 Ok(Some(distance)) => Ok(distance),
//                 Err(e) => Err(e),
//                 Ok(None) => {
//                     let msg = String::from("Empty array provided for tolerance. To specify unbounded mapping tolerance, omit the tolerance field.");
//                     Err(serde::de::Error::custom(msg))
//                 }
//             }?;

//             let distance_unit: DistanceUnit = match seq.next_element() {
//                 Ok(Some(distance_unit)) => Ok(distance_unit),
//                 Ok(None) => Err(serde::de::Error::custom(String::from(
//                     "Distance tolerance provided without distance unit.",
//                 ))),
//                 Err(e) => Err(e),
//             }?;
//             Ok(Some((distance, distance_unit)))
//         }
//     }

//     value.deserialize_seq(ToleranceVisitor {})
// }
