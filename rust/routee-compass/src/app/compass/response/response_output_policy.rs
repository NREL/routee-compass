use super::{response_output_format::ResponseOutputFormat, response_writer::ResponseSink};
use crate::app::compass::compass_app_error::CompassAppError;
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ResponseOutputPolicy {
    NoOutput,
    File {
        filename: String,
        format: ResponseOutputFormat,
    },
}

impl ResponseOutputPolicy {
    /// creates an instance of a writer which writes responses to some destination.
    /// the act of building this writer may include writing initial content to some sink,
    /// such as a file header.
    pub fn build(&self) -> Result<ResponseSink, CompassAppError> {
        match self {
            ResponseOutputPolicy::NoOutput => Ok(ResponseSink::None),
            ResponseOutputPolicy::File { filename, format } => {
                let output_file_path = PathBuf::from(filename);

                // initialize the file
                let header = format
                    .initial_file_contents()
                    .unwrap_or_else(|| String::from(""));
                std::fs::write(&output_file_path, header)?;

                // open the file with the option to append to it
                let file = OpenOptions::new().append(true).open(&output_file_path)?;

                // wrap the file in a mutex so we can share it between threads
                let file_shareable = Arc::new(Mutex::new(file));

                Ok(ResponseSink::File {
                    filename: filename.clone(),
                    file: file_shareable,
                    format: format.clone(),
                    delimiter: format.delimiter(),
                })
            }
        }
    }
}
