use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Connection not open")]
    NotConnected,

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Server error (code={code}): {message}")]
    Server { code: i64, message: String },

    #[error("Request timeout: {0}")]
    Timeout(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Protobuf decode error: {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for AppError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        AppError::WebSocket(e.to_string())
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
