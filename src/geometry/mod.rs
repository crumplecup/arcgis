//! Geometry conversion between ArcGIS JSON and geo-types.
//!
//! This module provides conversion functions between ArcGIS geometry JSON
//! and the Rust [`geo_types`](https://docs.rs/geo-types) types.

// Old types (will be removed in Phase 8)
mod arcgis;
mod convert;
pub use arcgis::*;
pub use convert::*;

// New types (replacing old) - exported with V2 suffix temporarily
mod new_errors;
mod new_spatial_ref;
mod new_types;

// Temporary V2 exports to avoid conflicts during migration
pub use new_errors::{
    ArcGISGeometryError as ArcGISGeometryErrorV2,
    ArcGISGeometryErrorKind as ArcGISGeometryErrorKindV2,
    GeoError,
    GeometryJsonError,
};
pub use new_spatial_ref::SpatialReference as SpatialReferenceV2;
pub use new_types::{
    ArcGISEnvelope as ArcGISEnvelopeV2,
    ArcGISGeometry as ArcGISGeometryV2,
    ArcGISMultipoint as ArcGISMultipointV2,
    ArcGISPoint as ArcGISPointV2,
    ArcGISPolygon as ArcGISPolygonV2,
    ArcGISPolyline as ArcGISPolylineV2,
};
