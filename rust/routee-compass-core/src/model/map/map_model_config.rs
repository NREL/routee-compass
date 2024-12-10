// use super::map_input_type::MapInputType;
use crate::model::unit::{Distance, DistanceUnit};
use serde::{de, Deserialize, Deserializer, Serialize};

use super::map_input_type::MapInputType;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum MapModelConfig {
    #[serde(rename = "vertex")]
    VertexMapModelConfig {
        #[serde(deserialize_with = "de_tolerance")]
        tolerance: Option<(Distance, DistanceUnit)>,
        geometry_input_file: Option<String>,
        queries_without_destinations: bool,
        map_input_type: Option<MapInputType>,
    },
    #[serde(rename = "edge")]
    EdgeMapModelConfig {
        #[serde(deserialize_with = "de_tolerance")]
        tolerance: Option<(Distance, DistanceUnit)>,
        geometry_input_file: String,
        queries_without_destinations: bool,
        map_input_type: Option<MapInputType>,
    },
}

impl Default for MapModelConfig {
    fn default() -> Self {
        MapModelConfig::VertexMapModelConfig {
            tolerance: None,
            geometry_input_file: None,
            queries_without_destinations: false,
            map_input_type: None,
        }
    }
}

fn de_tolerance<'de, D>(value: D) -> Result<Option<(Distance, DistanceUnit)>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ToleranceVisitor;

    impl<'de> de::Visitor<'de> for ToleranceVisitor {
        type Value = Option<(Distance, DistanceUnit)>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a vector of [Distance, DistanceUnit]")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let distance: Distance = match seq.next_element() {
                Ok(Some(distance)) => Ok(distance),
                Err(e) => Err(e),
                Ok(None) => {
                    let msg = String::from("Empty array provided for tolerance. To specify unbounded mapping tolerance, omit the tolerance field.");
                    Err(serde::de::Error::custom(msg))
                }
            }?;

            let distance_unit: DistanceUnit = match seq.next_element() {
                Ok(Some(distance_unit)) => Ok(distance_unit),
                Ok(None) => Err(serde::de::Error::custom(String::from(
                    "Distance tolerance provided without distance unit.",
                ))),
                Err(e) => Err(e),
            }?;
            Ok(Some((distance, distance_unit)))
        }
    }

    value.deserialize_seq(ToleranceVisitor {})
}
