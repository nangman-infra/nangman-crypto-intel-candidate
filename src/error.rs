use std::fmt::{Display, Formatter};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Aws(String),
    Config(String),
    Io(String),
    Json(String),
    Nats(String),
    Validation(String),
}

impl AppError {
    pub fn aws(message: impl Into<String>) -> Self {
        Self::Aws(message.into())
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn nats(message: impl Into<String>) -> Self {
        Self::Nats(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Aws(message) => write!(formatter, "aws error: {message}"),
            Self::Config(message) => write!(formatter, "config error: {message}"),
            Self::Io(message) => write!(formatter, "io error: {message}"),
            Self::Json(message) => write!(formatter, "json error: {message}"),
            Self::Nats(message) => write!(formatter, "nats error: {message}"),
            Self::Validation(message) => write!(formatter, "validation error: {message}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}
