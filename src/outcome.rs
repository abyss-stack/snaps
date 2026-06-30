use anyhow::Error as AnyhowError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
/*
AppError represents a fatal failure.
*/
#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppError {
    #[error("Could not read config from '{0}'.")]
    ConfigReadError(String),
    #[error("Unparsable json config.")]
    JsonParseError,
    #[error("Could not write fstab to '{0}'.")]
    FstabWriteError(String),
    #[error("Internal hash error")]
    InternalHashError,
    #[error("Could not open snaps dir: '{0}'.")]
    SnapsDirOpenFailed(String),
    #[error("Could not open source dir: '{0}'.")]
    SourceDirOpenFailed(String),
    #[error("Hash collision for: '{0}'.")]
    HashCollision(String),
    #[error("Could not convert to CString: '{0}'.")]
    CStringConvertError(String),
    #[error("Could not create dir for: '{0}'.")]
    HashDirCreateFailed(String),
    #[error("Kernel ioctl failure.")]
    KernelIoctlFailure,
    #[error("Could not get btrfs flags for: '{0}'.")]
    BtrfsGetFlagsError(String),
    #[error("Could not set btrfs flags for: '{0}'.")]
    BtrfsSetFlagsError(String),

    // INTENTIONAL: lazy way to wrap every error possible, still json-compatible
    #[error("General error: {0}")]
    GeneralError(String),
}

impl From<AnyhowError> for AppError {
    fn from(err: AnyhowError) -> Self {
        AppError::GeneralError(err.to_string())
    }
}

/*
AppMessage is a json-native part of output.
*/
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppMessage {}

/*
AppResult is the only result functions should return.
*/
pub type AppResult<T> = std::result::Result<T, AppError>;
