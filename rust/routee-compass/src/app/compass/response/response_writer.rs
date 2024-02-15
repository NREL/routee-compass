use super::response_output_format::ResponseOutputFormat;
use crate::app::compass::compass_app_error::CompassAppError;
use std::io::prelude::*;
use std::{
    fs::File,
    sync::{Arc, Mutex},
};

pub enum ResponseWriter {
    RunBatchFileSink {
        filename: String,
        file: Arc<Mutex<File>>,
        format: ResponseOutputFormat,
        delimiter: Option<String>,
    },
}

impl ResponseWriter {
    pub fn write_response(&self, response: &serde_json::Value) -> Result<(), CompassAppError> {
        match self {
            ResponseWriter::RunBatchFileSink {
                filename: _,
                file,
                format,
                delimiter,
            } => {
                let file_ref = Arc::clone(file);
                let mut file_attained = file_ref.lock().map_err(|e| {
                    CompassAppError::ReadOnlyPoisonError(format!(
                        "Could not aquire lock on output file: {}",
                        e
                    ))
                })?;

                // if write_delimiter {
                //     match delimiter {
                //         None => {}
                //         Some(delim) => writeln!(file_attained, "{}", delim)
                //             .map_err(CompassAppError::IOError)?,
                //     }
                // }

                let output_row = format.format_response(response)?;
                writeln!(file_attained, "{}", output_row).map_err(CompassAppError::IOError)?;
                Ok(())
            }
        }
    }

    pub fn close(&self) -> Result<String, CompassAppError> {
        match self {
            ResponseWriter::RunBatchFileSink {
                filename,
                file,
                format,
                delimiter,
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
        }
    }
}
