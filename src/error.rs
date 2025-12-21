//! Error types for the ArcGIS SDK.

/// The main error type for the ArcGIS SDK.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP request error from reqwest.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Authentication error.
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// ArcGIS API error with code and message.
    #[error("ArcGIS API error {code}: {message}")]
    Api {
        /// Error code from the API.
        code: i32,
        /// Error message from the API.
        message: String,
    },

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// URL parsing error.
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// OAuth error.
    #[error("OAuth error: {0}")]
    OAuth(String),

    /// Geometry conversion error.
    #[error("Geometry conversion error: {0}")]
    Geometry(String),

    /// Validation error for invalid input.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Generic error for other cases.
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Creates an API error with the given code and message.
    pub fn api(code: i32, message: impl Into<String>) -> Self {
        Self::Api {
            code,
            message: message.into(),
        }
    }

    /// Creates an authentication error.
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    /// Creates a geometry conversion error.
    pub fn geometry(message: impl Into<String>) -> Self {
        Self::Geometry(message.into())
    }

    /// Creates a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }
}
