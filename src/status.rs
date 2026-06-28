use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppError {
    #[error("Failed to read greet file: {0}")]
    ReadGreetError(String),

    #[error("No greet file found at: {0}")]
    GreetFileNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppMessage {
    GreetShown(String),
}
