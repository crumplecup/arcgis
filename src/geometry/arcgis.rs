//! ArcGIS geometry JSON structures.
//!
//! These types represent the ArcGIS REST API geometry format.
//! They serialize/deserialize to/from the JSON format expected by ArcGIS services.

use serde::{Deserialize, Serialize};

/// Spatial reference for ArcGIS geometries.
///
/// Most commonly uses WKID (Well-Known ID) for standard coordinate systems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpatialReference {
    /// Well-Known ID (e.g., 4326 for WGS84, 3857 for Web Mercator)
    Wkid {
        /// The WKID code.
        wkid: i32,
    },
    /// Well-Known Text representation.
    Wkt {
        /// The WKT string.
        wkt: String,
    },
}

impl SpatialReference {
    /// Creates a spatial reference from a WKID.
    pub fn wkid(wkid: i32) -> Self {
        Self::Wkid { wkid }
    }

    /// WGS84 (EPSG:4326) - standard geographic coordinate system.
    pub fn wgs84() -> Self {
        Self::wkid(4326)
    }

    /// Web Mercator (EPSG:3857) - common web mapping projection.
    pub fn web_mercator() -> Self {
        Self::wkid(3857)
    }
}

/// ArcGIS Point geometry.
///
/// # JSON Format
/// ```json
/// {
///   "x": -118.15,
///   "y": 33.80,
///   "spatialReference": {"wkid": 4326}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcGISPoint {
    /// X coordinate (longitude in geographic systems).
    pub x: f64,
    /// Y coordinate (latitude in geographic systems).
    pub y: f64,
    /// Optional Z coordinate (elevation/altitude).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>,
    /// Optional M value (measure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub m: Option<f64>,
    /// Spatial reference system.
    #[serde(rename = "spatialReference", skip_serializing_if = "Option::is_none")]
    pub spatial_reference: Option<SpatialReference>,
}

/// ArcGIS Multipoint geometry.
///
/// # JSON Format
/// ```json
/// {
///   "points": [[-97.06, 32.84], [-97.06, 32.85]],
///   "spatialReference": {"wkid": 4326}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcGISMultipoint {
    /// Array of [x, y] coordinate pairs.
    pub points: Vec<[f64; 2]>,
    /// Spatial reference system.
    #[serde(rename = "spatialReference", skip_serializing_if = "Option::is_none")]
    pub spatial_reference: Option<SpatialReference>,
}

/// ArcGIS Polyline geometry.
///
/// # JSON Format
/// ```json
/// {
///   "paths": [
///     [[-97.06, 32.84], [-97.06, 32.85]],
///     [[-97.06, 32.86], [-97.06, 32.87]]
///   ],
///   "spatialReference": {"wkid": 4326}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcGISPolyline {
    /// Array of paths, where each path is an array of [x, y] coordinate pairs.
    pub paths: Vec<Vec<[f64; 2]>>,
    /// Spatial reference system.
    #[serde(rename = "spatialReference", skip_serializing_if = "Option::is_none")]
    pub spatial_reference: Option<SpatialReference>,
}

/// ArcGIS Polygon geometry.
///
/// # JSON Format
/// ```json
/// {
///   "rings": [
///     [[-97.06, 32.84], [-97.06, 32.85], [-97.07, 32.85], [-97.06, 32.84]]
///   ],
///   "spatialReference": {"wkid": 4326}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcGISPolygon {
    /// Array of rings, where each ring is an array of [x, y] coordinate pairs.
    /// First ring is exterior, subsequent rings are holes.
    pub rings: Vec<Vec<[f64; 2]>>,
    /// Spatial reference system.
    #[serde(rename = "spatialReference", skip_serializing_if = "Option::is_none")]
    pub spatial_reference: Option<SpatialReference>,
}

/// ArcGIS Envelope (bounding box) geometry.
///
/// # JSON Format
/// ```json
/// {
///   "xmin": -109.55,
///   "ymin": 25.76,
///   "xmax": -86.39,
///   "ymax": 49.94,
///   "spatialReference": {"wkid": 4326}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcGISEnvelope {
    /// Minimum X coordinate.
    pub xmin: f64,
    /// Minimum Y coordinate.
    pub ymin: f64,
    /// Maximum X coordinate.
    pub xmax: f64,
    /// Maximum Y coordinate.
    pub ymax: f64,
    /// Spatial reference system.
    #[serde(rename = "spatialReference", skip_serializing_if = "Option::is_none")]
    pub spatial_reference: Option<SpatialReference>,
}

/// Union type for all ArcGIS geometry types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArcGISGeometry {
    /// Point geometry.
    Point(ArcGISPoint),
    /// Multipoint geometry.
    Multipoint(ArcGISMultipoint),
    /// Polyline geometry.
    Polyline(ArcGISPolyline),
    /// Polygon geometry.
    Polygon(ArcGISPolygon),
    /// Envelope (bounding box) geometry.
    Envelope(ArcGISEnvelope),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_serialization() {
        let point = ArcGISPoint {
            x: -118.15,
            y: 33.80,
            z: None,
            m: None,
            spatial_reference: Some(SpatialReference::wgs84()),
        };

        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("\"x\":-118.15"));
        assert!(json.contains("\"y\":33.8"));
        assert!(json.contains("\"wkid\":4326"));
    }

    #[test]
    fn test_point_deserialization() {
        let json = r#"{"x":-118.15,"y":33.80,"spatialReference":{"wkid":4326}}"#;
        let point: ArcGISPoint = serde_json::from_str(json).unwrap();

        assert_eq!(point.x, -118.15);
        assert_eq!(point.y, 33.80);
        assert_eq!(point.spatial_reference, Some(SpatialReference::wgs84()));
    }

    #[test]
    fn test_polygon_serialization() {
        let polygon = ArcGISPolygon {
            rings: vec![vec![
                [-97.06, 32.84],
                [-97.06, 32.85],
                [-97.07, 32.85],
                [-97.06, 32.84],
            ]],
            spatial_reference: Some(SpatialReference::wgs84()),
        };

        let json = serde_json::to_string(&polygon).unwrap();
        assert!(json.contains("\"rings\""));
        assert!(json.contains("-97.06"));
    }

    #[test]
    fn test_spatial_reference() {
        let wgs84 = SpatialReference::wgs84();
        let json = serde_json::to_string(&wgs84).unwrap();
        assert_eq!(json, r#"{"wkid":4326}"#);

        let web_merc = SpatialReference::web_mercator();
        let json = serde_json::to_string(&web_merc).unwrap();
        assert_eq!(json, r#"{"wkid":3857}"#);
    }
}
