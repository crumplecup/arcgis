# ESRI Geometry Integration Plan

**Date:** 2026-01-31 (Original)
**Updated:** 2026-02-01
**Status:** ✅ IMPLEMENTATION COMPLETE
**Scope:** Transform SDK from "API wrapper" to "Geospatial Library + ESRI Service Provider"

---

## ✅ Implementation Complete (2026-02-01)

The geometry consolidation refactor has been successfully completed using an **incremental multi-phase approach** instead of the originally proposed single-phase integration. The implementation consolidated two parallel geometry implementations into a single, superior design while maintaining all original type names.

### Key Deviations from Original Plan

1. **Approach:** Multi-phase incremental migration (10 commits) instead of single-phase integration
2. **Feature Flag:** geo-types made a standard dependency (no feature flag) instead of optional
3. **Type Names:** Kept `ArcGISPoint` etc. from `geometry/` module (not `EsriPoint` from `geo/`)
4. **Module Structure:** Consolidated into single `src/geometry/` module (removed `src/geo/`)

### Final Architecture

```
src/geometry/
├── mod.rs           # Public API exports
├── types.rs         # ArcGISPoint, ArcGISPolyline, ArcGISPolygon, etc.
├── spatial_ref.rs   # SpatialReference (single canonical implementation)
└── errors.rs        # ArcGISGeometryError, ArcGISGeometryErrorKind
```

