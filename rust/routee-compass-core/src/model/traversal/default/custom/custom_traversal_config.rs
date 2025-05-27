use super::CustomInputFormat;
use crate::model::state::CustomFeatureFormat;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomTraversalConfig {
    /// file containing custom values for each edge id
    pub input_file: String,
    /// whether the input data is dense (aka, an enumerated edge_id file) or
    /// sparse (aka, a CSV with key/value pairs)
    pub file_format: CustomInputFormat,
    /// name of the feature, a unique name apart from it's unit type
    pub name: String,
    /// name of the unit space the feature exists in, such as Percent
    pub unit: String,
    /// format and initial value of this feature
    pub feature: CustomFeatureFormat,
    /// whether to accumulate values (via addition) or simply insert/set them
    pub accumulator: bool,
}

impl std::fmt::Display for CustomTraversalConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = serde_json::to_string_pretty(self).unwrap_or_default();
        write!(f, "{}", output)
    }
}
