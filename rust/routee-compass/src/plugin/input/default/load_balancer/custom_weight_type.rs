use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    app::compass::config::config_json_extension::ConfigJsonExtensions,
    plugin::{input::input_field::InputField, plugin_error::PluginError},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]

pub enum CustomWeightType {
    /// a weight value found on each query which can be used directly as it is a numeric
    /// field. will use provided column_name or fall back to InputField::QueryWeightEstimate.
    /// if a default fill value is provided, it is used when the weight field is missing.
    Numeric { column_name: Option<String> },
    /// a weight value found on each query which has a categorical field value. symbols are
    /// converted to f64 values via a user-provided mapping. will use provided column_name
    /// or fall back to InputField::QueryWeightEstimate.
    /// if a default fill value is provided, it is used when the weight field is missing.
    Categorical {
        column_name: Option<String>,
        mapping: HashMap<String, f64>,
        default: Option<f64>,
    },
}

impl CustomWeightType {
    pub fn get_weight(&self, query: &serde_json::Value) -> Result<f64, PluginError> {
        match self {
            CustomWeightType::Numeric { column_name } => {
                let col: String = column_name
                    .to_owned()
                    .unwrap_or(InputField::QueryWeightEstimate.to_string());
                let value = query
                    .get_config_f64(&col, &"custom_weight_type")
                    .map_err(|_| PluginError::ParseError(col, String::from("String")))?;
                Ok(value)
            }
            CustomWeightType::Categorical {
                column_name,
                mapping,
                default,
            } => {
                let col: String = column_name
                    .to_owned()
                    .unwrap_or(InputField::QueryWeightEstimate.to_string());
                let categorical_value = query
                    .get_config_string(&col, &"custom_weight_type")
                    .map_err(|_| PluginError::ParseError(col.clone(), String::from("String")))?;
                match (mapping.get(&categorical_value), default) {
                    (Some(result), _) => Ok(*result),
                    (None, Some(fallback)) => Ok(*fallback),
                    _ => Err(PluginError::InputError(format!("load balancing categorical {} not found in mapping and no default was provided", categorical_value))),
                }
            }
        }
    }
}
