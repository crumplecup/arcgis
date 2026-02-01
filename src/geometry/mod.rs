//! ArcGIS geometry types with native geo-types integration.
//!
//! This module provides type-safe geometry handling for ESRI JSON formats with
//! seamless interoperability with the Rust GeoRust ecosystem via
//! [`geo_types`](https://docs.rs/geo-types).
//!
//! # Features
//!
//! - **Type-safe geometry:** Private fields with builder pattern and getters
//! - **ESRI JSON compatibility:** Full serde support for ArcGIS REST API formats
//! - **Native conversions:** `From`/`TryFrom` traits for geo-types integration
//! - **Comprehensive errors:** Detailed error types for geometry operations
//! - **Spatial reference:** Unified SpatialReference handling (WKID/WKT)
//!
//! # Geometry Types
//!
//! | ArcGIS Type | geo-types Equivalent | Notes |
//! |-------------|---------------------|-------|
//! | [`ArcGISPoint`] | [`geo_types::Point`] | 2D/3D points with optional M values |
//! | [`ArcGISPolyline`] | [`geo_types::MultiLineString`] | Multi-path lines |
//! | [`ArcGISPolygon`] | [`geo_types::Polygon`] | Rings with holes |
//! | [`ArcGISMultipoint`] | [`geo_types::MultiPoint`] | Point collections |
//! | [`ArcGISEnvelope`] | [`geo_types::Rect`] | Bounding boxes |
//! | [`ArcGISGeometry`] | [`geo_types::Geometry`] | Enum of all types |
//!
//! # Examples
//!
//! ## Creating Geometries
//!
//! ```
//! use arcgis::ArcGISPoint;
//!
//! // Simple 2D point
//! let point = ArcGISPoint::new(-118.2437, 34.0522);
//!
//! // 3D point with elevation
//! let point_3d = ArcGISPoint::with_z(-118.2437, 34.0522, 100.0);
//! ```
//!
//! ## Converting to geo-types
//!
//! ```
//! use arcgis::ArcGISPoint;
//! use geo_types::Point;
//!
//! let arcgis_point = ArcGISPoint::new(-118.0, 34.0);
//! let geo_point: Point = arcgis_point.into();
//!
//! assert_eq!(geo_point.x(), -118.0);
//! assert_eq!(geo_point.y(), 34.0);
//! ```
//!
//! ## Converting from geo-types
//!
//! ```
//! use arcgis::ArcGISPoint;
//! use geo_types::Point;
//!
//! let geo_point = Point::new(-117.0, 33.0);
//! let arcgis_point: ArcGISPoint = geo_point.into();
//!
//! assert_eq!(*arcgis_point.x(), -117.0);
//! ```
//!
//! ## Error Handling
//!
//! ```
//! use arcgis::{ArcGISPolyline, ArcGISGeometryError};
//! use geo_types::MultiLineString;
//!
//! let polyline = ArcGISPolyline::new(vec![]); // Empty paths
//! let result: Result<MultiLineString, ArcGISGeometryError> = polyline.try_into();
//! assert!(result.is_err());
//! ```

mod errors;
mod spatial_ref;
mod types;

pub use errors::{ArcGISGeometryError, ArcGISGeometryErrorKind, GeoError, GeometryJsonError};
pub use spatial_ref::SpatialReference;
pub use types::{
    ArcGISEnvelope, ArcGISGeometry, ArcGISMultipoint, ArcGISPoint, ArcGISPolygon, ArcGISPolyline,
    GeometryType, SpatialRel,
};
