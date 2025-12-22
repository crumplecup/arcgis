//! Error types for the ArcGIS SDK.

use tracing::instrument;

/// Specific error conditions for the ArcGIS SDK.
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display)]
pub enum ErrorKind {
    /// HTTP request error.
    #[display("HTTP request failed: {}", _0)]
    Http(String),

    /// Authentication error.
    #[display("Authentication failed: {}", _0)]
    Auth(String),

    /// ArcGIS API error with code and message.
    #[display("ArcGIS API error {}: {}", code, message)]
    Api {
        /// Error code from the API.
        code: i32,
        /// Error message from the API.
        message: String,
    },

    /// JSON serialization/deserialization error.
    #[display("JSON error: {}", _0)]
    Json(String),

    /// URL parsing error.
    #[display("Invalid URL: {}", _0)]
    Url(String),

    /// OAuth error.
    #[display("OAuth error: {}", _0)]
    OAuth(String),

    /// Geometry conversion error.
    #[display("Geometry conversion error: {}", _0)]
    Geometry(String),

    /// Validation error for invalid input.
    #[display("Validation error: {}", _0)]
    Validation(String),

    /// Generic error for other cases.
    #[display("{}", _0)]
    Other(String),
}

/// The main error type for the ArcGIS SDK with location tracking.
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
#[display("ArcGIS SDK: {} at {}:{}", kind, file, line)]
pub struct Error {
    /// The specific error condition.
    pub kind: ErrorKind,
    /// Line number where the error occurred.
    pub line: u32,
    /// File where the error occurred.
    pub file: &'static str,
}

impl Error {
    /// Creates a new error with the given kind and caller location.
    #[track_caller]
    #[instrument(skip(kind), fields(kind = %kind))]
    pub fn new(kind: ErrorKind) -> Self {
        let loc = std::panic::Location::caller();
        tracing::error!(error_kind = %kind, location = %loc, "Error created");
        Self {
            kind,
            line: loc.line(),
            file: loc.file(),
        }
    }

    /// Creates an API error with the given code and message.
    #[track_caller]
    #[instrument(fields(code, message))]
    pub fn api(code: i32, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Api {
            code,
            message: message.into(),
        })
    }

    /// Creates an authentication error.
    #[track_caller]
    #[instrument(skip(message), fields(message))]
    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Auth(message.into()))
    }

    /// Creates a geometry conversion error.
    #[track_caller]
    #[instrument(skip(message), fields(message))]
    pub fn geometry(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Geometry(message.into()))
    }

    /// Creates a validation error.
    #[track_caller]
    #[instrument(skip(message), fields(message))]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Validation(message.into()))
    }

    /// Creates an OAuth error.
    #[track_caller]
    #[instrument(skip(message), fields(message))]
    pub fn oauth(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::OAuth(message.into()))
    }
}

// Conversions from external error types
impl From<reqwest::Error> for Error {
    #[track_caller]
    fn from(err: reqwest::Error) -> Self {
        Self::new(ErrorKind::Http(err.to_string()))
    }
}

impl From<serde_json::Error> for Error {
    #[track_caller]
    fn from(err: serde_json::Error) -> Self {
        Self::new(ErrorKind::Json(err.to_string()))
    }
}

impl From<url::ParseError> for Error {
    #[track_caller]
    fn from(err: url::ParseError) -> Self {
        Self::new(ErrorKind::Url(err.to_string()))
    }
}