**Key Features Achieved:**
- ✅ Private fields with derive_getters/derive_setters
- ✅ Builder pattern via derive_builder
- ✅ Comprehensive error handling with derive_more
- ✅ Full tracing instrumentation (#[instrument])
- ✅ Native geo-types conversions (From/TryFrom traits)
- ✅ ESRI JSON serialization compatibility
- ✅ Zero inline tests (all in tests/ directory)

### Migration Summary

**Phase 0-2:** Branch setup, file copying, V2 type exposure
**Phase 3-7:** Incremental service and example migrations (180+ references)
**Phase 8:** Legacy code removal, V2 suffix cleanup
**Phase 9:** Test consolidation (moved to tests/ directory)
**Phase 10:** Final verification and documentation

**Total Changes:**
- 30+ files modified across services and examples
- 220+ references migrated from public fields to getters
- 3 SpatialReference implementations consolidated to 1
- All tests passing, zero clippy warnings

### User Migration Guide

See "Breaking Changes & Migration Guide" section below for detailed migration instructions.

---

---

## Executive Summary

Integrate native ESRI JSON geometry support with the GeoRust ecosystem to provide:

1. **Type-safe geometry handling** for all ESRI service responses
2. **Zero-friction spatial analysis** via GeoRust algorithms
3. **Format interoperability** (ESRI JSON ↔ GeoJSON ↔ WKT ↔ WKB ↔ geo-types)

**Key Insight:** We already depend on `geo-types` and `geojson`. Adding ESRI JSON support is a natural extension, not scope creep - it's completing the geospatial story.

---

## Architecture Overview

### Module Structure

```
src/
├── geo/                    # NEW MODULE
│   ├── mod.rs             # Public API, re-exports only
│   ├── esri/              # ESRI-specific types
│   │   ├── mod.rs         # Re-exports only
│   │   ├── geometry.rs    # Point, Polyline, Polygon, etc. + From impls
│   │   ├── feature.rs     # Feature, FeatureSet
│   │   ├── spatial_ref.rs # SpatialReference, WKID handling
│   │   └── serde.rs       # Custom deserializers
│   └── convert.rs         # TryFrom implementations (need Error access)
├── services/
│   ├── elevation/
│   │   ├── types.rs       # ENHANCED with geo methods
│   │   └── ...
│   └── ...
```

**Note on `From` implementations:** Since types have private fields, infallible `From` implementations must either:

1. Use public infallible constructors (e.g., `EsriPoint::new()`)
2. Use direct struct construction (requires being in same module as type)

Therefore, `From` impls should be in the same file as the type definitions (e.g., `geometry.rs`), while `TryFrom` impls that need error handling can be in `convert.rs`.

### Public API Surface

**All types exposed at crate root** (flat organization):

```rust
// src/lib.rs

// Geometry types (behind "geo" feature flag)
#[cfg(feature = "geo")]
pub use geo::esri::{
    // Core geometry types
    EsriGeometry,
    EsriPoint,
    EsriPolyline,
    EsriPolygon,
    EsriMultipoint,
    EsriEnvelope,

    // Feature types
    EsriFeature,
    EsriFeatureSet,
    Field,

    // Spatial reference
    SpatialReference,
};

// Geometry errors (behind "geo" feature flag)
#[cfg(feature = "geo")]
pub use geo::errors::{
    EsriGeometryError,
    EsriGeometryErrorKind,
    GeoError,
    GeometryJsonError,
};

// No custom conversion traits - using std From/TryFrom
```

**User imports** (simple and flat):
```rust
use arcgis::{EsriPoint, EsriPolyline, SpatialReference};
```

**NOT:**
```rust
use arcgis::geo::esri::EsriPoint;  // ❌ No nested paths
```

---

## Type Hierarchy

### 1. Geometry Types

**Design Decision:** Mirror ESRI's type system, not force into GeoJSON model.

**Encapsulation:** All types use private fields with derive-getters, derive-setters, derive_builder, and derive-new following project standards.

```rust
/// Core ESRI geometry enum matching ArcGIS REST API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum EsriGeometry {
    Point(EsriPoint),
    Polyline(EsriPolyline),
    Polygon(EsriPolygon),
    Multipoint(EsriMultipoint),
    Envelope(EsriEnvelope),
}

/// ESRI Point geometry with X/Y coordinates and optional Z/M values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters, derive_setters::Setters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct EsriPoint {
    /// X coordinate (longitude in geographic systems)
    x: f64,

    /// Y coordinate (latitude in geographic systems)
    y: f64,

    /// Z-value (elevation/height)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    z: Option<f64>,

    /// M-value (measure/distance along line)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    m: Option<f64>,

    /// Spatial reference system
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    spatial_reference: Option<SpatialReference>,
}

impl EsriPoint {
    /// Creates a simple point with X/Y coordinates in WGS84.
    #[instrument(skip_all, fields(x = %x, y = %y))]
    pub fn new(x: f64, y: f64) -> Self {
        tracing::debug!("Creating EsriPoint");
        Self {
            x,
            y,
            z: None,
            m: None,
            spatial_reference: Some(SpatialReference::wgs84()),
        }
    }

    /// Creates a point with elevation (Z value).
    #[instrument(skip_all, fields(x = %x, y = %y, z = %z))]
    pub fn with_z(x: f64, y: f64, z: f64) -> Self {
        tracing::debug!("Creating EsriPoint with elevation");
        Self {
            x,
            y,
            z: Some(z),
            m: None,
            spatial_reference: Some(SpatialReference::wgs84()),
        }
    }
}

/// ESRI Polyline with multiple paths.
///
/// Each path is a sequence of coordinates that can include Z and M values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters, derive_setters::Setters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct EsriPolyline {
    /// Multiple paths (array of coordinate arrays)
    /// - Simple: [[x,y], [x,y], ...]
    /// - With Z: [[x,y,z], [x,y,z], ...]
    /// - With M: [[x,y,m], [x,y,m], ...]
    /// - With ZM: [[x,y,z,m], [x,y,z,m], ...]
    paths: Vec<Vec<Vec<f64>>>,

    /// Spatial reference system
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    spatial_reference: Option<SpatialReference>,

    /// Cached detection of Z/M presence (computed on demand)
    #[serde(skip)]
    #[builder(default)]
    #[setters(skip)]
    has_z: Option<bool>,

    /// Cached detection of M presence (computed on demand)
    #[serde(skip)]
    #[builder(default)]
    #[setters(skip)]
    has_m: Option<bool>,
}

impl EsriPolyline {
    /// Detects if this polyline has Z values.
    #[instrument(skip(self))]
    pub fn has_z(&mut self) -> bool {
        if let Some(cached) = self.has_z {
            return cached;
        }

        let result = self.paths
            .first()
            .and_then(|path| path.first())
            .map(|coord| coord.len() >= 3)
            .unwrap_or(false);

        self.has_z = Some(result);
        tracing::debug!(has_z = result, "Detected Z values");
        result
    }

    /// Detects if this polyline has M values.
    #[instrument(skip(self))]
    pub fn has_m(&mut self) -> bool {
        if let Some(cached) = self.has_m {
            return cached;
        }

        let result = self.paths
            .first()
            .and_then(|path| path.first())
            .map(|coord| coord.len() == 4)
            .unwrap_or(false);

        self.has_m = Some(result);
        tracing::debug!(has_m = result, "Detected M values");
        result
    }
}

// Note: From implementations for geo-types → ESRI types will be in geometry.rs
// (same file as type definitions) to allow direct struct construction.
// TryFrom implementations (ESRI → geo-types) will be in convert.rs.
// See "Conversion Strategy" section below for full implementations.

/// ESRI Polygon with multiple rings.
///
/// First ring is the exterior boundary, subsequent rings are holes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters, derive_setters::Setters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct EsriPolygon {
    /// Multiple rings (first is exterior, rest are holes)
    /// Ring winding order determines exterior vs hole
    rings: Vec<Vec<Vec<f64>>>,

    /// Spatial reference system
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    spatial_reference: Option<SpatialReference>,

    /// Cached detection of Z/M presence
    #[serde(skip)]
    #[builder(default)]
    #[setters(skip)]
    has_z: Option<bool>,

    /// Cached detection of M presence
    #[serde(skip)]
    #[builder(default)]
    #[setters(skip)]
    has_m: Option<bool>,
}

/// ESRI Multipoint geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters, derive_setters::Setters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct EsriMultipoint {
    /// Array of points: [[x,y], ...] or [[x,y,z], ...]
    points: Vec<Vec<f64>>,

    /// Spatial reference system
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    spatial_reference: Option<SpatialReference>,
}

/// ESRI Envelope (bounding box).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters, derive_setters::Setters, derive_builder::Builder, derive_new::new)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct EsriEnvelope {
    /// Minimum X coordinate
    xmin: f64,

    /// Minimum Y coordinate
    ymin: f64,

    /// Maximum X coordinate
    xmax: f64,

    /// Maximum Y coordinate
    ymax: f64,

    /// Minimum Z coordinate
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[new(default)]
    zmin: Option<f64>,

    /// Maximum Z coordinate
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[new(default)]
    zmax: Option<f64>,

    /// Minimum M value
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[new(default)]
    mmin: Option<f64>,

    /// Maximum M value
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[new(default)]
    mmax: Option<f64>,

    /// Spatial reference system
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[new(default)]
    spatial_reference: Option<SpatialReference>,
}
```

### 2. Spatial Reference

**Encapsulation:** Private fields with derive-getters, instrumented methods with tracing.

```rust
use tracing::instrument;

/// ESRI Spatial Reference System.
///
/// Defines the coordinate system for geometry coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, derive_getters::Getters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
pub struct SpatialReference {
    /// Well-Known ID (e.g., 4326 for WGS84, 3857 for Web Mercator)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    wkid: Option<u32>,

    /// Latest WKID (for compatibility with updated definitions)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    latest_wkid: Option<u32>,

    /// Well-Known Text representation (alternative to WKID)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    wkt: Option<String>,
}

impl SpatialReference {
    /// Creates WGS84 (EPSG:4326) spatial reference.
    ///
    /// This is the standard geographic coordinate system used by GPS.
    pub const fn wgs84() -> Self {
        Self {
            wkid: Some(4326),
            latest_wkid: Some(4326),
            wkt: None,
        }
    }

    /// Creates Web Mercator (EPSG:3857) spatial reference.
    ///
    /// This is the standard projection used by web mapping applications.
    pub const fn web_mercator() -> Self {
        Self {
            wkid: Some(3857),
            latest_wkid: Some(3857),
            wkt: None,
        }
    }

    /// Checks if this is a geographic coordinate system (lat/lon).
    ///
    /// Geographic systems use angular units (degrees).
    #[instrument(skip(self), fields(wkid = ?self.wkid))]
    pub fn is_geographic(&self) -> bool {
        let result = matches!(self.wkid, Some(4326) | Some(4269) | Some(4267));
        tracing::debug!(is_geographic = result, "Checked coordinate system type");
        result
    }

    /// Checks if this is a projected coordinate system.
    ///
    /// Projected systems use linear units (meters, feet).
    #[instrument(skip(self), fields(wkid = ?self.wkid))]
    pub fn is_projected(&self) -> bool {
        let result = self.wkid.is_some() && !self.is_geographic();
        tracing::debug!(is_projected = result, "Checked coordinate system type");
        result
    }
}
```

### 3. Feature Types

```rust
/// ESRI Feature combining geometry and attributes.
#[derive(Debug, Clone, Serialize, Deserialize, derive_getters::Getters, derive_setters::Setters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
pub struct EsriFeature {
    /// Feature geometry (polymorphic)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    geometry: Option<EsriGeometry>,

    /// Feature attributes (arbitrary JSON key-value pairs)
    #[builder(default)]
    attributes: serde_json::Map<String, serde_json::Value>,
}

/// ESRI Feature Set (collection of features with metadata).
#[derive(Debug, Clone, Serialize, Deserialize, derive_getters::Getters, derive_setters::Setters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct EsriFeatureSet {
    /// Geometry type for all features ("esriGeometryPoint", etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    geometry_type: Option<String>,

    /// Spatial reference for all features
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    spatial_reference: Option<SpatialReference>,

    /// Array of features
    #[serde(default)]
    #[builder(default)]
    features: Vec<EsriFeature>,

    /// Field definitions (schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    fields: Option<Vec<Field>>,
}

/// Field definition describing an attribute column.
#[derive(Debug, Clone, Serialize, Deserialize, derive_getters::Getters, derive_new::new)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    /// Field name
    name: String,

    /// Display alias
    #[serde(skip_serializing_if = "Option::is_none")]
    #[new(default)]
    alias: Option<String>,

    /// Field type ("esriFieldTypeString", "esriFieldTypeInteger", etc.)
    #[serde(rename = "type")]
    field_type: String,
}
```

---

## Conversion Strategy

**Design Decision:** Use standard Rust `From`/`TryFrom` traits for all conversions. No custom traits.

This follows idiomatic Rust patterns and provides seamless integration with the ecosystem.

**Implementation Rules:**

- **`From` implementations must be infallible** - no `.unwrap()`, `.expect()`, or panics
- Use `TryFrom` for conversions that can fail (validation, missing data, etc.)
- `From` impls for geo-types → ESRI live in `geometry.rs` (need direct struct construction)
- `TryFrom` impls for ESRI → geo-types live in `convert.rs` (need error handling)
- All conversions are instrumented with tracing

### Conversion Matrix

All conversions use standard `From`/`TryFrom` traits.

| ESRI Type | geo-types Type | Conversion | Notes |
|-----------|----------------|------------|-------|
| `EsriPoint` | `geo_types::Point` | `From` | Infallible (X, Y only) |
| `EsriPoint` | `geo_types::Coord` | `From` | Infallible |
| `EsriPolyline` | `geo_types::MultiLineString` | `TryFrom` | Validates paths exist |
| `EsriPolygon` | `geo_types::Polygon` | `TryFrom` | Validates rings, handles holes |
| `EsriMultipoint` | `geo_types::MultiPoint` | `TryFrom` | Validates points |
| `EsriEnvelope` | `geo_types::Rect` | `From` | Infallible |
| `geo_types::Point` | `EsriPoint` | `From` | Defaults to WGS84 |
| `geo_types::LineString` | `EsriPolyline` | `From` | Single path |
| `geo_types::Polygon` | `EsriPolygon` | `From` | Handles holes |

**Z/M Values:** Stored in ESRI types but not converted to geo-types (which are 2D). Access via getters if needed.

### Implementation Examples

#### ESRI → geo-types (Infallible Conversions)

```rust
use crate::{Error, ErrorKind, EsriPoint, EsriPolyline, EsriPolygon, EsriEnvelope, SpatialReference};
use tracing::instrument;

// ============================================================================
// ESRI → geo-types (infallible)
// ============================================================================

/// Convert EsriPoint to geo_types::Point (X, Y only, ignores Z/M).
#[cfg(feature = "geo")]
impl From<EsriPoint> for geo_types::Point {
    #[instrument(skip(point), fields(x = %point.x(), y = %point.y()))]
    fn from(point: EsriPoint) -> Self {
        tracing::debug!("Converting EsriPoint to geo_types::Point");
        geo_types::Point::new(*point.x(), *point.y())
    }
}

/// Convert EsriPoint to geo_types::Coord.
#[cfg(feature = "geo")]
impl From<EsriPoint> for geo_types::Coord {
    fn from(point: EsriPoint) -> Self {
        geo_types::Coord {
            x: *point.x(),
            y: *point.y(),
        }
    }
}

/// Convert EsriEnvelope to geo_types::Rect.
#[cfg(feature = "geo")]
impl From<EsriEnvelope> for geo_types::Rect {
    #[instrument(skip(env))]
    fn from(env: EsriEnvelope) -> Self {
        tracing::debug!("Converting EsriEnvelope to geo_types::Rect");
        geo_types::Rect::new(
            geo_types::Coord { x: *env.xmin(), y: *env.ymin() },
            geo_types::Coord { x: *env.xmax(), y: *env.ymax() },
        )
    }
}

// ============================================================================
// ESRI → geo-types (fallible)
// ============================================================================

/// Convert EsriPolyline to geo_types::MultiLineString.
///
/// # Errors
///
/// Returns error if:
/// - Polyline has no paths
/// - Any path is empty
/// - Any coordinate has fewer than 2 values (X, Y)
#[cfg(feature = "geo")]
impl TryFrom<EsriPolyline> for geo_types::MultiLineString {
    type Error = Error;

    #[instrument(skip(polyline), fields(path_count = polyline.paths().len()))]
    fn try_from(polyline: EsriPolyline) -> Result<Self, Self::Error> {
        tracing::debug!("Converting EsriPolyline to MultiLineString");

        if polyline.paths().is_empty() {
            tracing::error!("Polyline has no paths");
            return Err(Error::from(ErrorKind::InvalidGeometry(
                "Polyline has no paths".to_string()
            )));
        }

        let line_strings: Result<Vec<_>, _> = polyline.paths()
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                if path.is_empty() {
                    tracing::error!(path_index = idx, "Path is empty");
                    return Err(Error::from(ErrorKind::InvalidGeometry(
                        format!("Path {} has no coordinates", idx)
                    )));
                }

                let coords: Result<Vec<_>, _> = path
                    .iter()
                    .enumerate()
                    .map(|(coord_idx, c)| {
                        if c.len() < 2 {
                            tracing::error!(
                                path_index = idx,
                                coord_index = coord_idx,
                                coord_len = c.len(),
                                "Coordinate missing X or Y"
                            );
                            return Err(Error::from(ErrorKind::InvalidGeometry(
                                format!("Coordinate at path[{}][{}] must have X and Y", idx, coord_idx)
                            )));
                        }
                        Ok(geo_types::Coord { x: c[0], y: c[1] })
                    })
                    .collect();

                coords.map(geo_types::LineString::new)
            })
            .collect();

        let result = geo_types::MultiLineString::new(line_strings?);
        tracing::debug!(line_string_count = result.0.len(), "Conversion successful");
        Ok(result)
    }
}

/// Convert EsriPolygon to geo_types::Polygon.
///
/// First ring becomes exterior, subsequent rings become holes.
///
/// # Errors
///
/// Returns error if:
/// - Polygon has no rings
/// - Any ring is empty
/// - Any coordinate has fewer than 2 values
#[cfg(feature = "geo")]
impl TryFrom<EsriPolygon> for geo_types::Polygon {
    type Error = Error;

    #[instrument(skip(polygon), fields(ring_count = polygon.rings().len()))]
    fn try_from(polygon: EsriPolygon) -> Result<Self, Self::Error> {
        tracing::debug!("Converting EsriPolygon to Polygon");

        if polygon.rings().is_empty() {
            tracing::error!("Polygon has no rings");
            return Err(Error::from(ErrorKind::InvalidGeometry(
                "Polygon has no rings".to_string()
            )));
        }

        // Helper to convert ring to LineString
        let convert_ring = |ring: &[Vec<f64>], ring_idx: usize| -> Result<geo_types::LineString, Error> {
            if ring.is_empty() {
                tracing::error!(ring_index = ring_idx, "Ring is empty");
                return Err(Error::from(ErrorKind::InvalidGeometry(
                    format!("Ring {} is empty", ring_idx)
                )));
            }

            let coords: Result<Vec<_>, _> = ring
                .iter()
                .enumerate()
                .map(|(coord_idx, c)| {
                    if c.len() < 2 {
                        tracing::error!(
                            ring_index = ring_idx,
                            coord_index = coord_idx,
                            "Coordinate missing X or Y"
                        );
                        return Err(Error::from(ErrorKind::InvalidGeometry(
                            format!("Coordinate at ring[{}][{}] must have X and Y", ring_idx, coord_idx)
                        )));
                    }
                    Ok(geo_types::Coord { x: c[0], y: c[1] })
                })
                .collect();

            Ok(geo_types::LineString::new(coords?))
        };

        // First ring is exterior
        let exterior = convert_ring(&polygon.rings()[0], 0)?;

        // Subsequent rings are holes
        let interiors: Result<Vec<_>, _> = polygon.rings()[1..]
            .iter()
            .enumerate()
            .map(|(idx, ring)| convert_ring(ring, idx + 1))
            .collect();

        let result = geo_types::Polygon::new(exterior, interiors?);
        tracing::debug!(
            hole_count = result.interiors().len(),
            "Polygon conversion successful"
        );
        Ok(result)
    }
}

// ============================================================================
// geo-types → ESRI (infallible)
// ============================================================================

/// Convert geo_types::Point to EsriPoint with WGS84 spatial reference.
#[cfg(feature = "geo")]
impl From<geo_types::Point> for EsriPoint {
    #[instrument(skip(point), fields(x = %point.x(), y = %point.y()))]
    fn from(point: geo_types::Point) -> Self {
        tracing::debug!("Converting geo_types::Point to EsriPoint");
        // Use constructor - infallible
        EsriPoint::new(point.x(), point.y())
    }
}

/// Convert geo_types::LineString to EsriPolyline (single path).
#[cfg(feature = "geo")]
impl From<geo_types::LineString> for EsriPolyline {
    #[instrument(skip(line), fields(coord_count = line.coords_count()))]
    fn from(line: geo_types::LineString) -> Self {
        tracing::debug!("Converting LineString to EsriPolyline");

        let path: Vec<Vec<f64>> = line
            .coords()
            .map(|c| vec![c.x, c.y])
            .collect();

        // Manual construction - infallible (no builder validation needed)
        Self {
            paths: vec![path],
            spatial_reference: Some(SpatialReference::wgs84()),
            has_z: Some(false),
            has_m: Some(false),
        }
    }
}

/// Convert geo_types::Polygon to EsriPolygon (exterior + holes).
#[cfg(feature = "geo")]
impl From<geo_types::Polygon> for EsriPolygon {
    #[instrument(skip(polygon), fields(hole_count = polygon.interiors().len()))]
    fn from(polygon: geo_types::Polygon) -> Self {
        tracing::debug!("Converting Polygon to EsriPolygon");

        let mut rings = Vec::new();

        // Exterior ring
        let exterior: Vec<Vec<f64>> = polygon
            .exterior()
            .coords()
            .map(|c| vec![c.x, c.y])
            .collect();
        rings.push(exterior);

        // Interior rings (holes)
        for interior in polygon.interiors() {
            let hole: Vec<Vec<f64>> = interior
                .coords()
                .map(|c| vec![c.x, c.y])
                .collect();
            rings.push(hole);
        }

        // Manual construction - infallible (no builder validation needed)
        Self {
            rings,
            spatial_reference: Some(SpatialReference::wgs84()),
            has_z: Some(false),
            has_m: Some(false),
        }
    }
}
```

---

## Error Handling Implementation

### Complete Error Module Structure

Following project error handling standards with site-specific errors and concrete types:

```rust
// src/geo/errors.rs

use derive_getters::Getters;
use derive_more::{Display, Error};

// ============================================================================
// External error wrappers (concrete types only)
// ============================================================================

/// Wrapper for geo crate errors.
///
/// Preserves the original geo::Error in the error chain.
#[derive(Debug, Clone, Display, Error, Getters)]
#[display("Geo operation failed: {}", source)]
pub struct GeoError {
    /// Original geo crate error.
    source: geo::Error,
}

impl GeoError {
    /// Creates a new GeoError wrapping a geo::Error.
    #[track_caller]
    pub fn new(source: geo::Error) -> Self {
        Self { source }
    }
}

/// Wrapper for serde_json errors during geometry parsing.
#[derive(Debug, Clone, Display, Error, Getters)]
#[display("JSON parsing failed: {}", source)]
pub struct GeometryJsonError {
    /// Original serde_json error.
    source: serde_json::Error,
}

impl GeometryJsonError {
    /// Creates a new GeometryJsonError wrapping a serde_json::Error.
    #[track_caller]
    pub fn new(source: serde_json::Error) -> Self {
        Self { source }
    }
}

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
    InvalidCoordinateLength { expected: usize, actual: usize },

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
#[derive(Debug, Clone, Display, Error, Getters)]
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

// Implement source() to defer to kind variants
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

impl From<geo::Error> for EsriGeometryError {
    #[track_caller]
    fn from(err: geo::Error) -> Self {
        let kind = EsriGeometryErrorKind::Geo(GeoError::new(err));
        Self::new(kind)
    }
}

impl From<serde_json::Error> for EsriGeometryError {
    #[track_caller]
    fn from(err: serde_json::Error) -> Self {
        let kind = EsriGeometryErrorKind::Json(GeometryJsonError::new(err));
        Self::new(kind)
    }
}
```

### Integration with Crate-Level Error

```rust
// src/error.rs

use crate::EsriGeometryError;

/// Crate-level error kind aggregating all error sources.
#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum ErrorKind {
    // ... existing variants

    /// ESRI geometry conversion/validation error.
    #[from]
    #[display("{}", _0)]
    Geometry(EsriGeometryError),
}

/// Main crate error type.
#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("{}", _0)]
pub struct Error(Box<ErrorKind>);

impl<T> From<T> for Error
where
    T: Into<ErrorKind>,
{
    #[track_caller]
    fn from(err: T) -> Self {
        let kind = err.into();
        tracing::error!(error_kind = %kind, "Error created");
        Self(Box::new(kind))
    }
}

/// Convenience type alias for Results.
pub type Result<T> = std::result::Result<T, Error>;
```

### Usage Examples

```rust
use crate::{EsriGeometryError, EsriGeometryErrorKind, Result};
use tracing::instrument;

#[instrument(skip(polyline))]
fn validate_polyline(polyline: &EsriPolyline) -> Result<()> {
    if polyline.paths().is_empty() {
        return Err(EsriGeometryError::new(
            EsriGeometryErrorKind::EmptyGeometry(
                "polyline".to_string(),
                "no paths provided".to_string()
            )
        ).into());
    }

    for (idx, path) in polyline.paths().iter().enumerate() {
        if path.is_empty() {
            return Err(EsriGeometryError::new(
                EsriGeometryErrorKind::EmptyGeometry(
                    "path".to_string(),
                    format!("path[{}]", idx)
                )
            ).into());
        }

        for (coord_idx, coord) in path.iter().enumerate() {
            if coord.len() < 2 {
                return Err(EsriGeometryError::new(
                    EsriGeometryErrorKind::InvalidCoordinateLength {
                        expected: 2,
                        actual: coord.len(),
                    }
                ).into());
            }
        }
    }

    Ok(())
}
```

---

## Service Integration

### Elevation Service Example

```rust
// In src/services/elevation/types.rs

use crate::{EsriFeatureSet, EsriGeometry, Error, ErrorKind, Result};
use tracing::instrument;

#[cfg(feature = "geo")]
impl ProfileResult {
    /// Extract elevation profile as LineString with Z values.
    ///
    /// Returns a MultiLineString where each LineString represents a profile path.
    /// Note: Z-values and M-values are lost in conversion (geo-types is 2D).
    /// Use `elevation_points()` to preserve elevation data.
    #[instrument(skip(self))]
    pub fn to_line_strings(&self) -> Result<geo_types::MultiLineString> {
        tracing::debug!("Extracting line strings from profile");

        let feature_set: EsriFeatureSet = serde_json::from_value(
            self.output_profile()
                .clone()
                .ok_or_else(|| Error::from(ErrorKind::MissingData("No output_profile".to_string())))?
        )?;

        let mut line_strings = Vec::new();

        for feature in feature_set.features() {
            if let Some(EsriGeometry::Polyline(polyline)) = feature.geometry() {
                // Use TryFrom for conversion
                let multi_ls: geo_types::MultiLineString = polyline.clone().try_into()?;
                line_strings.extend(multi_ls.0);
            }
        }

        let result = geo_types::MultiLineString::new(line_strings);
        tracing::debug!(line_string_count = result.0.len(), "Extracted line strings");
        Ok(result)
    }

    /// Extract elevation values as (distance, elevation) pairs.
    ///
    /// Preserves Z (elevation) and M (distance) values from profile.
    #[instrument(skip(self))]
    pub fn elevation_points(&self) -> Result<Vec<(f64, f64)>> {
        tracing::debug!("Extracting elevation points");

        let feature_set: EsriFeatureSet = serde_json::from_value(
            self.output_profile().clone()
                .ok_or_else(|| Error::from(ErrorKind::MissingData("No output_profile".to_string())))?
        )?;

        let mut points = Vec::new();

        for feature in feature_set.features() {
            if let Some(EsriGeometry::Polyline(polyline)) = feature.geometry() {
                for path in polyline.paths() {
                    for coord in path {
                        if coord.len() >= 4 {
                            // [x, y, z, m] where m is distance, z is elevation
                            points.push((coord[3], coord[2]));
                        }
                    }
                }
            }
        }

        tracing::debug!(point_count = points.len(), "Extracted elevation points");
        Ok(points)
    }
}

#[cfg(feature = "geo")]
impl ViewshedResult {
    /// Extract viewshed as Polygon.
    ///
    /// Returns the first polygon from the viewshed feature set.
    #[instrument(skip(self))]
    pub fn to_polygon(&self) -> Result<geo_types::Polygon> {
        tracing::debug!("Extracting polygon from viewshed");

        let feature_set: EsriFeatureSet = serde_json::from_value(
            self.output_viewshed().clone()
                .ok_or_else(|| Error::from(ErrorKind::MissingData("No output_viewshed".to_string())))?
        )?;

        // Get first feature's geometry
        let feature = feature_set.features().first()
            .ok_or_else(|| Error::from(ErrorKind::MissingData("No features in viewshed".to_string())))?;

        if let Some(EsriGeometry::Polygon(polygon)) = feature.geometry() {
            // Use TryFrom for conversion
            let result: geo_types::Polygon = polygon.clone().try_into()?;
            tracing::debug!("Extracted viewshed polygon");
            Ok(result)
        } else {
            tracing::error!("Feature geometry is not a polygon");
            Err(Error::from(ErrorKind::InvalidGeometry("Expected polygon geometry".to_string())))
        }
    }

    /// Extract all visible areas as MultiPolygon.
    ///
    /// Viewsheds can contain multiple disconnected visible areas.
    #[instrument(skip(self))]
    pub fn to_multi_polygon(&self) -> Result<geo_types::MultiPolygon> {
        tracing::debug!("Extracting multi-polygon from viewshed");

        let feature_set: EsriFeatureSet = serde_json::from_value(
            self.output_viewshed().clone()
                .ok_or_else(|| Error::from(ErrorKind::MissingData("No output_viewshed".to_string())))?
        )?;

        let mut polygons = Vec::new();

        for feature in feature_set.features() {
            if let Some(EsriGeometry::Polygon(polygon)) = feature.geometry() {
                // Use TryFrom for conversion
                let poly: geo_types::Polygon = polygon.clone().try_into()?;
                polygons.push(poly);
            }
        }

        let result = geo_types::MultiPolygon::new(polygons);
        tracing::debug!(polygon_count = result.0.len(), "Extracted multi-polygon");
        Ok(result)
    }
}

#[cfg(feature = "geo")]
impl SummarizeElevationResult {
    /// Extract analysis area as Polygon.
    #[instrument(skip(self))]
    pub fn to_polygon(&self) -> Result<geo_types::Polygon> {
        tracing::debug!("Extracting polygon from elevation summary");

        let feature_set: EsriFeatureSet = serde_json::from_value(
            self.output_summary().clone()
                .ok_or_else(|| Error::from(ErrorKind::MissingData("No output_summary".to_string())))?
        )?;

        let feature = feature_set.features().first()
            .ok_or_else(|| Error::from(ErrorKind::MissingData("No features in summary".to_string())))?;

        if let Some(EsriGeometry::Polygon(polygon)) = feature.geometry() {
            // Use TryFrom for conversion
            let result: geo_types::Polygon = polygon.clone().try_into()?;
            tracing::debug!("Extracted summary polygon");
            Ok(result)
        } else {
            tracing::error!("Feature geometry is not a polygon");
            Err(Error::from(ErrorKind::InvalidGeometry("Expected polygon geometry".to_string())))
        }
    }
}
```

### Feature Service Example

```rust
// In src/services/feature/types.rs

use crate::{EsriGeometry, EsriPoint, Result};
use tracing::instrument;

#[cfg(feature = "geo")]
impl QueryResult {
    /// Extract features as typed geometries using TryFrom.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Extract all points
    /// let points: Vec<geo_types::Point> = result.to_geometries()?;
    /// ```
    #[instrument(skip(self))]
    pub fn to_geometries<T>(&self) -> Result<Vec<T>>
    where
        T: TryFrom<EsriGeometry, Error = crate::Error>
    {
        tracing::debug!("Extracting typed geometries");

        self.features()
            .iter()
            .filter_map(|f| f.geometry().as_ref())
            .map(|g| T::try_from(g.clone()))
            .collect()
    }

    /// Extract all points (if this is a point layer).
    ///
    /// Filters features to only include points, converting each to geo_types::Point.
    #[instrument(skip(self))]
    pub fn to_points(&self) -> Result<Vec<geo_types::Point>> {
        tracing::debug!("Extracting points from features");

        let points: Vec<_> = self.features()
            .iter()
            .filter_map(|f| {
                if let Some(EsriGeometry::Point(pt)) = f.geometry() {
                    // Use From for infallible conversion
                    Some(pt.clone().into())
                } else {
                    None
                }
            })
            .collect();

        tracing::debug!(point_count = points.len(), "Extracted points");
        Ok(points)
    }

    /// Extract as GeoJSON FeatureCollection.
    ///
    /// Converts ESRI features to GeoJSON format for interoperability.
    #[instrument(skip(self))]
    pub fn to_geojson(&self) -> Result<geojson::FeatureCollection> {
        tracing::debug!("Converting features to GeoJSON");

        // Convert ESRI features to GeoJSON features
        let features: Result<Vec<_>> = self.features()
            .iter()
            .map(|esri_feature| {
                let geometry = esri_feature.geometry()
                    .as_ref()
                    .map(|g| {
                        // Convert ESRI geometry to geo-types, then to GeoJSON
                        match g {
                            EsriGeometry::Point(pt) => {
                                let geo_pt: geo_types::Point = pt.clone().into();
                                Ok(geojson::Geometry::from(&geo_pt))
                            }
                            EsriGeometry::Polyline(polyline) => {
                                let multi_ls: geo_types::MultiLineString = polyline.clone().try_into()?;
                                Ok(geojson::Geometry::from(&multi_ls))
                            }
                            EsriGeometry::Polygon(polygon) => {
                                let poly: geo_types::Polygon = polygon.clone().try_into()?;
                                Ok(geojson::Geometry::from(&poly))
                            }
                            EsriGeometry::Multipoint(mp) => {
                                // TODO: Implement TryFrom for Multipoint
                                Err(crate::Error::from(crate::ErrorKind::InvalidGeometry(
                                    "Multipoint conversion not yet implemented".to_string()
                                )))
                            }
                            EsriGeometry::Envelope(env) => {
                                let rect: geo_types::Rect = env.clone().into();
                                // Convert rect to polygon for GeoJSON
                                let poly = geo_types::Polygon::new(
                                    geo_types::LineString::from(vec![
                                        rect.min(),
                                        geo_types::Coord { x: rect.max().x, y: rect.min().y },
                                        rect.max(),
                                        geo_types::Coord { x: rect.min().x, y: rect.max().y },
                                        rect.min(),
                                    ]),
                                    vec![],
                                );
                                Ok(geojson::Geometry::from(&poly))
                            }
                        }
                    })
                    .transpose()?;

                Ok(geojson::Feature {
                    bbox: None,
                    geometry,
                    id: None,
                    properties: Some(esri_feature.attributes().clone()),
                    foreign_members: None,
                })
            })
            .collect();

        let feature_collection = geojson::FeatureCollection {
            bbox: None,
            features: features?,
            foreign_members: None,
        };

        tracing::debug!(
            feature_count = feature_collection.features.len(),
            "Converted to GeoJSON"
        );
        Ok(feature_collection)
    }
}
```

---

## Feature Flags

```toml
[features]
default = []

