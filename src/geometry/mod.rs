//! Geometry conversion between ArcGIS JSON and geo-types.
//!
//! This module provides conversion functions between ArcGIS geometry JSON
//! and the Rust [`geo_types`](https://docs.rs/geo-types) types.

mod errors;
mod spatial_ref;
mod types;

pub use errors::{ArcGISGeometryError, ArcGISGeometryErrorKind, GeoError, GeometryJsonError};
pub use spatial_ref::SpatialReference;
pub use types::{
    ArcGISEnvelope, ArcGISGeometry, ArcGISMultipoint, ArcGISPoint, ArcGISPolygon, ArcGISPolyline,
};
