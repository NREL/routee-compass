use flate2::write::GzEncoder;
use std::{fs::File, io::Result, io::Write};

use crate::app::compass::{
    response::response_output_format::ResponseOutputFormat, CompassAppError,
};

pub enum InternalWriter {
    File { file: File },
    GzippedFile { encoder: GzEncoder<File> },
}

impl InternalWriter {
    pub fn write_header(
        &mut self,
        format: &ResponseOutputFormat,
    ) -> core::result::Result<(), CompassAppError> {
        let header = format
            .initial_file_contents()
            .unwrap_or_else(|| String::from(""));

        self.write(header.as_bytes()).map(|_| {}).map_err(|e| {
            CompassAppError::InternalError(format!("Failure writing header to file: {}", e))
        })
    }

    pub fn finish(&mut self) -> core::result::Result<(), CompassAppError> {
        match self {
            InternalWriter::File { ref mut file } => {
                file.flush().map_err(|e| {
                    CompassAppError::InternalError(format!("failure flushing output {}", e))
                })?;
                Ok(())
            }
            InternalWriter::GzippedFile { ref mut encoder } => {
                // NOTE: Because GzEncoder::finish requires ownership, we use try_finish instead.
                // Subsequent attempts to write to this file will panic!
                let _ = encoder.flush();
                encoder.try_finish().map_err(|e| {
                    CompassAppError::InternalError(format!(
                        "failure finishing encoded output {}",
                        e
                    ))
                })?;
                Ok(())
            }
        }
    }
}

impl Write for InternalWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            InternalWriter::File { file } => file.write(buf),
            InternalWriter::GzippedFile { encoder } => encoder.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            InternalWriter::File { file } => file.flush(),
            InternalWriter::GzippedFile { encoder } => encoder.flush(),
        }
    }
}
