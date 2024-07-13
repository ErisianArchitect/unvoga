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
}

pub type Result<T> = std::result::Result<T, Error>;