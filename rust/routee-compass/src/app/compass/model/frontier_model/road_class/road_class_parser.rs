use std::collections::{HashMap, HashSet};

use indoc::formatdoc;
use serde::Deserialize;

use crate::app::compass::compass_app_error::CompassAppError;

#[derive(Default, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RoadClassParser {
    mapping: HashMap<String, u8>,
}

impl RoadClassParser {
    pub fn read_query(
        &self,
        query: &serde_json::Value,
    ) -> Result<Option<HashSet<u8>>, CompassAppError> {
        let road_classes = match query.get("road_classes") {
            None => None,
            Some(value) => {
                // try parsing as a u8 first
                let rc = serde_json::from_value::<HashSet<u8>>(value.to_owned())
                    .or_else(|_| {
                        if self.mapping.is_empty() {
                            // if we don't have a road class mapping then we fail
                            let value_string = value.to_string();
                            Err(CompassAppError::CompassFailure(
                                formatdoc! {r#"
                                    Could not parse incoming query valid road_classes of {value_string} as an array of integers 
                                    and this FrontierModel does not specify a mapping of string to integer. 
                                    Either pass a valid array of integers or reload the application with a 
                                    mapping from string road class to integer road class.
                                    "#
                                }.to_string()
                            ))
                        } else {
                                let strings =
                                    serde_json::from_value::<HashSet<String>>(value.to_owned())?;
                                let ints = strings
                                    .iter()
                                    .map(|s| {
                                        self.mapping.get(s).cloned().ok_or_else(|| {
                                            let valid_mapping: Vec<String> = self.mapping.keys().cloned().collect();
                                            CompassAppError::CompassFailure(
                                                format!("Could not find road class mapping for incoming value {}. Here are the road classes I have: {:?}", s, valid_mapping)
                                            )
                                        })
                                    })
                                    .collect::<Result<HashSet<u8>, CompassAppError>>()?;
                                Ok(ints)
                        }
                    })?;
                Some(rc)
            }
        };
        Ok(road_classes)
    }
}