# Spatial analysis with geo crate (area, distance, contains, etc.)
geo = ["dep:geo"]

# Format conversions (WKT, WKB, GeoJSON, MVT, etc.)
geozero = ["dep:geozero", "geo"]

# Everything enabled
geo-full = ["geo", "geozero"]

[dependencies]
# Always included (zero cost if unused)
geo-types = "0.7"
geojson = { version = "0.24", features = ["geo-types"] }

# Optional spatial analysis
geo = { version = "0.32", optional = true, features = ["use-serde"] }

# Optional format conversions
geozero = { version = "0.15", optional = true, features = ["with-wkb", "with-wkt"] }
```

**Usage:**

```toml
# Just API access (minimal)
arcgis = "0.1"

# With spatial analysis
arcgis = { version = "0.1", features = ["geo"] }

# With all geo features
arcgis = { version = "0.1", features = ["geo-full"] }
```

---

## Testing Strategy

### Test Standards (Project Requirements)

All tests must follow these standards:

1. **Return Type**: Use `anyhow::Result<()>` for tests that can error
   - No `.unwrap()` or `.expect()` - use `?` operator
   - Exercises library error types through anyhow conversion

2. **Tracing Initialization**: Call `init_tracing()` helper at start of each test

   ```rust
   fn init_tracing() {
       let _ = tracing_subscriber::fmt()
           .with_env_filter(
               tracing_subscriber::EnvFilter::try_from_default_env()
                   .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"))
           )
           .with_test_writer()
           .try_init();
   }
   ```

3. **Observability**: Use tracing throughout tests
   - `info!()` for major test steps and results
   - `debug!()` for detailed state inspection
   - Structured fields for important values
   - Makes debugging test failures much easier

4. **Assertions**: Use descriptive assertion messages

   ```rust
   assert!(
       condition,
       "Expected X but got Y: details"
   );
   ```

5. **Feature Gating**: API tests use `#[cfg(all(feature = "geo", feature = "test-location"))]`

