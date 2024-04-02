use super::{response_output_format::ResponseOutputFormat, response_sink::ResponseSink};
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
    None,
    File {
        filename: String,
        format: ResponseOutputFormat,
        file_flush_rate: Option<i64>,
    },
    Combined {
        policies: Vec<Box<ResponseOutputPolicy>>,
    },
}

impl ResponseOutputPolicy {
    /// creates an instance of a writer which writes responses to some destination.
    /// the act of building this writer may include writing initial content to some sink,
    /// such as a file header.
    pub fn build(&self) -> Result<ResponseSink, CompassAppError> {
        match self {
            ResponseOutputPolicy::None => Ok(ResponseSink::None),
            ResponseOutputPolicy::File {
                filename,
                format,
                file_flush_rate,
            } => {
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

                let iterations_per_flush = match file_flush_rate {
                    Some(rate) if *rate <= 0 => Err(CompassAppError::InvalidInput(format!(
                        "file policy iterations_per_flush must be positive, found {}",
                        rate
                    ))),
                    None => Ok(1),
                    Some(rate) => Ok(*rate as u64),
                }?;

                let iterations: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

                Ok(ResponseSink::File {
                    filename: filename.clone(),
                    file: file_shareable,
                    format: format.clone(),
                    delimiter: format.delimiter(),
                    iterations_per_flush,
                    iterations,
                })
            }
            ResponseOutputPolicy::Combined { policies } => {
                let policies = policies
                    .iter()
                    .map(|p| p.build().map(Box::new))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ResponseSink::Combined(policies))
            }
        }
    }
}
