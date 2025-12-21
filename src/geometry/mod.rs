//! Geometry conversion between ArcGIS JSON and geo-types.
//!
//! This module provides conversion functions between ArcGIS geometry JSON
//! and the Rust [`geo_types`](https://docs.rs/geo-types) types.

pub mod convert;

pub use convert::*;

// TODO: Implement conversions for all geometry types
// TODO: Add spatial reference handling
// TODO: Add Z/M coordinate support