### Unit Tests

**Test Standards:**

- Return `anyhow::Result<()>` for all tests that can error
- Initialize tracing with test helper at start of each test
- Use tracing for observability during test execution
- No `.unwrap()` or `.expect()` - use `?` operator
- Exercise library error types through anyhow conversion

```rust
// tests/geo_conversion_test.rs

use arcgis::{EsriPoint, EsriPolyline, EsriPolygon, EsriPolygonBuilder, EsriPolylineBuilder, SpatialReference};
use anyhow::Result;
use tracing::info;

/// Helper to initialize tracing for tests
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"))
        )
        .with_test_writer()
        .try_init();
}

#[test]
fn test_esri_point_roundtrip() -> Result<()> {
    init_tracing();
    info!("Testing ESRI Point roundtrip conversion");

    let esri_pt = EsriPoint::with_z(-118.2437, 34.0522, 100.0);
    info!(x = %esri_pt.x(), y = %esri_pt.y(), z = ?esri_pt.z(), "Created EsriPoint");

    // ESRI → geo-types using From
    let geo_pt: geo_types::Point = esri_pt.clone().into();
    info!(x = %geo_pt.x(), y = %geo_pt.y(), "Converted to geo_types::Point");
    assert_eq!(geo_pt.x(), -118.2437);
    assert_eq!(geo_pt.y(), 34.0522);
    // Note: Z value is lost (geo-types is 2D)

    // geo-types → ESRI using From
    let back_to_esri: EsriPoint = geo_pt.into();
    info!("Converted back to EsriPoint");
    assert_eq!(*back_to_esri.x(), -118.2437);
    assert_eq!(*back_to_esri.y(), 34.0522);
    assert_eq!(*back_to_esri.z(), None); // Z not preserved

    info!("Roundtrip conversion successful");
    Ok(())
}

#[test]
fn test_polyline_to_multilinestring() -> Result<()> {
    init_tracing();
    info!("Testing EsriPolyline to MultiLineString conversion");

    let polyline = EsriPolylineBuilder::default()
        .paths(vec![
            vec![
                vec![-118.0, 34.0],
                vec![-118.1, 34.1],
                vec![-118.2, 34.2],
            ]
        ])
        .spatial_reference(SpatialReference::wgs84())
        .build()?; // Use ? instead of expect

    info!(path_count = polyline.paths().len(), "Created EsriPolyline");

    // ESRI → geo-types using TryFrom
    let multi_ls: geo_types::MultiLineString = polyline.try_into()?; // Use ? instead of unwrap
    info!(line_string_count = multi_ls.0.len(), "Converted to MultiLineString");

    assert_eq!(multi_ls.0.len(), 1);
    assert_eq!(multi_ls.0[0].coords_count(), 3);

    info!("Conversion successful");
    Ok(())
}

#[test]
fn test_polygon_with_holes() -> Result<()> {
    init_tracing();
    info!("Testing EsriPolygon with holes conversion");

    let polygon = EsriPolygonBuilder::default()
        .rings(vec![
            // Exterior ring
            vec![
                vec![0.0, 0.0],
                vec![10.0, 0.0],
                vec![10.0, 10.0],
                vec![0.0, 10.0],
                vec![0.0, 0.0],
            ],
            // Interior ring (hole)
            vec![
                vec![2.0, 2.0],
                vec![8.0, 2.0],
                vec![8.0, 8.0],
                vec![2.0, 8.0],
                vec![2.0, 2.0],
            ],
        ])
        .spatial_reference(SpatialReference::wgs84())
        .build()?; // Use ? instead of expect

    info!(ring_count = polygon.rings().len(), "Created EsriPolygon with holes");

    // ESRI → geo-types using TryFrom
    let geo_polygon: geo_types::Polygon = polygon.try_into()?; // Use ? instead of unwrap
    info!(hole_count = geo_polygon.interiors().len(), "Converted to Polygon");

    assert_eq!(geo_polygon.interiors().len(), 1);

    info!("Polygon with holes conversion successful");
    Ok(())
}

#[test]
fn test_polyline_empty_paths_error() -> Result<()> {
    init_tracing();
    info!("Testing EsriPolyline empty paths error handling");

    let polyline = EsriPolylineBuilder::default()
        .paths(vec![]) // Empty!
        .build()?;

    info!("Created empty polyline, attempting conversion");

    // Should fail conversion - exercise error type
    let result: Result<geo_types::MultiLineString, _> = polyline.try_into();

    match result {
        Ok(_) => {
            info!("ERROR: Empty polyline conversion should have failed");
            anyhow::bail!("Expected conversion to fail for empty polyline");
        }
        Err(e) => {
            info!(error = %e, "Conversion correctly failed");
            // Verify it's the right error
            assert!(e.to_string().contains("no paths"));
            Ok(())
        }
    }
}

#[test]
fn test_geo_to_esri_conversions() -> Result<()> {
    init_tracing();
    info!("Testing geo-types to ESRI conversions");

    // Point
    let geo_pt = geo_types::Point::new(-120.0, 38.0);
    info!(x = %geo_pt.x(), y = %geo_pt.y(), "Created geo_types::Point");

    let esri_pt: EsriPoint = geo_pt.into();
    info!("Converted to EsriPoint");
    assert_eq!(*esri_pt.x(), -120.0);
    assert_eq!(*esri_pt.y(), 38.0);

    // LineString
    let geo_line = geo_types::LineString::from(vec![
        geo_types::Coord { x: -120.0, y: 38.0 },
        geo_types::Coord { x: -119.0, y: 39.0 },
    ]);
    info!(coord_count = geo_line.coords_count(), "Created LineString");

    let esri_polyline: EsriPolyline = geo_line.into();
    info!(path_count = esri_polyline.paths().len(), "Converted to EsriPolyline");
    assert_eq!(esri_polyline.paths().len(), 1);
    assert_eq!(esri_polyline.paths()[0].len(), 2);

    // Polygon
    let geo_polygon = geo_types::Polygon::new(
        geo_types::LineString::from(vec![
            geo_types::Coord { x: 0.0, y: 0.0 },
            geo_types::Coord { x: 10.0, y: 0.0 },
            geo_types::Coord { x: 10.0, y: 10.0 },
            geo_types::Coord { x: 0.0, y: 10.0 },
            geo_types::Coord { x: 0.0, y: 0.0 },
        ]),
        vec![],
    );
    info!(hole_count = geo_polygon.interiors().len(), "Created Polygon");

    let esri_polygon: EsriPolygon = geo_polygon.into();
    info!(ring_count = esri_polygon.rings().len(), "Converted to EsriPolygon");
    assert_eq!(esri_polygon.rings().len(), 1);

    info!("All geo-types to ESRI conversions successful");
    Ok(())
}

#[test]
fn test_coordinate_validation_errors() -> Result<()> {
    init_tracing();
    info!("Testing coordinate validation error handling");

    // Polyline with invalid coordinates (missing Y)
    let invalid_polyline = EsriPolylineBuilder::default()
        .paths(vec![
            vec![
                vec![-118.0], // Only X, missing Y!
                vec![-118.1, 34.1],
            ]
        ])
        .build()?;

    info!("Created polyline with invalid coordinates");

    let result: Result<geo_types::MultiLineString, _> = invalid_polyline.try_into();

    match result {
        Ok(_) => anyhow::bail!("Expected validation error for invalid coordinates"),
        Err(e) => {
            info!(error = %e, "Validation correctly caught invalid coordinates");
            assert!(e.to_string().contains("X and Y"));
            Ok(())
        }
    }
}
```

