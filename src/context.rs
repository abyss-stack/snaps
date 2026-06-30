use crate::outcome::AppError::{ColorizeError, IoError, JsonError};
use crate::outcome::{AppError, AppMessage, AppResult};
use json_color::Colorizer;
use serde::Serialize;
use std::io::Write;

pub struct AppContext {
    pub raw: bool,
}

impl AppContext {
    pub fn from_args(raw: bool) -> Self {
        Self { raw }
    }

    pub fn emit<T: Serialize, W: Write>(&self, value: &T, writer: &mut W) -> AppResult<()> {
        let payload = if self.raw {
            serde_json::to_string(value).map_err(|e| JsonError(e.to_string()))?
        } else {
            let json = serde_json::to_string_pretty(value).map_err(|e| JsonError(e.to_string()))?;
            let colorizer: Colorizer = Colorizer::arbitrary();
            colorizer
                .colorize_json_str(&json)
                .map_err(|_| ColorizeError)? // Use --raw if this fails.
        };

        writeln!(writer, "{}", payload).map_err(|e| IoError(e.to_string()))?;

        Ok(())
    }

    pub fn emit_message(&self, msg: &AppMessage) -> AppResult<()> {
        self.emit(msg, &mut std::io::stdout())
    }

    pub fn emit_error(&self, err: &AppError) -> AppResult<()> {
        self.emit(err, &mut std::io::stderr())
    }
}
