use super::response_output_format_json as json_ops;
use crate::app::compass::compass_app_error::CompassAppError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ResponseOutputFormat {
    Json { newline_delimited: bool },
}

impl ResponseOutputFormat {
    pub fn initial_file_contents(&self) -> Option<String> {
        match self {
            ResponseOutputFormat::Json { newline_delimited } => {
                json_ops::initial_file_contents(*newline_delimited)
            }
        }
    }

    pub fn final_file_contents(&self) -> Option<String> {
        match self {
            ResponseOutputFormat::Json { newline_delimited } => {
                json_ops::final_file_contents(*newline_delimited)
            }
        }
    }

    pub fn format_response(&self, response: &serde_json::Value) -> Result<String, CompassAppError> {
        match self {
            ResponseOutputFormat::Json { newline_delimited } => {
                json_ops::format_response(response, *newline_delimited)
            }
        }
    }

    pub fn delimiter(&self) -> Option<String> {
        match self {
            ResponseOutputFormat::Json { newline_delimited } => {
                json_ops::delimiter(*newline_delimited)
            }
        }
    }
}