### Integration Tests

**Integration Test Standards:**

- Return `anyhow::Result<()>` for async tests
- Initialize tracing at start of test
- Use tracing throughout for observability
- No `.unwrap()` or `.expect()` - use `?` operator
- Feature-gated with `test-location` for API tests

```rust
// tests/geo_integration_test.rs

use arcgis::{ApiKeyAuth, ApiKeyTier, ArcGISClient, ElevationClient};
use arcgis::{ProfileParametersBuilder, ViewshedParametersBuilder};
use anyhow::Result;
use tracing::{info, debug};

/// Helper to initialize tracing for tests
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"))
        )
        .with_test_writer()
        .try_init();
}

#[tokio::test]
#[cfg(all(feature = "geo", feature = "test-location"))]
async fn test_elevation_profile_to_geo() -> Result<()> {
    init_tracing();
    info!("Testing elevation profile to geo-types conversion");

    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let elevation = ElevationClient::new(&client);
    debug!("Created elevation client");

    let params = ProfileParametersBuilder::default()
        .input_geometry(r#"{"paths":[[[-119.5,37.8],[-119.4,37.9]]]}"#)
        .geometry_type("esriGeometryPolyline")
        .build()?;

    info!("Requesting elevation profile from API");
    let result = elevation.profile(params).await?;

    debug!(
        first_z = ?result.first_point_z(),
        last_z = ?result.last_point_z(),
        "Received profile result"
    );

    // Extract as geo-types (uses TryFrom internally)
    info!("Converting profile to LineStrings");
    let line_strings = result.to_line_strings()?;
    info!(line_string_count = line_strings.0.len(), "Converted to geo-types");
    assert!(!line_strings.0.is_empty());

    // Extract elevation points (preserves Z/M values)
    info!("Extracting elevation points");
    let points = result.elevation_points()?;
    info!(point_count = points.len(), "Extracted elevation data");
    assert!(!points.is_empty());

    // Verify first point has reasonable elevation (Yosemite area ~2000-3000m)
    let (_distance, elevation) = points[0];
    info!(elevation_meters = elevation, "First point elevation");
    assert!(
        elevation > 1000.0 && elevation < 4000.0,
        "Elevation {} outside expected range for Yosemite area",
        elevation
    );

    info!("Elevation profile conversion test successful");
    Ok(())
}

#[tokio::test]
#[cfg(all(feature = "geo", feature = "test-location"))]
async fn test_viewshed_to_polygon() -> Result<()> {
    init_tracing();
    info!("Testing viewshed to polygon conversion");

    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let elevation = ElevationClient::new(&client);
    debug!("Created elevation client");

    let params = ViewshedParametersBuilder::default()
        .input_points(r#"{"points":[[-119.5,37.8]]}"#)
        .geometry_type("esriGeometryMultipoint")
        .maximum_distance(5000.0)
        .build()?;

    info!("Requesting viewshed analysis from API");
    let result = elevation.viewshed(params).await?;

    debug!(
        visible_area = ?result.visible_area(),
        total_area = ?result.total_area(),
        "Received viewshed result"
    );

    // Extract as geo-types polygon (uses TryFrom internally)
    info!("Converting viewshed to Polygon");
    let polygon = result.to_polygon()?;
    info!("Converted to geo-types::Polygon");

    // Verify polygon has area
    use geo::Area;
    let area = polygon.unsigned_area();
    info!(area_sq_meters = area, "Calculated polygon area");

    assert!(area > 0.0, "Polygon should have positive area");

    let area_km2 = area / 1_000_000.0;
    info!(area_km2 = %format!("{:.2}", area_km2), "Viewshed visible area");

    info!("Viewshed polygon conversion test successful");
    Ok(())
}

#[tokio::test]
#[cfg(all(feature = "geo", feature = "test-location"))]
async fn test_viewshed_multi_polygon() -> Result<()> {
    init_tracing();
    info!("Testing viewshed to multi-polygon conversion");

    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let elevation = ElevationClient::new(&client);

    let params = ViewshedParametersBuilder::default()
        .input_points(r#"{"points":[[-119.5,37.8]]}"#)
        .geometry_type("esriGeometryMultipoint")
        .maximum_distance(10000.0)
        .build()?;

    info!("Requesting viewshed analysis");
    let result = elevation.viewshed(params).await?;

    // Extract as multi-polygon (may have multiple disconnected visible areas)
    info!("Converting viewshed to MultiPolygon");
    let multi_polygon = result.to_multi_polygon()?;

    info!(
        polygon_count = multi_polygon.0.len(),
        "Extracted visible area polygons"
    );

    assert!(!multi_polygon.0.is_empty(), "Should have at least one visible area");

    // Calculate total visible area
    use geo::Area;
    let total_area: f64 = multi_polygon.0
        .iter()
        .map(|p| p.unsigned_area())
        .sum();

    info!(
        total_visible_area_km2 = %format!("{:.2}", total_area / 1_000_000.0),
        "Total visible area"
    );

    info!("Multi-polygon conversion test successful");
    Ok(())
}
```

