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
