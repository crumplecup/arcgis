//! Error types for ESRI geometry operations.
//!
//! This module provides site-specific errors for geometry parsing, validation,
//! and conversion operations, following the project's error handling standards.

use derive_getters::Getters;
use derive_more::Display;

// ============================================================================
// External error wrappers (concrete types only - no dyn Error)
// ============================================================================

/// Wrapper for geo crate errors.
///
/// Preserves the original error message in the error chain.
#[derive(Debug, Clone, Display, Getters)]
#[display("Geo operation failed: {}", source)]
pub struct GeoError {
    /// Original geo crate error message.
    source: String,
}

impl GeoError {
    /// Creates a new GeoError wrapping a geo error.
    #[track_caller]
    pub fn new(source: impl std::fmt::Display) -> Self {
        Self {
            source: source.to_string(),
        }
    }
}

impl std::error::Error for GeoError {}

/// Wrapper for serde_json errors during geometry parsing.
#[derive(Debug, Clone, Display, Getters)]
#[display("JSON parsing failed: {}", source)]
pub struct GeometryJsonError {
    /// Original serde_json error message.
    source: String,
}

impl GeometryJsonError {
    /// Creates a new GeometryJsonError wrapping a serde_json::Error.
    #[track_caller]
    pub fn new(source: &serde_json::Error) -> Self {
        Self {
            source: source.to_string(),
        }
    }
}

impl std::error::Error for GeometryJsonError {}

// ============================================================================
// Site-specific error kind
// ============================================================================

/// Specific error conditions for ESRI geometry operations.
///
/// This enum contains all possible error types that can occur during
/// ESRI geometry parsing, validation, and conversion.
#[derive(Debug, Clone, Display)]
pub enum EsriGeometryErrorKind {
    /// Invalid geometry structure.
    #[display("Invalid geometry: {}", _0)]
    InvalidGeometry(String),

    /// Missing required coordinate data.
    #[display("Missing coordinate at {}", _0)]
    MissingCoordinate(String),

    /// Empty geometry where data expected.
    #[display("Empty {} in {}", _0, _1)]
    EmptyGeometry(String, String), // (geometry_type, context)

    /// Invalid coordinate values (NaN, Inf, out of range).
    #[display("Invalid coordinate value: {}", _0)]
    InvalidCoordinate(String),

    /// Coordinate array has wrong length.
    #[display("Expected {} coordinates, got {}", expected, actual)]
    InvalidCoordinateLength {
        /// Expected number of coordinates.
        expected: usize,
        /// Actual number of coordinates.
        actual: usize,
    },

    /// Wrapped geo crate error (preserves error chain).
    #[display("{}", _0)]
    Geo(GeoError),

    /// Wrapped JSON parsing error (preserves error chain).
    #[display("{}", _0)]
    Json(GeometryJsonError),
}

// ============================================================================
// Main error type (kind + location)
// ============================================================================

/// Error from ESRI geometry operations with source location tracking.
///
/// This error type wraps an [`EsriGeometryErrorKind`] and adds file/line
/// information for debugging using `#[track_caller]`.
#[derive(Debug, Clone, Display, Getters)]
#[display("ESRI Geometry error: {} at {}:{}", kind, file, line)]
pub struct EsriGeometryError {
    /// Specific error kind.
    kind: EsriGeometryErrorKind,

    /// Line number where error occurred.
    line: u32,

    /// Source file where error occurred.
    file: &'static str,
}

impl EsriGeometryError {
    /// Creates a new geometry error with caller location tracking.
    #[track_caller]
    pub fn new(kind: EsriGeometryErrorKind) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            kind,
            line: loc.line(),
            file: loc.file(),
        }
    }
}

// Manual implementation of Error trait
impl std::error::Error for EsriGeometryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            EsriGeometryErrorKind::Geo(e) => Some(e),
            EsriGeometryErrorKind::Json(e) => Some(e),
            // String variants have no source
            _ => None,
        }
    }
}

// ============================================================================
// Conversions from external errors
// ============================================================================

impl From<serde_json::Error> for EsriGeometryError {
    #[track_caller]
    fn from(err: serde_json::Error) -> Self {
        let kind = EsriGeometryErrorKind::Json(GeometryJsonError::new(&err));
        Self::new(kind)
    }
}
