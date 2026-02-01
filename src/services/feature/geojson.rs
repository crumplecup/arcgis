//! GeoJSON format support for ArcGIS Feature Services.
//!
//! This module provides conversion from GeoJSON responses to ArcGIS FeatureSet format.

use crate::{
    ArcGISGeometry, ArcGISMultipoint,
    ArcGISPoint, ArcGISPolygon,
    ArcGISPolyline, Error, ErrorKind, Feature, FeatureSet, Result,
};
use std::collections::HashMap;
use tracing::instrument;

/// Convert a GeoJSON FeatureCollection to a FeatureSet.
///
/// This function handles the conversion from GeoJSON format to our standard
/// FeatureSet type, including geometry conversion and property mapping.
///
/// # Errors
///
/// Returns an error if:
/// - Geometry conversion fails
/// - Invalid coordinate arrays (missing x or y values)
/// - Unsupported geometry types (MultiPolygon, GeometryCollection)
#[instrument(skip(fc), fields(feature_count = fc.features.len()))]
pub fn from_geojson(fc: geojson::FeatureCollection) -> Result<FeatureSet> {
    tracing::debug!("Converting GeoJSON FeatureCollection to FeatureSet");

    let mut features = Vec::new();

    for (index, geojson_feature) in fc.features.into_iter().enumerate() {
        // Convert properties to attributes HashMap
        let attributes: HashMap<String, serde_json::Value> = geojson_feature
            .properties
            .map(|props| props.into_iter().collect())
            .unwrap_or_default();

        tracing::debug!(
            feature_index = index,
            attribute_count = attributes.len(),
            "Converting feature"
        );

        // Convert GeoJSON geometry to ArcGIS geometry
        let geometry = match geojson_feature.geometry {
            Some(geom) => match geometry_from_geojson(&geom) {
                Ok(g) => Some(g),
                Err(e) => {
                    tracing::error!(
                        feature_index = index,
                        error = %e,
                        "Failed to convert geometry"
                    );
                    return Err(e);
                }
            },
            None => None,
        };

        features.push(Feature::new(attributes, geometry));
    }

    tracing::debug!(
        converted_features = features.len(),
        "GeoJSON conversion completed"
    );

    // GeoJSON doesn't include these ArcGIS-specific fields
    Ok(FeatureSet::new(
        None,     // geometry_type
        features, // features
        None,     // count
        false,    // exceeded_transfer_limit
    ))
}

/// Convert GeoJSON geometry to ArcGIS geometry.
///
/// # Errors
///
/// Returns an error if coordinates are invalid or geometry type is unsupported.
fn geometry_from_geojson(geom: &geojson::Geometry) -> Result<ArcGISGeometry> {
    use geojson::Value;

    match &geom.value {
        Value::Point(coords) => {
            validate_coords(coords, 2)?;
            let point = if let Some(z) = coords.get(2).copied() {
                ArcGISPoint::with_z(coords[0], coords[1], z)
            } else {
                ArcGISPoint::new(coords[0], coords[1])
            };
            Ok(ArcGISGeometry::Point(point))
        }
        Value::MultiPoint(coords) => {
            let points: Vec<Vec<f64>> = coords
                .iter()
                .map(|c| {
                    validate_coords(c, 2)?;
                    Ok(vec![c[0], c[1]])
                })
                .collect::<Result<Vec<_>>>()?;

            let multipoint = ArcGISMultipoint::new(points);
            Ok(ArcGISGeometry::Multipoint(multipoint))
        }
        Value::LineString(coords) => {
            let path: Vec<Vec<f64>> = coords
                .iter()
                .map(|c| {
                    validate_coords(c, 2)?;
                    Ok(vec![c[0], c[1]])
                })
                .collect::<Result<Vec<_>>>()?;

            let polyline = ArcGISPolyline::new(vec![path]);
            Ok(ArcGISGeometry::Polyline(polyline))
        }
        Value::MultiLineString(lines) => {
            let paths: Vec<Vec<Vec<f64>>> = lines
                .iter()
                .map(|line| {
                    line.iter()
                        .map(|c| {
                            validate_coords(c, 2)?;
                            Ok(vec![c[0], c[1]])
                        })
                        .collect::<Result<Vec<_>>>()
                })
                .collect::<Result<Vec<_>>>()?;

            let polyline = ArcGISPolyline::new(paths);
            Ok(ArcGISGeometry::Polyline(polyline))
        }
        Value::Polygon(rings) => {
            let polygon_rings: Vec<Vec<Vec<f64>>> = rings
                .iter()
                .map(|ring| {
                    ring.iter()
                        .map(|c| {
                            validate_coords(c, 2)?;
                            Ok(vec![c[0], c[1]])
                        })
                        .collect::<Result<Vec<_>>>()
                })
                .collect::<Result<Vec<_>>>()?;

            let polygon = ArcGISPolygon::new(polygon_rings);
            Ok(ArcGISGeometry::Polygon(polygon))
        }
        Value::MultiPolygon(_) => {
            tracing::warn!("MultiPolygon geometry type not yet supported");
            Err(Error::from(ErrorKind::Other(
                "MultiPolygon not yet supported in GeoJSON conversion".to_string(),
            )))
        }
        Value::GeometryCollection(_) => {
            tracing::warn!("GeometryCollection geometry type not supported");
            Err(Error::from(ErrorKind::Other(
                "GeometryCollection not supported in ArcGIS geometry".to_string(),
            )))
        }
    }
}

/// Validate that a coordinate array has the required number of elements.
///
/// # Errors
///
/// Returns an error if the coordinate array has fewer than `min_len` elements.
fn validate_coords(coords: &[f64], min_len: usize) -> Result<()> {
    if coords.len() < min_len {
        tracing::error!(
            actual_len = coords.len(),
            required_len = min_len,
            "Invalid coordinate array length"
        );
        Err(Error::from(ErrorKind::Other(format!(
            "Invalid GeoJSON coordinates: expected at least {} values, got {}",
            min_len,
            coords.len()
        ))))
    } else {
        Ok(())
    }
}
