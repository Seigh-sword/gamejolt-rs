use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[non_exhaustive]
pub enum GameJoltError {
    #[error("Game Jolt API error: {code} - {message}")]
    ApiError { code: i32, message: String },

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Rate limited. Retry after {retry_after:?}")]
    RateLimited { retry_after: Duration },

    #[error("Offline queue is full (capacity: {capacity})")]
    OfflineQueueFull { capacity: usize },

    #[error("Batch request failed with {count} errors: {errors:?}")]
    BatchError {
        count: usize,
        errors: Vec<GameJoltError>,
    },

    #[error("Invalid parameter: {parameter} - {reason}")]
    InvalidParameter { parameter: String, reason: String },

    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Session expired: {session_id}")]
    SessionExpired { session_id: String },

    #[error("User not authenticated")]
    UserNotAuthenticated,

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Permission denied: {action}")]
    PermissionDenied { action: String },

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Operation timeout after {timeout:?}")]
    Timeout { timeout: Duration },

    #[error("Queue persistence error: {0}")]
    QueuePersistenceError(String),

    #[error("Signature generation failed: {0}")]
    SignatureError(String),
}

impl GameJoltError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            GameJoltError::NetworkError(_)
                | GameJoltError::RateLimited { .. }
                | GameJoltError::Timeout { .. }
                | GameJoltError::InternalError(_)
        )
    }

    pub fn api_code(&self) -> Option<i32> {
        match self {
            GameJoltError::ApiError { code, .. } => Some(*code),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for GameJoltError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            GameJoltError::Timeout {
                timeout: Duration::from_secs(30),
            }
        } else if err.is_status() {
            GameJoltError::NetworkError(err.to_string())
        } else {
            GameJoltError::NetworkError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for GameJoltError {
    fn from(err: serde_json::Error) -> Self {
        GameJoltError::SerializationError(err.to_string())
    }
}

#[cfg(feature = "offline-queue")]
impl From<bincode::Error> for GameJoltError {
    fn from(err: bincode::Error) -> Self {
        GameJoltError::QueuePersistenceError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GameJoltError>;
