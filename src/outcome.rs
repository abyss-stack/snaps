mod error;
mod message;

pub use error::AppError;
pub use message::AppMessage;

pub type AppResult<T> = Result<T, AppError>;
