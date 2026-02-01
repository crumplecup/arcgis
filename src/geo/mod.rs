//! Geospatial data types and conversions.
//!
//! This module provides native support for ESRI JSON geometry formats and
//! seamless integration with the GeoRust ecosystem.
//!
//! # Quick Start
//!
//! ```rust
//! # #[cfg(feature = "geo")]
//! # {
//! use arcgis::EsriPoint;
//!
//! // Parse ESRI JSON
//! let json = r#"{"x": -118.2437, "y": 34.0522, "spatialReference": {"wkid": 4326}}"#;
//! let esri_point: EsriPoint = serde_json::from_str(json)?;
//!
//! // Convert to geo-types using From trait
//! let geo_point: geo_types::Point = esri_point.into();
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod errors;
pub mod esri;

// Re-exports only (no implementations in mod.rs)
pub use errors::{
    EsriGeometryError, EsriGeometryErrorKind, GeometryJsonError, GeoError,
};
pub use esri::{
    EsriEnvelope, EsriGeometry, EsriMultipoint, EsriPoint, EsriPolygon, EsriPolyline,
    SpatialReference,
};
