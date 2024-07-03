use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ByteStreamError {
    #[error(
        "Decoder unable to find the four-byte sequence 0x00 00 00 01 to signal end of preprocess and commence step-wise process."
    )]
    PreProcessTerminationMarkerNotFound,

    #[error("Invalid lane count for following workload: {0}")]
    InvalidLaneCount(String),

    #[error("the data for key `{0}` is not available")]
    Redaction(String),

    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
}

#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("File path not recognized: {0}")]
    UnrecognizedFilePath(String),

    #[error("An error occurred when opening the file")]
    FileError(#[from] io::Error),

    #[error("An error occurred reading from the nal unit stream buffer")]
    BytestreamError(#[from] ByteStreamError),
}
