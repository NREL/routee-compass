use super::response_output_format::ResponseOutputFormat;
use crate::app::compass::CompassAppError;
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
        iterations_per_flush: u64,
        iterations: Arc<Mutex<u64>>,
    },
    Combined(Vec<Box<ResponseSink>>),
}

impl ResponseSink {
    /// uses a writer
    pub fn write_response(&self, response: &mut serde_json::Value) -> Result<(), CompassAppError> {
        match self {
            ResponseSink::None => Ok(()),
            ResponseSink::File {
                filename,
                file,
                format,
                delimiter: _,
                iterations_per_flush,
                iterations,
            } => {
                let file_ref = Arc::clone(file);
                let mut file_attained = file_ref.lock().map_err(|e| {
                    CompassAppError::ReadOnlyPoisonError(format!(
                        "Could not aquire lock on output file: {}",
                        e
                    ))
                })?;
                let it_ref = Arc::new(iterations);
                let mut it_attained = it_ref.lock().map_err(|e| {
                    CompassAppError::ReadOnlyPoisonError(format!(
                        "Could not aquire lock on File::iterations: {}",
                        e
                    ))
                })?;

                let output_row = format.format_response(response)?;
                writeln!(file_attained, "{}", output_row).map_err(|e| {
                    CompassAppError::InternalError(format!(
                        "failure writing to {}: {}",
                        filename, e
                    ))
                })?;
                *it_attained += 1;
                if *it_attained % iterations_per_flush == 0 {
                    file_attained.flush().map_err(|e| {
                        CompassAppError::InternalError(format!(
                            "failure flushing output to {}: {}",
                            filename, e
                        ))
                    })?;
                }

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
                iterations_per_flush: _,
                iterations: _,
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
                writeln!(file_attained, "{}", final_contents).map_err(|e| {
                    CompassAppError::InternalError(format!(
                        "failure writing final contents to {}: {}",
                        filename, e
                    ))
                })?;

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
