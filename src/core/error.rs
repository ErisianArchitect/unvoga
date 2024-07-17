use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("std::io error: {0}")]
    IoError(#[from]std::io::Error),
    #[error("From UTF-8 Error: {0}")]
    FromUtf8Error(#[from]std::string::FromUtf8Error),
    #[error("String too long")]
    StringTooLong,
    #[error("Array too long")]
    ArrayTooLong,
    #[error("Invalid binary format")]
    InvalidBinaryFormat,
    #[error("Chunk was too large and did not fit into the buffer")]
    ChunkTooLarge,
    #[error("File was too small to contain header")]
    NoHead,
    #[error("Path was not a file")]
    NotAFile,
    #[error("Allocation failed")]
    AllocationFailure,
    #[error("Chunk not found")]
    ChunkNotFound,
    #[error("Parent directory not found")]
    ParentNotFound,
    #[error("Temporary error")]
    Temporary,
}

pub type Result<T> = std::result::Result<T, Error>;