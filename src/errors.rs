use thiserror::Error;

#[derive(Error, Debug)]
pub enum WallaceError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON Parsing Error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV Error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Failed to parse message type {log_type} ({name}): {reason}")]
    ParsingError {
        log_type: u16,
        name: String,
        reason: String,
    },

    #[error("Message type {0} not found in registry")]
    UnknownMessageType(u16),

    #[error("Failed to convert path to string: {path:?}")]
    PathConversionError { path: std::path::PathBuf },
    // Add more specific errors as needed
}

// Define a convenient Result type
pub type Result<T> = std::result::Result<T, WallaceError>;
