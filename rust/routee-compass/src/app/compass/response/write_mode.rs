use super::response_output_format::ResponseOutputFormat;
use crate::app::compass::CompassAppError;
use routee_compass_core::config::CompassConfigurationError;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    path::Path,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WriteMode {
    Append,
    Overwrite,
    Error,
}

impl WriteMode {
    pub fn open_file(
        &self,
        path: &Path,
        format: &ResponseOutputFormat,
    ) -> Result<File, CompassAppError> {
        match self {
            WriteMode::Append => {
                if !path.exists() {
                    write_header(path, format)?
                }
                open_append(path)
            }
            WriteMode::Overwrite => {
                write_header(path, format)?;
                open_append(path)
            }
            WriteMode::Error => {
                if path.exists() {
                    Err(CompassAppError::CompassConfigurationError(
                        CompassConfigurationError::UserConfigurationError(format!(
                            "file exists but write mode is 'error' {}",
                            path.to_str().unwrap_or_default()
                        )),
                    ))?
                }
                write_header(path, format)?;
                open_append(path)
            }
        }
    }
}

fn write_header(path: &Path, format: &ResponseOutputFormat) -> Result<(), CompassAppError> {
    let header = format
        .initial_file_contents()
        .unwrap_or_else(|| String::from(""));
    std::fs::write(path, header).map_err(|e| {
        CompassAppError::InternalError(format!(
            "failure writing to {}: {}",
            path.to_str().unwrap_or_default(),
            e
        ))
    })
}

fn open_append(path: &Path) -> Result<File, CompassAppError> {
    OpenOptions::new().append(true).open(path).map_err(|e| {
        CompassAppError::InternalError(format!(
            "failure opening file {} in append mode: {}",
            path.to_str().unwrap_or_default(),
            e
        ))
    })
}
