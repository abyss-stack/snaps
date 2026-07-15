use crate::outcome::AppError;

pub type AppResult<T> = Result<T, AppError>;
