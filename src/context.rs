use crate::outcome::AppError::{IoError, JsonError};
use crate::outcome::{AppError, AppMessage, AppResult};
use json_color::Colorizer;
use serde::Serialize;
use std::io::{self, Write};

pub struct AppContext {
    pub raw: bool,
}

impl AppContext {
    pub fn from_args(raw: bool) -> Self {
        Self { raw }
    }

    pub fn emit<T: Serialize>(&self, value: &T, writer: &mut dyn Write) -> AppResult<()> {
        let payload = if self.raw {
            serde_json::to_string(value).map_err(|e| JsonError(e.to_string()))?
        } else {
            let json = serde_json::to_string_pretty(value).map_err(|e| JsonError(e.to_string()))?;
            let colorizer = Colorizer::default();
            colorizer.colorize_json_str(&json).unwrap_or(json) // UNWRAP: uncolorized json as fallback.
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
