//! Geometry conversion between ArcGIS JSON and geo-types.
//!
//! This module provides conversion functions between ArcGIS geometry JSON
//! and the Rust [`geo_types`](https://docs.rs/geo-types) types.

pub mod arcgis;
pub mod convert;

pub use arcgis::*;
pub use convert::*;
