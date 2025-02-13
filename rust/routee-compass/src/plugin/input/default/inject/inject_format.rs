use routee_compass_core::config::CompassConfigurationError;
use routee_compass_core::util::serde::serde_ops;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InjectFormat {
    String,
    Json,
    Toml,
}

impl InjectFormat {
    pub fn to_json(&self, value: &str) -> Result<serde_json::Value, CompassConfigurationError> {
        match self {
            InjectFormat::String => {
                let decode_result = serde_ops::string_deserialize(value);
                decode_result.map_err(|e| {
                    CompassConfigurationError::UserConfigurationError(format!(
                        "could not deserialize inject value as string: {}",
                        e
                    ))
                })
            }
            InjectFormat::Json => {
                let result = serde_json::from_str(value);
                result.map_err(|e| {
                    CompassConfigurationError::UserConfigurationError(format!(
                        "could not deserialize inject value as JSON: {}",
                        e
                    ))
                })
            }
            InjectFormat::Toml => todo!(),
        }
    }
}