### Real-World Example Tests

```rust
// tests/real_world_analysis_test.rs

use arcgis::{ApiKeyAuth, ApiKeyTier, ArcGISClient, ElevationClient, ProfileParametersBuilder};
use anyhow::Result;
use tracing::{info, debug};

/// Helper to initialize tracing for tests
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"))
        )
        .with_test_writer()
        .try_init();
}

#[tokio::test]
#[cfg(all(feature = "geo", feature = "test-location"))]
async fn test_trail_difficulty_analysis() -> Result<()> {
    init_tracing();
    info!("Testing real-world trail difficulty analysis");

    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let elevation = ElevationClient::new(&client);

    // Half Dome trail (simplified transect)
    let trail = r#"{"paths":[[[-119.533,37.745],[-119.534,37.746]]]}"#;
    info!("Analyzing Half Dome trail segment");

    let params = ProfileParametersBuilder::default()
        .input_geometry(trail)
        .geometry_type("esriGeometryPolyline")
        .dem_resolution("30m")
        .build()?;

    debug!("Requesting elevation profile");
    let result = elevation.profile(params).await?;

    info!("Extracting elevation points");
    let points = result.elevation_points()?;
    debug!(point_count = points.len(), "Extracted elevation data");

    // Calculate trail metrics
    info!("Calculating trail difficulty metrics");

    let elevation_gain: f64 = points
        .windows(2)
        .map(|w| (w[1].1 - w[0].1).max(0.0))
        .sum();

    let total_distance: f64 = points
        .windows(2)
        .map(|w| (w[1].0 - w[0].0).abs())
        .sum();

    let avg_grade = if total_distance > 0.0 {
        (elevation_gain / total_distance) * 100.0
    } else {
        0.0
    };

    info!(
        elevation_gain_m = %format!("{:.1}", elevation_gain),
        distance_m = %format!("{:.1}", total_distance),
        avg_grade_pct = %format!("{:.1}", avg_grade),
        "Trail analysis complete"
    );

    info!("Trail Analysis Results:");
    info!("  Elevation gain: {:.1}m", elevation_gain);
    info!("  Distance: {:.1}m", total_distance);
    info!("  Average grade: {:.1}%", avg_grade);

    // Half Dome is steep!
    assert!(
        avg_grade > 10.0,
        "Expected steep grade for Half Dome, got {:.1}%",
        avg_grade
    );

    // Classify difficulty
    let difficulty = match avg_grade {
        g if g < 5.0 => "Easy",
        g if g < 10.0 => "Moderate",
        g if g < 15.0 => "Difficult",
        _ => "Very Difficult",
    };

    info!(difficulty = difficulty, "Trail difficulty classification");

    info!("Trail difficulty analysis test successful");
    Ok(())
}

#[tokio::test]
#[cfg(all(feature = "geo", feature = "test-location"))]
async fn test_watershed_elevation_analysis() -> Result<()> {
    init_tracing();
    info!("Testing watershed elevation analysis");

    use arcgis::SummarizeElevationParametersBuilder;

    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let elevation = ElevationClient::new(&client);

    // Small watershed polygon in Sierra Nevada
    let watershed = r#"{"rings":[[[-119.60,37.80],[-119.50,37.80],[-119.50,37.90],[-119.60,37.90],[-119.60,37.80]]],\"spatialReference\":{\"wkid\":4326}}"#;
    info!("Analyzing Sierra Nevada watershed");

    let params = SummarizeElevationParametersBuilder::default()
        .input_geometry(watershed)
        .geometry_type("esriGeometryPolygon")
        .dem_resolution("30m")
        .include_slope(true)
        .include_aspect(true)
        .build()?;

    debug!("Requesting elevation summary");
    let result = elevation.summarize_elevation(params).await?;

    debug!(
        min = ?result.min_elevation(),
        max = ?result.max_elevation(),
        mean = ?result.mean_elevation(),
        "Received elevation statistics"
    );

    // Extract polygon for spatial analysis
    info!("Converting summary to polygon");
    let polygon = result.to_polygon()?;

    use geo::Area;
    let area = polygon.unsigned_area();
    let area_km2 = area / 1_000_000.0;

    info!(
        area_km2 = %format!("{:.2}", area_km2),
        "Watershed area"
    );

    // Verify we got statistics
    if let (Some(min), Some(max), Some(mean)) = (
        result.min_elevation(),
        result.max_elevation(),
        result.mean_elevation(),
    ) {
        let relief = max - min;

        info!("Watershed Elevation Statistics:");
        info!("  Minimum: {:.1}m", min);
        info!("  Maximum: {:.1}m", max);
        info!("  Mean: {:.1}m", mean);
        info!("  Relief: {:.1}m", relief);
        info!("  Area: {:.2} km²", area_km2);

        assert!(relief > 0.0, "Watershed should have elevation variation");
        assert!(mean >= min && mean <= max, "Mean should be within min/max range");
    } else {
        anyhow::bail!("Expected elevation statistics in result");
    }

    info!("Watershed elevation analysis test successful");
    Ok(())
}
```

---

## Documentation Plan

### 1. Module-Level Docs

```rust
//! # Geospatial Data Types
//!
//! This module provides native support for ESRI JSON geometry formats and
//! seamless integration with the GeoRust ecosystem.
//!
//! ## Overview
//!
//! All ArcGIS services return geometries in ESRI JSON format. This module provides:
//!
//! - **Type-safe ESRI geometry types** ([`EsriPoint`], [`EsriPolyline`], etc.)
//! - **Conversion to/from geo-types** for spatial analysis
//! - **Feature set parsing** for service responses
//!
//! ## Quick Start
//!
//! ```rust
//! use arcgis::EsriPoint;
//! use geo::EuclideanDistance;
//!
//! // Parse ESRI JSON
//! let json = r#"{"x": -118.2437, "y": 34.0522, "spatialReference": {"wkid": 4326}}"#;
//! let esri_point: EsriPoint = serde_json::from_str(json)?;
//!
//! // Convert to geo-types using From trait
//! let geo_point: geo_types::Point = esri_point.into();
//!
//! // Use GeoRust algorithms
//! let other = geo_types::Point::new(-118.0, 34.0);
//! let distance = geo_point.euclidean_distance(&other);
//! ```
//!
//! ## Feature Flags
//!
//! - `geo` - Enable conversion to/from geo-types and spatial analysis algorithms
//! - `geozero` - Enable format conversions (WKT, WKB, GeoJSON, etc.)
//!
//! ## ESRI vs GeoJSON
//!
//! | Format | Point | Multi-value |
//! |--------|-------|-------------|
//! | ESRI | `{"x": -118, "y": 34}` | Z/M values supported |
//! | GeoJSON | `{"type": "Point", "coordinates": [-118, 34]}` | No Z/M |
//!
//! ## Examples
//!
//! See the [`examples/`](https://github.com/crumplecup/arcgis/tree/main/examples)
//! directory for complete examples:
//!
//! - [`elevation_analysis.rs`] - Extract and analyze elevation profiles
//! - [`viewshed_analysis.rs`] - Compute visible areas and analyze polygons
//! - [`spatial_query.rs`] - Query features and convert to geo-types
```

### 2. Cookbook Examples

Create `examples/geo/` directory with real-world recipes:

```
examples/geo/
├── elevation_profile_chart.rs    # Extract elevation, compute grade, create chart data
├── viewshed_coverage.rs           # Compute viewshed area, find coverage gaps
├── trail_difficulty_rating.rs     # Analyze elevation gain, grade, distance
├── feature_to_geojson.rs          # Query features, convert to GeoJSON
├── spatial_analysis.rs            # Buffer, intersect, contains operations
└── coordinate_transform.rs        # Transform between spatial references
```

### 3. Updated README Section

```markdown
## Geospatial Analysis

The SDK provides native support for ESRI geometries and seamless integration with
the GeoRust ecosystem for spatial analysis using standard Rust `From`/`TryFrom` traits.

```rust
use arcgis::{ElevationClient, ProfileParametersBuilder};
use geo::EuclideanLength;

// Get elevation profile
let result = elevation.profile(params).await?;

// Extract as geo-types for analysis (uses TryFrom internally)
let line_strings = result.to_line_strings()?;

// Use GeoRust algorithms
let total_distance = line_strings.euclidean_length();

// Extract elevation points (preserves Z/M values)
let elevation_points = result.elevation_points()?;
let elevation_gain = elevation_points
    .windows(2)
    .map(|w| (w[1].1 - w[0].1).max(0.0))
    .sum::<f64>();
```

