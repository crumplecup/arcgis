//! # arcgis
//!
//! A type-safe Rust SDK for the [ArcGIS REST API](https://developers.arcgis.com/rest/).
//!
//! This library provides strongly-typed interfaces to ArcGIS services with compile-time
//! guarantees. Instead of error-prone string constants, it uses Rust enums and newtypes
//! to make invalid states unrepresentable.
//!
//! ## Features
//!
//! - 🔒 **Type-safe**: Enums instead of strings - compile-time validation
//! - 🌍 **GeoRust integration**: Native `geo-types` support
//! - 🔐 **Authentication**: API Key and OAuth 2.0
//! - ⚡ **Async/await**: Built on `tokio` and `reqwest`
//! - 🎯 **Modular**: Optional services via Cargo features
//!
//! ## Quick Start
//!
//! ```no_run
//! use arcgis::{ArcGISClient, auth::ApiKeyAuth};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), arcgis::Error> {
//!     let auth = ApiKeyAuth::new("YOUR_API_KEY");
//!     let client = ArcGISClient::new(auth);
//!
//!     // Use the client to access ArcGIS services
//!     Ok(())
//! }
//! ```
//!
//! ## Type Safety
//!
//! This SDK enforces type safety throughout:
//!
//! ```rust
//! use arcgis::types::{GeometryType, SpatialRel};
//!
//! // ✅ Compile-time validated
//! let geom_type = GeometryType::Point;
//! let spatial_rel = SpatialRel::Intersects;
//!
//! // ❌ Won't compile
//! // let geom_type = "esriGeometryPoint";  // Wrong type!
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

// Re-export major dependencies for user convenience
pub use geo_types;
pub use geojson;

// Core modules (always available)
pub mod auth;
pub mod client;
pub mod error;
pub mod geometry;
pub mod types;

// Optional service modules (feature-gated)
// TODO: Implement these modules
// #[cfg(feature = "feature-service")]
// pub mod feature;

// #[cfg(feature = "map-service")]
// pub mod map;

// #[cfg(feature = "geocoding")]
// pub mod geocoding;

// Utility modules
mod util;

// Re-exports for convenience
pub use auth::AuthProvider;
pub use client::ArcGISClient;
pub use error::Error;

/// Result type alias using this crate's [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;
