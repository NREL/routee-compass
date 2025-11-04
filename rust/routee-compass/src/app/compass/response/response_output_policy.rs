use super::{
    response_output_format::ResponseOutputFormat, response_sink::ResponseSink,
    write_mode::WriteMode,
};
use crate::app::compass::{response::internal_writer::InternalWriter, CompassAppError};
use flate2::{write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ResponseOutputPolicy {
    #[default]
    None,
    File {
        /// destination file. may be a standard file suffix, or, if terminates with '.gz' will be gzip-encrypted.
        filename: String,
        /// file format to target
        format: ResponseOutputFormat,
        /// optional argument to specify the frequency (in rows) to flush data to the file
        file_flush_rate: Option<u64>,
        /// optional argument to specify if we expect to open, append, or overwrite data.
        write_mode: Option<WriteMode>,
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
                write_mode,
            } => {
                let wm = write_mode.clone().unwrap_or_default();
                let mut wrapped_file = get_or_create_file_writer(filename, &wm)?;
                wrapped_file.write_header(format)?;

                // wrap the file in a mutex so we can share it between threads
                let file_shareable = Arc::new(Mutex::new(wrapped_file));
                let iterations_per_flush = file_flush_rate.unwrap_or(1);
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

/// helper function to handle the various file type + write mode options
fn get_or_create_file_writer(
    filename: &str,
    write_mode: &WriteMode,
) -> Result<InternalWriter, CompassAppError> {
    let output_file_path = PathBuf::from(filename);
    if filename.ends_with(".gz") {
        let file = write_mode.open_file(&output_file_path)?;
        let encoder = GzEncoder::new(file, Compression::default());
        Ok(InternalWriter::GzippedFile { encoder })
    } else {
        let file = write_mode.open_file(&output_file_path)?;
        Ok(InternalWriter::File { file })
    }
}