**Supported conversions:**

- ESRI JSON ↔ geo-types (Point, LineString, Polygon, etc.)
- ESRI JSON → GeoJSON (with `geojson` crate)
- geo-types → WKT/WKB (with `geozero` feature)

**Spatial operations** (via `geo` crate):

- Distance calculations (Euclidean, Haversine, Geodesic)
- Area and length measurements
- Geometric operations (buffer, simplify, centroid)
- Spatial relationships (contains, intersects, within)
- Coordinate transformations (with `proj` crate)

```

---

## Migration Path

### Phase 1: Foundation (Week 1)
**Goal:** Core types, error handling, and basic conversions

- [ ] Create `src/geo/` module structure (mod.rs with re-exports only)
- [ ] Implement error handling following project standards:
  - [ ] Create `src/geo/errors.rs` with site-specific errors
  - [ ] `GeoError` wrapper (wraps `geo::Error`)
  - [ ] `GeometryJsonError` wrapper (wraps `serde_json::Error`)
  - [ ] `EsriGeometryErrorKind` enum with all error variants
  - [ ] `EsriGeometryError` (kind + location tracking)
  - [ ] All errors use private fields + Getters
  - [ ] All constructors use `#[track_caller]`
  - [ ] Add `Geometry` variant to crate-level `ErrorKind`
  - [ ] Integration tests for error chain preservation
- [ ] Implement ESRI geometry types with private fields + derives:
  - [ ] `EsriPoint` (Getters, Setters, Builder, with `new()` and `with_z()`)
  - [ ] `EsriPolyline` (Getters, Setters, Builder, with `has_z()`/`has_m()`)
  - [ ] `EsriPolygon` (Getters, Setters, Builder)
  - [ ] `EsriMultipoint` (Getters, Setters, Builder)
  - [ ] `EsriEnvelope` (Getters, Setters, Builder, new)
  - [ ] `EsriGeometry` enum
- [ ] Implement `SpatialReference` with:
  - [ ] Private fields + Getters + Builder
  - [ ] `wgs84()` and `web_mercator()` constructors
  - [ ] `is_geographic()` and `is_projected()` methods (instrumented)
- [ ] Implement standard Rust conversions:
  - [ ] `From<EsriPoint>` for `geo_types::Point`
  - [ ] `From<geo_types::Point>` for `EsriPoint`
  - [ ] `TryFrom<EsriPolyline>` for `geo_types::MultiLineString`
  - [ ] `TryFrom<EsriPolygon>` for `geo_types::Polygon`
  - [ ] `From<geo_types::LineString>` for `EsriPolyline`
  - [ ] `From<geo_types::Polygon>` for `EsriPolygon`
- [ ] Add `#[instrument]` to all public methods with proper tracing
- [ ] Update `src/lib.rs` to expose types at crate root:
  - [ ] Feature-gated exports: `#[cfg(feature = "geo")]`
  - [ ] Export all geometry types (EsriPoint, EsriPolyline, etc.)
  - [ ] Export error types (EsriGeometryError, etc.)
  - [ ] Flat organization (users import from `arcgis::` root)
- [ ] Unit tests for all conversions following test standards:
  - [ ] Return `anyhow::Result<()>`
  - [ ] Initialize tracing with helper
  - [ ] Use tracing for observability
  - [ ] No `.unwrap()` or `.expect()` - use `?`
  - [ ] Test both success and error cases
  - [ ] Test error chain preservation
  - [ ] Descriptive assertion messages
- [ ] Documentation for geometry module with doctests

**Deliverable:** Users can parse ESRI JSON and convert to/from geo-types using `.into()` and `.try_into()` with proper error handling

### Phase 2: Service Integration (Week 2)
**Goal:** Add geo methods to existing service results

- [ ] Add `#[cfg(feature = "geo")]` instrumented methods to `ProfileResult`:
  - [ ] `to_line_strings()` - uses `TryFrom` internally
  - [ ] `elevation_points()` - preserves Z/M values
- [ ] Add instrumented methods to `ViewshedResult`:
  - [ ] `to_polygon()` - uses `TryFrom` internally
  - [ ] `to_multi_polygon()` - handles multiple visible areas
- [ ] Add instrumented methods to `SummarizeElevationResult`:
  - [ ] `to_polygon()` - uses `TryFrom` internally
- [ ] Add instrumented methods to Feature Service `QueryResult`:
  - [ ] `to_geometries<T>()` - generic conversion
  - [ ] `to_points()` - specific for points
  - [ ] `to_geojson()` - full GeoJSON export
- [ ] Integration tests with real API calls following test standards:
  - [ ] Feature-gated with `test-location`
  - [ ] Return `anyhow::Result<()>` for async tests
  - [ ] Initialize tracing at test start
  - [ ] Use tracing for API call observability
  - [ ] No `.unwrap()` or `.expect()` - use `?`
  - [ ] Test real-world scenarios with meaningful assertions
- [ ] Update elevation_analysis example to demonstrate:
  - [ ] Using `.into()` for direct conversions
  - [ ] Using helper methods on results
  - [ ] GeoRust algorithm integration
  - [ ] Proper error handling with `?`
  - [ ] Tracing for observability

**Deliverable:** All service results provide convenience methods that use `From`/`TryFrom` internally

### Phase 3: Feature Sets (Week 3)
**Goal:** Full feature parsing

- [ ] Implement `EsriFeature` and `EsriFeatureSet`
- [ ] Implement `Field` metadata
- [ ] Polymorphic geometry deserialization
- [ ] Attribute access helpers
- [ ] Feature → GeoJSON conversion
- [ ] Tests with real feature service responses

**Deliverable:** Users can work with full feature sets

### Phase 4: Advanced Features (Week 4)
**Goal:** Z/M values, multipart geometries, format conversions

- [ ] Z-value preservation (elevation)
- [ ] M-value preservation (measures)
- [ ] Multipart geometry handling
- [ ] `geozero` integration (WKT, WKB)
- [ ] Spatial reference transformation helpers
- [ ] Performance benchmarks

**Deliverable:** Complete geospatial toolkit

### Phase 5: Polish (Week 5)
**Goal:** Documentation and examples

- [ ] Cookbook examples (5-10 real-world scenarios)
- [ ] API documentation review
- [ ] Performance optimization
- [ ] Error message improvements
- [ ] Blog post / announcement

**Deliverable:** Production-ready, well-documented

---

## Success Metrics

### Technical Metrics
- [ ] 100% coverage of ESRI geometry types
- [ ] Zero-copy conversions where possible
- [ ] < 5% performance overhead vs raw JSON parsing
- [ ] No breaking changes to existing API

### User Experience Metrics
- [ ] Examples demonstrate clear value (before/after code samples)
- [ ] Documentation shows real-world use cases
- [ ] Feature flag system allows opt-out (minimal users unaffected)

### Ecosystem Metrics
- [ ] Compatible with all major GeoRust crates
- [ ] Can round-trip ESRI JSON → geo-types → ESRI JSON
- [ ] Supports all spatial operations from `geo` crate

---

## Open Questions for Review

### 1. **Module Organization** ✅ DECIDED

**Decision:** Option C - Flat at crate root with feature flags

All types exposed at crate root, regardless of internal module structure:

```rust
// src/lib.rs
#[cfg(feature = "geo")]
pub use geo::esri::{
    EsriPoint, EsriPolyline, EsriPolygon, EsriMultipoint, EsriEnvelope,
    EsriGeometry, EsriFeature, EsriFeatureSet, SpatialReference,
};
```

**Internal structure** (implementation detail):
```
src/geo/esri/geometry.rs  // Implementation lives here
```

**User imports** (public API):
```rust
use arcgis::{EsriPoint, EsriPolyline};  // ✅ Flat at root
```

**Rationale:**
- Names don't conflict, no need for nested modules in imports
- Feature flags work at crate root level
- Consistent with project vision for type organization
- Simpler, cleaner API for users

### 2. **Z/M Value Handling** ✅ DECIDED

**Decision:** Option A for Phase 1, Option B for Phase 4

**Phase 1 (Current):** Single `EsriPoint` with optional Z/M fields
```rust
pub struct EsriPoint {
    x: f64,
    y: f64,
    z: Option<f64>,  // Optional elevation
    m: Option<f64>,  // Optional measure
    spatial_reference: Option<SpatialReference>,
}
```

**Phase 4 (Future):** Dedicated wrapper types for type safety
```rust
pub struct EsriPoint { x: f64, y: f64, ... }        // 2D
pub struct EsriPoint3D { x: f64, y: f64, z: f64, ... }   // 3D with elevation
pub struct EsriPointM { x: f64, y: f64, m: f64, ... }    // 2D with measure
pub struct EsriPointZM { x: f64, y: f64, z: f64, m: f64, ... }  // 3D with measure
```

**Rationale:**
- Phase 1: Simple, works immediately, matches ESRI JSON structure
- Phase 4: Type-safe, idiomatic, prevents Z/M confusion
- Start simple, add sophistication based on user needs

### 3. **Error Handling** ✅ DECIDED

**Decision:** New site-specific `EsriGeometryError` with proper error chain

**Pattern:** Follow project error handling standards with concrete types (no `dyn Error`).

#### Error Structure

