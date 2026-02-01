//! ESRI geometry types and implementations.
//!
//! This module provides type-safe representations of ESRI JSON geometry formats.

mod geometry;
mod spatial_ref;

// Re-exports only (no implementations in mod.rs)
pub use geometry::{
    EsriEnvelope, EsriGeometry, EsriMultipoint, EsriPoint, EsriPolygon, EsriPolyline,
};
pub use spatial_ref::SpatialReference;
