use super::response_output_format::ResponseOutputFormat;
use crate::app::compass::compass_app_error::CompassAppError;
use std::io::prelude::*;
use std::{
    fs::File,
    sync::{Arc, Mutex},
};

pub enum ResponseSink {
    None,
    File {
        filename: String,
        file: Arc<Mutex<File>>,
        format: ResponseOutputFormat,
        delimiter: Option<String>,
    },
    Combined(Vec<Box<ResponseSink>>),
}

impl ResponseSink {
    /// uses a writer
    pub fn write_response(&self, response: &mut serde_json::Value) -> Result<(), CompassAppError> {
        match self {
            ResponseSink::None => Ok(()),
            ResponseSink::File {
                filename: _,
                file,
                format,
                delimiter: _,
            } => {
                let file_ref = Arc::clone(file);
                let mut file_attained = file_ref.lock().map_err(|e| {
                    CompassAppError::ReadOnlyPoisonError(format!(
                        "Could not aquire lock on output file: {}",
                        e
                    ))
                })?;

                let output_row = format.format_response(response)?;
                writeln!(file_attained, "{}", output_row).map_err(CompassAppError::IOError)?;
                Ok(())
            }
            ResponseSink::Combined(policies) => {
                for policy in policies {
                    policy.write_response(response)?;
                }
                Ok(())
            }
        }
    }

    pub fn close(&self) -> Result<String, CompassAppError> {
        match self {
            ResponseSink::None => Ok(String::from("")),
            ResponseSink::File {
                filename,
                file,
                format,
                delimiter: _,
            } => {
                let file_ref = Arc::clone(file);
                let mut file_attained = file_ref.lock().map_err(|e| {
                    CompassAppError::ReadOnlyPoisonError(format!(
                        "Could not aquire lock on output file: {}",
                        e
                    ))
                })?;

                let final_contents = format
                    .final_file_contents()
                    .unwrap_or_else(|| String::from(""));
                writeln!(file_attained, "{}", final_contents).map_err(CompassAppError::IOError)?;

                Ok(filename.clone())
            }
            ResponseSink::Combined(policies) => {
                let mut out_strs = vec![];
                for policy in policies {
                    let out_str = policy.close()?;
                    if !out_str.is_empty() {
                        out_strs.push(out_str);
                    }
                }

                Ok(out_strs.join(","))
            }
        }
    }
}
