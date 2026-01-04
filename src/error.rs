//! Error types for the ArcGIS SDK.

/// HTTP request error wrapper.
#[derive(Debug, derive_more::Display, derive_more::Error, derive_getters::Getters)]
#[display("HTTP request failed: {}", source)]
pub struct HttpError {
    /// The underlying reqwest error.
    source: reqwest::Error,
    /// Line number where the error occurred.
    line: u32,
    /// File where the error occurred.
    file: &'static str,
}

impl HttpError {
    /// Creates a new HTTP error with caller location.
    #[track_caller]
    pub fn new(source: reqwest::Error) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            source,
            line: loc.line(),
            file: loc.file(),
        }
    }
}

impl From<reqwest::Error> for HttpError {
    #[track_caller]
    fn from(source: reqwest::Error) -> Self {
        Self::new(source)
    }
}

/// JSON serialization/deserialization error wrapper.
#[derive(Debug, derive_more::Display, derive_more::Error, derive_getters::Getters)]
#[display("JSON error: {}", source)]
pub struct JsonError {
    /// The underlying serde_json error.
    source: serde_json::Error,
    /// Line number where the error occurred.
    line: u32,
    /// File where the error occurred.
    file: &'static str,
}

impl JsonError {
    /// Creates a new JSON error with caller location.
    #[track_caller]
    pub fn new(source: serde_json::Error) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            source,
            line: loc.line(),
            file: loc.file(),
        }
    }
}

impl From<serde_json::Error> for JsonError {
    #[track_caller]
    fn from(source: serde_json::Error) -> Self {
        Self::new(source)
    }
}

/// URL parsing error wrapper.
#[derive(Debug, derive_more::Display, derive_more::Error, derive_getters::Getters)]
#[display("Invalid URL: {}", source)]
pub struct UrlError {
    /// The underlying url::ParseError.
    source: url::ParseError,
    /// Line number where the error occurred.
    line: u32,
    /// File where the error occurred.
    file: &'static str,
}

impl UrlError {
    /// Creates a new URL error with caller location.
    #[track_caller]
    pub fn new(source: url::ParseError) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            source,
            line: loc.line(),
            file: loc.file(),
        }
    }
}

impl From<url::ParseError> for UrlError {
    #[track_caller]
    fn from(source: url::ParseError) -> Self {
        Self::new(source)
    }
}

/// File I/O error wrapper.
#[derive(Debug, derive_more::Display, derive_more::Error, derive_getters::Getters)]
#[display("I/O error: {}", source)]
pub struct IoError {
    /// The underlying std::io::Error.
    source: std::io::Error,
    /// Line number where the error occurred.
    line: u32,
    /// File where the error occurred.
    file: &'static str,
}

impl IoError {
    /// Creates a new I/O error with caller location.
    #[track_caller]
    pub fn new(source: std::io::Error) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            source,
            line: loc.line(),
            file: loc.file(),
        }
    }
}

impl From<std::io::Error> for IoError {
    #[track_caller]
    fn from(source: std::io::Error) -> Self {
        Self::new(source)
    }
}

/// Builder error wrapper for derive_builder errors.
#[derive(Debug, derive_more::Display, derive_more::Error, derive_getters::Getters)]
#[display("Builder error: {}", message)]
pub struct BuilderError {
    /// Error message from the builder.
    message: String,
    /// Line number where the error occurred.
    line: u32,
    /// File where the error occurred.
    file: &'static str,
}

impl BuilderError {
    /// Creates a new builder error with caller location.
    #[track_caller]
    pub fn new(message: impl Into<String>) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            message: message.into(),
            line: loc.line(),
            file: loc.file(),
        }
    }
}

impl From<String> for BuilderError {
    #[track_caller]
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

/// Specific error conditions for the ArcGIS SDK.
#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum ErrorKind {
    /// HTTP request error.
    #[display("{}", _0)]
    #[from]
    Http(HttpError),

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
    #[display("{}", _0)]
    #[from]
    Json(JsonError),

    /// URL parsing error.
    #[display("{}", _0)]
    #[from]
    Url(UrlError),

    /// File I/O error.
    #[display("{}", _0)]
    #[from]
    Io(IoError),

    /// Builder error from derive_builder.
    #[display("{}", _0)]
    #[from]
    Builder(BuilderError),

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

/// Macro to generate bridge From implementations for external errors.
///
/// This creates the conversion chain: ExternalError → WrapperError → ErrorKind → Error
///
/// # Example
/// ```ignore
/// bridge_error!(reqwest::Error => HttpError);
/// // Generates:
/// // impl From<reqwest::Error> for ErrorKind {
/// //     #[track_caller]
/// //     fn from(err: reqwest::Error) -> Self {
/// //         HttpError::from(err).into()
/// //     }
/// // }
/// ```
macro_rules! bridge_error {
    ($external:ty => $wrapper:ty) => {
        impl From<$external> for ErrorKind {
            #[track_caller]
            fn from(err: $external) -> Self {
                <$wrapper>::from(err).into()
            }
        }
    };
}

// Bridge From implementations to chain external errors through wrappers
bridge_error!(reqwest::Error => HttpError);
bridge_error!(serde_json::Error => JsonError);
bridge_error!(url::ParseError => UrlError);
bridge_error!(std::io::Error => IoError);

/// The main error type for the ArcGIS SDK.
///
/// This type wraps all error conditions and provides automatic conversion
/// from underlying error types through the `?` operator.
#[derive(Debug, derive_more::Display)]
#[display("ArcGIS SDK: {}", _0)]
pub struct Error(Box<ErrorKind>);

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &*self.0 {
            ErrorKind::Http(e) => Some(e.source()),
            ErrorKind::Json(e) => Some(e.source()),
            ErrorKind::Url(e) => Some(e.source()),
            ErrorKind::Io(e) => Some(e.source()),
            _ => None,
        }
    }
}

impl Error {
    /// Returns a reference to the underlying error kind.
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }
}

/// Macro to implement From<SourceError> for Error.
///
/// This creates the full conversion chain: SourceError → ErrorKind → Error
/// with proper location tracking and error logging.
///
/// # Example
/// ```ignore
/// error_from!(reqwest::Error);
/// // Generates:
/// // impl From<reqwest::Error> for Error {
/// //     #[track_caller]
/// //     fn from(err: reqwest::Error) -> Self {
/// //         let kind = ErrorKind::from(err);
/// //         tracing::error!(error_kind = %kind, "Error created");
/// //         Self(Box::new(kind))
/// //     }
/// // }
/// ```
macro_rules! error_from {
    ($source:ty) => {
        impl From<$source> for Error {
            #[track_caller]
            fn from(err: $source) -> Self {
                let kind = ErrorKind::from(err);
                tracing::error!(error_kind = %kind, "Error created");
                Self(Box::new(kind))
            }
        }
    };
}

// Implement From<ErrorKind> for Error
impl From<ErrorKind> for Error {
    #[track_caller]
    fn from(kind: ErrorKind) -> Self {
        tracing::error!(error_kind = %kind, "Error created");
        Self(Box::new(kind))
    }
}

// Implement From for all external error types
error_from!(reqwest::Error);
error_from!(serde_json::Error);
error_from!(url::ParseError);
error_from!(std::io::Error);
error_from!(BuilderError);