```rust
// ============================================================================
// External error wrappers (concrete types, not dyn)
// ============================================================================

/// Wrapper for geo crate errors.
#[derive(Debug, Clone, derive_more::Display, derive_more::Error, derive_getters::Getters)]
#[display("Geo operation failed: {}", source)]
pub struct GeoError {
    source: geo::Error,  // Concrete type!
}

impl GeoError {
    #[track_caller]
    pub fn new(source: geo::Error) -> Self {
        Self { source }
    }
}

/// Wrapper for serde_json errors during geometry parsing.
#[derive(Debug, Clone, derive_more::Display, derive_more::Error, derive_getters::Getters)]
#[display("JSON parsing failed: {}", source)]
pub struct GeometryJsonError {
    source: serde_json::Error,
}

impl GeometryJsonError {
    #[track_caller]
    pub fn new(source: serde_json::Error) -> Self {
        Self { source }
    }
}

// ============================================================================
// Site-specific error kind for ESRI geometry operations
// ============================================================================

/// Specific error conditions for ESRI geometry operations.
#[derive(Debug, Clone, derive_more::Display)]
pub enum EsriGeometryErrorKind {
    /// Invalid geometry structure.
    #[display("Invalid geometry: {}", _0)]
    InvalidGeometry(String),

    /// Missing required coordinate data.
    #[display("Missing coordinate at {}", _0)]
    MissingCoordinate(String),

    /// Empty path/ring where data expected.
    #[display("Empty {}: {}", _0, _1)]
    EmptyGeometry(String, String),  // (geometry_type, details)

    /// Invalid coordinate values.
    #[display("Invalid coordinate: {}", _0)]
    InvalidCoordinate(String),

    /// Wrapped geo crate error.
    #[display("{}", _0)]
    Geo(GeoError),

    /// Wrapped JSON parsing error.
    #[display("{}", _0)]
    Json(GeometryJsonError),
}

// ============================================================================
// Main ESRI geometry error (kind + location)
// ============================================================================

/// Error from ESRI geometry operations with source location tracking.
#[derive(Debug, Clone, derive_more::Display, derive_more::Error, derive_getters::Getters)]
#[display("ESRI Geometry: {} at {}:{}", kind, file, line)]
pub struct EsriGeometryError {
    kind: EsriGeometryErrorKind,
    line: u32,
    file: &'static str,
}

impl EsriGeometryError {
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

// source() method defers to kind variants
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
// Integration with crate-level ErrorKind
// ============================================================================

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum ErrorKind {
    // ... existing variants (Http, Storage, etc.)

    /// ESRI geometry conversion/validation error.
    #[from]
    #[display("{}", _0)]
    Geometry(EsriGeometryError),
}
```

**Key principles:**
1. ✅ Site-specific error for geometry operations
2. ✅ No `Box<dyn Error>` - all concrete types
3. ✅ External errors wrapped in specific types (GeoError, GeometryJsonError)
4. ✅ Error chain preserved (source field in wrappers)
5. ✅ Tree-like structure: external → wrapper → ErrorKind → Error
6. ✅ All errors follow coding standards (private fields + Getters)
7. ✅ All constructors use `#[track_caller]`

**Rationale:**
- Errors are site-specific (geometry vs HTTP vs storage)
- Source chain preserved for debugging
- Type-safe (no dynamic dispatch except in `source()` method)
- Location tracking for debugging

### 4. **Feature Set API** ✅ DECIDED

**Decision:** Option C - Both iterator API and typed extraction helpers

Provide flexibility for different use cases:

```rust
impl EsriFeatureSet {
    // Iterator API for power users
    pub fn features(&self) -> &[EsriFeature] { ... }

    // Typed extraction helpers for common cases
    pub fn to_points(&self) -> Result<Vec<geo_types::Point>> { ... }
    pub fn to_polygons(&self) -> Result<Vec<geo_types::Polygon>> { ... }
    pub fn to_geometries<T>(&self) -> Result<Vec<T>> where T: TryFrom<EsriGeometry> { ... }
}
```

**Rationale:**
- Power users get full control with iterators
- Common cases have simple convenience methods
- Type safety with generic `to_geometries<T>()`

### 5. **Spatial Reference Handling** ✅ DECIDED

**Decision:** Option A for Phase 1, Option C for later phases

**Phase 1:** Store WKID/WKT, basic queries, no transformations
```rust
impl SpatialReference {
    pub const fn wgs84() -> Self { ... }
    pub const fn web_mercator() -> Self { ... }
    pub fn is_geographic(&self) -> bool { ... }
    pub fn is_projected(&self) -> bool { ... }
}
```

**Later Phase:** Optional `proj` integration behind feature flag
```toml
[features]
geo-proj = ["dep:proj", "geo"]  # Optional!
```

```rust
#[cfg(feature = "geo-proj")]
impl SpatialReference {
    pub fn transform_to(&self, target: &SpatialReference) -> Result<Transformer> { ... }
}
```

**Rationale:**
- Not all users have proj installed (system dependency)
- Keep proj optional, behind feature flag
- Phase 1 covers 90% of use cases (WGS84, Web Mercator)
- Advanced users can opt into transformation support

### 6. **Scope Boundaries** ✅ DECIDED

**Decision:** NO for initial implementation - focus on vector geometries

**Out of scope for now:**
- ❌ Raster support (ESRI Image Service results)
- ❌ Topology validation
- ❌ Spatial indexing (R-tree via `rstar`)

**In scope (Phases 1-5):**
- ✅ Vector geometries (Point, Polyline, Polygon, Multipoint, Envelope)
- ✅ Feature/FeatureSet support
- ✅ Spatial reference metadata
- ✅ Conversions to/from geo-types
- ✅ Service integration (elevation, features)

**Rationale:**
- Walk before we run
- Vector geometries are 90% of use cases
- Complex features (raster, topology, indexing) can come later
- Better to do core well than spread too thin

---

## Risk Assessment

### Low Risk
- ✅ Core types (Point, LineString, Polygon) - well-understood, tested patterns
- ✅ Feature-gated - doesn't impact users who don't need it
- ✅ GeoRust ecosystem stable and mature

### Medium Risk
- ⚠️ Z/M value handling - may need iteration based on user feedback
- ⚠️ Multipart geometry edge cases - need comprehensive test data
- ⚠️ Performance overhead - need benchmarks

### High Risk (Mitigation Strategies)
- 🔴 **API surface growth** → Keep behind feature flags, review each public method
- 🔴 **Maintenance burden** → Comprehensive tests, follow ESRI spec exactly
- 🔴 **Breaking changes to existing code** → Zero tolerance, use semver carefully

---

## Alternative Considered: External Crate

**Option:** Create separate `esri-geo` crate

**Pros:**
- Cleaner separation of concerns
- Independent versioning
- Could be used by other ESRI-related projects

**Cons:**
- Friction for users (two crates to install)
- Duplicated error handling
- Less integrated documentation
- Harder to maintain coherent API

**Decision:** Include in main crate, feature-gated. Rationale: Users expect geometry support in a geospatial SDK, and the integration points are too numerous to split cleanly.

---

## Conclusion

This plan transforms the SDK from an "API wrapper" to a "geospatial toolkit" by:

1. **Embracing the domain** - Geospatial data is our core business
2. **Leveraging existing work** - GeoRust ecosystem is mature and excellent
3. **Feature-gating complexity** - Users who don't need it pay zero cost
4. **Providing immediate value** - Every service result becomes analyzable

**Estimated Effort:** 4-5 weeks for complete implementation

**Recommendation:** Approve Phase 1 (foundation) and reevaluate after completion.

---

## Key Decisions Summary

### 1. Module Organization
✅ **Flat at crate root** - All types exposed at `arcgis::` root, internal modules are implementation detail

### 2. Type Encapsulation
✅ **Private fields + derives** - All types use Getters, Setters, Builder, new where appropriate

### 3. Conversions
✅ **Standard Rust traits** - `From` for infallible, `TryFrom` for fallible conversions (no custom traits)

### 4. Error Handling
✅ **Site-specific errors with concrete types** - No `Box<dyn Error>`, proper error chain preservation

### 5. Testing
✅ **anyhow::Result + tracing** - All tests return `Result`, initialize tracing, no `.unwrap()`

### 6. Instrumentation
✅ **All public methods traced** - `#[instrument]` on all public functions with proper field filtering

### 7. Z/M Values
✅ **Phase 1: Optional fields** - `Option<f64>`, Phase 4: Dedicated wrapper types

### 8. Feature Flags
✅ **Opt-in at crate root** - `#[cfg(feature = "geo")]` in lib.rs for each type

---

## Anti-Patterns to Avoid

### ❌ Buried Panics
```rust
// ❌ NEVER
impl From<geo_types::Point> for EsriPoint {
    fn from(p: geo_types::Point) -> Self {
        EsriPointBuilder::default()
            .x(p.x())
            .build()
            .expect("Failed")  // ❌ Panic in From!
    }
}

// ✅ CORRECT
impl From<geo_types::Point> for EsriPoint {
    fn from(p: geo_types::Point) -> Self {
        EsriPoint::new(p.x(), p.y())  // ✅ Infallible constructor
    }
}
```

### ❌ Dynamic Error Types
```rust
// ❌ NEVER
pub struct MyError {
    source: Box<dyn std::error::Error>,  // ❌ No dyn!
}

// ✅ CORRECT
pub struct GeoError {
    source: geo::Error,  // ✅ Concrete type
}
```

### ❌ Test Anti-Patterns
```rust
// ❌ NEVER
#[test]
fn test_conversion() {
    let result = convert().unwrap();  // ❌ No unwrap!
    assert_eq!(result, expected);
}

// ✅ CORRECT
#[test]
fn test_conversion() -> anyhow::Result<()> {
    init_tracing();  // ✅ Initialize tracing
    info!("Testing conversion");
    let result = convert()?;  // ✅ Use ?
    assert_eq!(result, expected);
    Ok(())
}
```

### ❌ Nested Module Imports
```rust
// ❌ NEVER
use arcgis::geo::esri::EsriPoint;

// ✅ CORRECT
use arcgis::EsriPoint;  // Flat at root
```

---

## Approval Checklist

- [ ] Architecture approved
- [ ] Module organization approved (flat at crate root)
- [ ] Error handling pattern approved (site-specific, concrete types)
- [ ] Feature flag strategy approved
- [ ] Testing strategy approved (anyhow::Result + tracing)
- [ ] Documentation plan approved
- [ ] Success metrics approved
- [ ] Risk mitigation accepted
- [ ] Timeline realistic

---

**Next Steps After Approval:**
1. Create feature branch `feature/esri-geo-integration`
2. Implement Phase 1 (foundation types + error handling)
3. Create draft PR with working examples
4. Iterate based on feedback
5. Merge when Phase 1 complete and tested
