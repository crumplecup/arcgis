//! Geometry decoder for PBF format.
//!
//! Handles delta-encoded, quantized coordinates from ArcGIS Protocol Buffer responses.

use super::feature_collection_p_buffer::*;
use crate::{
    ArcGISGeometryV2 as ArcGISGeometry, ArcGISMultipointV2 as ArcGISMultipoint,
    ArcGISPointV2 as ArcGISPoint, ArcGISPolygonV2 as ArcGISPolygon,
    ArcGISPolylineV2 as ArcGISPolyline, GeometryType, Result,
};

/// Decode a PBF Geometry into an ArcGISGeometry.
///
/// This handles the delta-encoding and quantization used in PBF format:
/// 1. Coordinates are stored as delta-encoded integers
/// 2. Integer coords are converted to real coords via: real = (int * scale) + translate
/// 3. The `lengths` array describes the structure (points per part)
pub fn decode_geometry(
    pbf_geometry: &Geometry,
    geometry_type: GeometryType,
    transform: Option<&Transform>,
    _has_z: bool,
    _has_m: bool,
) -> Result<ArcGISGeometry> {
    // Extract transform parameters
    let (x_scale, y_scale, x_translate, y_translate) = if let Some(t) = transform {
        let scale = t.scale.as_ref().ok_or_else(|| {
            crate::Error::from(crate::ErrorKind::Other(
                "Missing scale in transform".to_string(),
            ))
        })?;
        let translate = t.translate.as_ref().ok_or_else(|| {
            crate::Error::from(crate::ErrorKind::Other(
                "Missing translate in transform".to_string(),
            ))
        })?;
        (
            scale.x_scale,
            scale.y_scale,
            translate.x_translate,
            translate.y_translate,
        )
    } else {
        // No transform means coords are already in real space (scale=1, translate=0)
        (1.0, 1.0, 0.0, 0.0)
    };

    match geometry_type {
        GeometryType::Point => decode_point(
            &pbf_geometry.coords,
            x_scale,
            y_scale,
            x_translate,
            y_translate,
        ),
        GeometryType::Multipoint => decode_multipoint(
            &pbf_geometry.coords,
            x_scale,
            y_scale,
            x_translate,
            y_translate,
        ),
        GeometryType::Polyline => decode_polyline(
            &pbf_geometry.coords,
            &pbf_geometry.lengths,
            x_scale,
            y_scale,
            x_translate,
            y_translate,
        ),
        GeometryType::Polygon => decode_polygon(
            &pbf_geometry.coords,
            &pbf_geometry.lengths,
            x_scale,
            y_scale,
            x_translate,
            y_translate,
        ),
        GeometryType::Envelope => Err(crate::Error::from(crate::ErrorKind::Other(
            "Envelope geometry not yet supported in PBF".to_string(),
        ))),
    }
}

/// Decode a Point geometry.
///
/// Point has 2 coordinates: [x, y]
fn decode_point(
    coords: &[i64],
    x_scale: f64,
    y_scale: f64,
    x_translate: f64,
    y_translate: f64,
) -> Result<ArcGISGeometry> {
    if coords.len() < 2 {
        return Err(crate::Error::from(crate::ErrorKind::Other(
            "Point geometry requires at least 2 coordinates".to_string(),
        )));
    }

    // Points are NOT delta-encoded in PBF (they're single values)
    let x = (coords[0] as f64 * x_scale) + x_translate;
    let y = (coords[1] as f64 * y_scale) + y_translate;

    Ok(ArcGISGeometry::Point(ArcGISPoint::new(x, y)))
}

/// Decode a Multipoint geometry.
///
/// Multipoint has coordinates: [x1, y1, x2, y2, ...]
/// Coordinates are delta-encoded: each value is a delta from the previous
fn decode_multipoint(
    coords: &[i64],
    x_scale: f64,
    y_scale: f64,
    x_translate: f64,
    y_translate: f64,
) -> Result<ArcGISGeometry> {
    if coords.len() % 2 != 0 {
        return Err(crate::Error::from(crate::ErrorKind::Other(
            "Multipoint coordinates must be in pairs".to_string(),
        )));
    }

    let mut points = Vec::new();
    let mut x_accum: i64 = 0;
    let mut y_accum: i64 = 0;

    for chunk in coords.chunks(2) {
        // Accumulate deltas
        x_accum += chunk[0];
        y_accum += chunk[1];

        // Convert to real coordinates
        let x = (x_accum as f64 * x_scale) + x_translate;
        let y = (y_accum as f64 * y_scale) + y_translate;

        points.push(vec![x, y]);
    }

    Ok(ArcGISGeometry::Multipoint(ArcGISMultipoint::new(points)))
}

/// Decode a Polyline geometry.
///
/// Polyline coordinates are delta-encoded and grouped by the `lengths` array.
/// Each entry in `lengths` specifies how many points are in that path.
fn decode_polyline(
    coords: &[i64],
    lengths: &[u32],
    x_scale: f64,
    y_scale: f64,
    x_translate: f64,
    y_translate: f64,
) -> Result<ArcGISGeometry> {
    let paths = decode_paths(coords, lengths, x_scale, y_scale, x_translate, y_translate)?;
    Ok(ArcGISGeometry::Polyline(ArcGISPolyline::new(paths)))
}

/// Decode a Polygon geometry.
///
/// Polygon coordinates are delta-encoded and grouped by the `lengths` array.
/// Each entry in `lengths` specifies how many points are in that ring.
fn decode_polygon(
    coords: &[i64],
    lengths: &[u32],
    x_scale: f64,
    y_scale: f64,
    x_translate: f64,
    y_translate: f64,
) -> Result<ArcGISGeometry> {
    let rings = decode_paths(coords, lengths, x_scale, y_scale, x_translate, y_translate)?;
    Ok(ArcGISGeometry::Polygon(ArcGISPolygon::new(rings)))
}

/// Helper to decode paths/rings for polyline/polygon geometries.
///
/// The `lengths` array specifies the number of points in each path/ring.
/// Coordinates are delta-encoded across the entire geometry.
fn decode_paths(
    coords: &[i64],
    lengths: &[u32],
    x_scale: f64,
    y_scale: f64,
    x_translate: f64,
    y_translate: f64,
) -> Result<Vec<Vec<Vec<f64>>>> {
    let mut paths = Vec::new();
    let mut coord_idx = 0;
    let mut x_accum: i64 = 0;
    let mut y_accum: i64 = 0;

    for &length in lengths {
        let mut path = Vec::new();
        let point_count = length as usize;

        for _ in 0..point_count {
            if coord_idx + 1 >= coords.len() {
                return Err(crate::Error::from(crate::ErrorKind::Other(
                    "Insufficient coordinates for geometry".to_string(),
                )));
            }

            // Accumulate deltas
            x_accum += coords[coord_idx];
            y_accum += coords[coord_idx + 1];
            coord_idx += 2;

            // Convert to real coordinates
            let x = (x_accum as f64 * x_scale) + x_translate;
            let y = (y_accum as f64 * y_scale) + y_translate;

            path.push(vec![x, y]);
        }

        paths.push(path);
    }

    Ok(paths)
}
