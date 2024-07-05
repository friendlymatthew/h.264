use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ByteStreamError {
    #[error("Expected {expected:?}. Got: {got:?}.")]
    IncorrectByteSequence { expected: String, got: String },

    #[error("Unexpected halt to the byte stream occurred")]
    UnexpectedTermination(String),

    #[error("Expected the following pattern")]
    MisalignedIndices(String),

    #[error("Invalid lane count for following workload: {0}")]
    InvalidLaneCount(String),

    #[error("the data for key `{0}` is not available")]
    Redaction(String),

    #[error("Unexpected byte in byte stream sequence {0} {1}")]
    UnexpectedByte(String, String),
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
