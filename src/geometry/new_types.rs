//! ESRI geometry type implementations.

use super::new_spatial_ref::SpatialReference;
use derive_getters::Getters;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tracing::instrument;

// ============================================================================
// ArcGISPoint
// ============================================================================

/// ESRI Point geometry with X/Y coordinates and optional Z/M values.
///
/// # Examples
///
/// ```
/// # use arcgis::ArcGISPoint;
/// // Create a simple 2D point
/// let point = ArcGISPoint::new(-118.2437, 34.0522);
/// assert_eq!(*point.x(), -118.2437);
/// assert_eq!(*point.y(), 34.0522);
///
/// // Create a 3D point with elevation
/// let point_3d = ArcGISPoint::with_z(-118.2437, 34.0522, 100.0);
/// assert_eq!(*point_3d.z(), Some(100.0));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct ArcGISPoint {
    /// X coordinate (longitude in geographic systems).
    x: f64,

    /// Y coordinate (latitude in geographic systems).
    y: f64,

    /// Z-value (elevation/height).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[setters(skip)] // Skip setter due to custom with_z constructor
    z: Option<f64>,

    /// M-value (measure/distance along line).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    m: Option<f64>,

    /// Spatial reference system.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    spatial_reference: Option<SpatialReference>,
}

impl ArcGISPoint {
    /// Creates a simple point with X/Y coordinates in WGS84.
    ///
    /// # Examples
    ///
    /// ```
    /// # use arcgis::ArcGISPoint;
    /// let point = ArcGISPoint::new(-118.2437, 34.0522);
    /// ```
    #[instrument(skip_all, fields(x = %x, y = %y))]
    pub fn new(x: f64, y: f64) -> Self {
        tracing::debug!("Creating ArcGISPoint");
        Self {
            x,
            y,
            z: None,
            m: None,
            spatial_reference: Some(SpatialReference::wgs84()),
        }
    }

    /// Creates a point with elevation (Z value).
    ///
    /// # Examples
    ///
    /// ```
    /// # use arcgis::ArcGISPoint;
    /// let point = ArcGISPoint::with_z(-118.2437, 34.0522, 100.0);
    /// assert_eq!(*point.z(), Some(100.0));
    /// ```
    #[instrument(skip_all, fields(x = %x, y = %y, z = %z))]
    pub fn with_z(x: f64, y: f64, z: f64) -> Self {
        tracing::debug!("Creating ArcGISPoint with elevation");
        Self {
            x,
            y,
            z: Some(z),
            m: None,
            spatial_reference: Some(SpatialReference::wgs84()),
        }
    }
}

// ============================================================================
// ArcGISPolyline
// ============================================================================

/// ESRI Polyline geometry with paths array.
///
/// # Examples
///
/// ```
/// # use arcgis::ArcGISPolyline;
/// let paths = vec![
///     vec![vec![-118.0, 34.0], vec![-117.0, 33.0]],
/// ];
/// let polyline = ArcGISPolyline::new(paths);
/// assert_eq!(polyline.paths().len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct ArcGISPolyline {
    /// Array of paths, where each path is an array of points.
    paths: Vec<Vec<Vec<f64>>>,

    /// Indicates if the polyline has Z values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    has_z: Option<bool>,

    /// Indicates if the polyline has M values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    has_m: Option<bool>,

    /// Spatial reference system.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    spatial_reference: Option<SpatialReference>,
}

impl ArcGISPolyline {
    /// Creates a polyline from paths in WGS84.
    ///
    /// # Examples
    ///
    /// ```
    /// # use arcgis::ArcGISPolyline;
    /// let paths = vec![
    ///     vec![vec![-118.0, 34.0], vec![-117.0, 33.0]],
    /// ];
    /// let polyline = ArcGISPolyline::new(paths);
    /// ```
    #[instrument(skip_all, fields(path_count = paths.len()))]
    pub fn new(paths: Vec<Vec<Vec<f64>>>) -> Self {
        tracing::debug!("Creating ArcGISPolyline");
        Self {
            paths,
            has_z: None,
            has_m: None,
            spatial_reference: Some(SpatialReference::wgs84()),
        }
    }
}

// ============================================================================
// ArcGISPolygon
// ============================================================================

/// ESRI Polygon geometry with rings array.
///
/// # Examples
///
/// ```
/// # use arcgis::ArcGISPolygon;
/// let rings = vec![
///     vec![
///         vec![-118.0, 34.0],
///         vec![-117.0, 34.0],
///         vec![-117.0, 33.0],
///         vec![-118.0, 34.0],  // Closed ring
///     ],
/// ];
/// let polygon = ArcGISPolygon::new(rings);
/// assert_eq!(polygon.rings().len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct ArcGISPolygon {
    /// Array of rings, where each ring is an array of points.
    /// First ring is exterior, subsequent rings are holes.
    rings: Vec<Vec<Vec<f64>>>,

    /// Indicates if the polygon has Z values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    has_z: Option<bool>,

    /// Indicates if the polygon has M values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    has_m: Option<bool>,

    /// Spatial reference system.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    spatial_reference: Option<SpatialReference>,
}

impl ArcGISPolygon {
    /// Creates a polygon from rings in WGS84.
    ///
    /// # Examples
    ///
    /// ```
    /// # use arcgis::ArcGISPolygon;
    /// let rings = vec![
    ///     vec![
    ///         vec![-118.0, 34.0],
    ///         vec![-117.0, 34.0],
    ///         vec![-117.0, 33.0],
    ///         vec![-118.0, 34.0],
    ///     ],
    /// ];
    /// let polygon = ArcGISPolygon::new(rings);
    /// ```
    #[instrument(skip_all, fields(ring_count = rings.len()))]
    pub fn new(rings: Vec<Vec<Vec<f64>>>) -> Self {
        tracing::debug!("Creating ArcGISPolygon");
        Self {
            rings,
            has_z: None,
            has_m: None,
            spatial_reference: Some(SpatialReference::wgs84()),
        }
    }
}

// ============================================================================
// ArcGISMultipoint
// ============================================================================

/// ESRI Multipoint geometry.
///
/// # Examples
///
/// ```
/// # use arcgis::ArcGISMultipoint;
/// let points = vec![
///     vec![-118.0, 34.0],
///     vec![-117.0, 33.0],
/// ];
/// let multipoint = ArcGISMultipoint::new(points);
/// assert_eq!(multipoint.points().len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct ArcGISMultipoint {
    /// Array of points.
    points: Vec<Vec<f64>>,

    /// Indicates if the multipoint has Z values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    has_z: Option<bool>,

    /// Indicates if the multipoint has M values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    has_m: Option<bool>,

    /// Spatial reference system.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    spatial_reference: Option<SpatialReference>,
}

impl ArcGISMultipoint {
    /// Creates a multipoint from points in WGS84.
    ///
    /// # Examples
    ///
    /// ```
    /// # use arcgis::ArcGISMultipoint;
    /// let points = vec![
    ///     vec![-118.0, 34.0],
    ///     vec![-117.0, 33.0],
    /// ];
    /// let multipoint = ArcGISMultipoint::new(points);
    /// ```
    #[instrument(skip_all, fields(point_count = points.len()))]
    pub fn new(points: Vec<Vec<f64>>) -> Self {
        tracing::debug!("Creating ArcGISMultipoint");
        Self {
            points,
            has_z: None,
            has_m: None,
            spatial_reference: Some(SpatialReference::wgs84()),
        }
    }
}

// ============================================================================
// geo-types conversions for ArcGISPoint
// ============================================================================

impl From<ArcGISPoint> for geo_types::Point {
    #[instrument(skip(point), fields(x = %point.x, y = %point.y))]
    fn from(point: ArcGISPoint) -> Self {
        tracing::debug!("Converting ArcGISPoint to geo_types::Point");
        geo_types::Point::new(point.x, point.y)
    }
}

impl From<ArcGISPoint> for geo_types::Coord {
    fn from(point: ArcGISPoint) -> Self {
        geo_types::Coord {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<geo_types::Point> for ArcGISPoint {
    #[instrument(skip(point), fields(x = %point.x(), y = %point.y()))]
    fn from(point: geo_types::Point) -> Self {
        tracing::debug!("Converting geo_types::Point to ArcGISPoint");
        ArcGISPoint::new(point.x(), point.y())
    }
}

// ============================================================================
// geo-types conversions for ArcGISPolyline
// ============================================================================

impl TryFrom<ArcGISPolyline> for geo_types::MultiLineString {
    type Error = crate::geometry::new_errors::ArcGISGeometryError;

    #[instrument(skip(polyline), fields(path_count = polyline.paths.len()))]
    fn try_from(polyline: ArcGISPolyline) -> Result<Self, Self::Error> {
        use crate::geometry::new_errors::{ArcGISGeometryError, ArcGISGeometryErrorKind};

        tracing::debug!("Converting ArcGISPolyline to geo_types::MultiLineString");

        if polyline.paths.is_empty() {
            return Err(ArcGISGeometryError::new(ArcGISGeometryErrorKind::EmptyGeometry(
                "polyline".to_string(),
                "conversion to MultiLineString".to_string(),
            )));
        }

        let line_strings: Result<Vec<geo_types::LineString>, ArcGISGeometryError> = polyline
            .paths
            .into_iter()
            .enumerate()
            .map(|(path_idx, path)| {
                if path.is_empty() {
                    return Err(ArcGISGeometryError::new(
                        ArcGISGeometryErrorKind::EmptyGeometry(
                            "path".to_string(),
                            format!("path index {}", path_idx),
                        ),
                    ));
                }

                let coords: Result<Vec<geo_types::Coord>, ArcGISGeometryError> = path
                    .into_iter()
                    .enumerate()
                    .map(|(_pt_idx, pt)| {
                        if pt.len() < 2 {
                            return Err(ArcGISGeometryError::new(
                                ArcGISGeometryErrorKind::InvalidCoordinateLength {
                                    expected: 2,
                                    actual: pt.len(),
                                },
                            ));
                        }
                        Ok(geo_types::Coord { x: pt[0], y: pt[1] })
                    })
                    .collect();

                coords.map(geo_types::LineString::new)
            })
            .collect();

        line_strings.map(geo_types::MultiLineString::new)
    }
}

impl From<geo_types::LineString> for ArcGISPolyline {
    #[instrument(skip(line))]
    fn from(line: geo_types::LineString) -> Self {
        tracing::debug!("Converting geo_types::LineString to ArcGISPolyline");
        let paths = vec![line
            .into_inner()
            .into_iter()
            .map(|coord| vec![coord.x, coord.y])
            .collect()];
        ArcGISPolyline::new(paths)
    }
}

impl From<geo_types::MultiLineString> for ArcGISPolyline {
    #[instrument(skip(mls))]
    fn from(mls: geo_types::MultiLineString) -> Self {
        tracing::debug!("Converting geo_types::MultiLineString to ArcGISPolyline");
        let paths = mls
            .into_iter()
            .map(|line| {
                line.into_inner()
                    .into_iter()
                    .map(|coord| vec![coord.x, coord.y])
                    .collect()
            })
            .collect();
        ArcGISPolyline::new(paths)
    }
}

// ============================================================================
// geo-types conversions for ArcGISPolygon
// ============================================================================

impl TryFrom<ArcGISPolygon> for geo_types::Polygon {
    type Error = crate::geometry::new_errors::ArcGISGeometryError;

    #[instrument(skip(polygon), fields(ring_count = polygon.rings.len()))]
    fn try_from(polygon: ArcGISPolygon) -> Result<Self, Self::Error> {
        use crate::geometry::new_errors::{ArcGISGeometryError, ArcGISGeometryErrorKind};

        tracing::debug!("Converting ArcGISPolygon to geo_types::Polygon");

        if polygon.rings.is_empty() {
            return Err(ArcGISGeometryError::new(ArcGISGeometryErrorKind::EmptyGeometry(
                "polygon".to_string(),
                "conversion to Polygon".to_string(),
            )));
        }

        let mut rings_iter = polygon.rings.into_iter();
        let exterior_ring = rings_iter.next().unwrap(); // Safe: checked is_empty above

        let exterior_coords: Result<Vec<geo_types::Coord>, ArcGISGeometryError> = exterior_ring
            .into_iter()
            .map(|pt| {
                if pt.len() < 2 {
                    return Err(ArcGISGeometryError::new(
                        ArcGISGeometryErrorKind::InvalidCoordinateLength {
                            expected: 2,
                            actual: pt.len(),
                        },
                    ));
                }
                Ok(geo_types::Coord { x: pt[0], y: pt[1] })
            })
            .collect();

        let exterior = geo_types::LineString::new(exterior_coords?);

        let interiors: Result<Vec<geo_types::LineString>, ArcGISGeometryError> = rings_iter
            .map(|ring| {
                let coords: Result<Vec<geo_types::Coord>, ArcGISGeometryError> = ring
                    .into_iter()
                    .map(|pt| {
                        if pt.len() < 2 {
                            return Err(ArcGISGeometryError::new(
                                ArcGISGeometryErrorKind::InvalidCoordinateLength {
                                    expected: 2,
                                    actual: pt.len(),
                                },
                            ));
                        }
                        Ok(geo_types::Coord { x: pt[0], y: pt[1] })
                    })
                    .collect();
                coords.map(geo_types::LineString::new)
            })
            .collect();

        Ok(geo_types::Polygon::new(exterior, interiors?))
    }
}

impl From<geo_types::Polygon> for ArcGISPolygon {
    #[instrument(skip(polygon))]
    fn from(polygon: geo_types::Polygon) -> Self {
        tracing::debug!("Converting geo_types::Polygon to ArcGISPolygon");
        let (exterior, interiors) = polygon.into_inner();

        let mut rings = vec![exterior
            .into_inner()
            .into_iter()
            .map(|coord| vec![coord.x, coord.y])
            .collect()];

        rings.extend(interiors.into_iter().map(|interior| {
            interior
                .into_inner()
                .into_iter()
                .map(|coord| vec![coord.x, coord.y])
                .collect()
        }));

        ArcGISPolygon::new(rings)
    }
}

impl From<geo_types::MultiPolygon> for ArcGISPolygon {
    #[instrument(skip(mp))]
    fn from(mp: geo_types::MultiPolygon) -> Self {
        tracing::debug!("Converting geo_types::MultiPolygon to ArcGISPolygon");
        let rings = mp
            .into_iter()
            .flat_map(|polygon| {
                let (exterior, interiors) = polygon.into_inner();
                let mut all_rings = vec![exterior
                    .into_inner()
                    .into_iter()
                    .map(|coord| vec![coord.x, coord.y])
                    .collect()];

                all_rings.extend(interiors.into_iter().map(|interior| {
                    interior
                        .into_inner()
                        .into_iter()
                        .map(|coord| vec![coord.x, coord.y])
                        .collect()
                }));

                all_rings
            })
            .collect();

        ArcGISPolygon::new(rings)
    }
}

// ============================================================================
// geo-types conversions for ArcGISMultipoint
// ============================================================================

impl TryFrom<ArcGISMultipoint> for geo_types::MultiPoint {
    type Error = crate::geometry::new_errors::ArcGISGeometryError;

    #[instrument(skip(multipoint), fields(point_count = multipoint.points.len()))]
    fn try_from(multipoint: ArcGISMultipoint) -> Result<Self, Self::Error> {
        use crate::geometry::new_errors::{ArcGISGeometryError, ArcGISGeometryErrorKind};

        tracing::debug!("Converting ArcGISMultipoint to geo_types::MultiPoint");

        if multipoint.points.is_empty() {
            return Err(ArcGISGeometryError::new(ArcGISGeometryErrorKind::EmptyGeometry(
                "multipoint".to_string(),
                "conversion to MultiPoint".to_string(),
            )));
        }

        let points: Result<Vec<geo_types::Point>, ArcGISGeometryError> = multipoint
            .points
            .into_iter()
            .enumerate()
            .map(|(_idx, pt)| {
                if pt.len() < 2 {
                    return Err(ArcGISGeometryError::new(
                        ArcGISGeometryErrorKind::InvalidCoordinateLength {
                            expected: 2,
                            actual: pt.len(),
                        },
                    ));
                }
                Ok(geo_types::Point::new(pt[0], pt[1]))
            })
            .collect();

        points.map(geo_types::MultiPoint::new)
    }
}

impl From<geo_types::MultiPoint> for ArcGISMultipoint {
    #[instrument(skip(mp))]
    fn from(mp: geo_types::MultiPoint) -> Self {
        tracing::debug!("Converting geo_types::MultiPoint to ArcGISMultipoint");
        let points = mp
            .into_iter()
            .map(|point| vec![point.x(), point.y()])
            .collect();
        ArcGISMultipoint::new(points)
    }
}

// ============================================================================
// geo-types conversions for ArcGISGeometry
// ============================================================================

impl From<geo_types::Geometry> for ArcGISGeometry {
    #[instrument(skip(geom))]
    fn from(geom: geo_types::Geometry) -> Self {
        tracing::debug!("Converting geo_types::Geometry to ArcGISGeometry");
        match geom {
            geo_types::Geometry::Point(p) => ArcGISGeometry::Point(p.into()),
            geo_types::Geometry::Line(l) => {
                ArcGISGeometry::Polyline(geo_types::LineString::from(l).into())
            }
            geo_types::Geometry::LineString(ls) => ArcGISGeometry::Polyline(ls.into()),
            geo_types::Geometry::Polygon(p) => ArcGISGeometry::Polygon(p.into()),
            geo_types::Geometry::MultiPoint(mp) => ArcGISGeometry::Multipoint(mp.into()),
            geo_types::Geometry::MultiLineString(mls) => ArcGISGeometry::Polyline(mls.into()),
            geo_types::Geometry::MultiPolygon(mp) => ArcGISGeometry::Polygon(mp.into()),
            geo_types::Geometry::GeometryCollection(gc) => {
                // For GeometryCollection, convert first geometry if available
                // Otherwise create empty point at origin
                if let Some(first) = gc.into_iter().next() {
                    first.into()
                } else {
                    ArcGISGeometry::Point(ArcGISPoint::new(0.0, 0.0))
                }
            }
            geo_types::Geometry::Rect(r) => {
                let (min, max) = (r.min(), r.max());
                ArcGISGeometry::Envelope(ArcGISEnvelope::new(min.x, min.y, max.x, max.y))
            }
            geo_types::Geometry::Triangle(t) => {
                let coords: Vec<Vec<f64>> = t
                    .to_array()
                    .iter()
                    .map(|c| vec![c.x, c.y])
                    .chain(std::iter::once(vec![t.0.x, t.0.y])) // Close the ring
                    .collect();
                ArcGISGeometry::Polygon(ArcGISPolygon::new(vec![coords]))
            }
        }
    }
}

impl TryFrom<ArcGISGeometry> for geo_types::Geometry {
    type Error = crate::geometry::new_errors::ArcGISGeometryError;

    #[instrument(skip(geom))]
    fn try_from(geom: ArcGISGeometry) -> Result<Self, Self::Error> {
        tracing::debug!("Converting ArcGISGeometry to geo_types::Geometry");
        match geom {
            ArcGISGeometry::Point(p) => Ok(geo_types::Geometry::Point(p.into())),
            ArcGISGeometry::Polyline(pl) => {
                let mls: geo_types::MultiLineString = pl.try_into()?;
                Ok(geo_types::Geometry::MultiLineString(mls))
            }
            ArcGISGeometry::Polygon(pg) => {
                let p: geo_types::Polygon = pg.try_into()?;
                Ok(geo_types::Geometry::Polygon(p))
            }
            ArcGISGeometry::Multipoint(mp) => {
                let mpt: geo_types::MultiPoint = mp.try_into()?;
                Ok(geo_types::Geometry::MultiPoint(mpt))
            }
            ArcGISGeometry::Envelope(e) => Ok(geo_types::Geometry::Rect(e.into())),
        }
    }
}

// ============================================================================
// ArcGISGeometry (polymorphic wrapper)
// ============================================================================

/// Polymorphic ESRI geometry type.
///
/// This enum represents any ESRI geometry type and supports type-tagged
/// JSON serialization for API responses.
///
/// # Examples
///
/// ```
/// # use arcgis::{ArcGISGeometry, ArcGISPoint};
/// let point = ArcGISPoint::new(-118.2437, 34.0522);
/// let geometry = ArcGISGeometry::Point(point);
///
/// // Serializes with geometry type tag
/// let json = serde_json::to_string(&geometry)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ArcGISGeometry {
    /// Point geometry.
    #[serde(rename = "point")]
    Point(ArcGISPoint),

    /// Polyline geometry.
    #[serde(rename = "polyline")]
    Polyline(ArcGISPolyline),

    /// Polygon geometry.
    #[serde(rename = "polygon")]
    Polygon(ArcGISPolygon),

    /// Multipoint geometry.
    #[serde(rename = "multipoint")]
    Multipoint(ArcGISMultipoint),

    /// Envelope (bounding box) geometry.
    #[serde(rename = "envelope")]
    Envelope(ArcGISEnvelope),
}

impl ArcGISGeometry {
    /// Returns the spatial reference if present.
    ///
    /// # Examples
    ///
    /// ```
    /// # use arcgis::{ArcGISGeometry, ArcGISPoint};
    /// let point = ArcGISPoint::new(-118.2437, 34.0522);
    /// let geometry = ArcGISGeometry::Point(point);
    /// assert!(geometry.spatial_reference().is_some());
    /// ```
    pub fn spatial_reference(&self) -> Option<&SpatialReference> {
        match self {
            ArcGISGeometry::Point(p) => p.spatial_reference().as_ref(),
            ArcGISGeometry::Polyline(pl) => pl.spatial_reference().as_ref(),
            ArcGISGeometry::Polygon(pg) => pg.spatial_reference().as_ref(),
            ArcGISGeometry::Multipoint(mp) => mp.spatial_reference().as_ref(),
            ArcGISGeometry::Envelope(e) => e.spatial_reference().as_ref(),
        }
    }
}

// Convenience From impls for each geometry type
impl From<ArcGISPoint> for ArcGISGeometry {
    fn from(point: ArcGISPoint) -> Self {
        ArcGISGeometry::Point(point)
    }
}

impl From<ArcGISPolyline> for ArcGISGeometry {
    fn from(polyline: ArcGISPolyline) -> Self {
        ArcGISGeometry::Polyline(polyline)
    }
}

impl From<ArcGISPolygon> for ArcGISGeometry {
    fn from(polygon: ArcGISPolygon) -> Self {
        ArcGISGeometry::Polygon(polygon)
    }
}

impl From<ArcGISMultipoint> for ArcGISGeometry {
    fn from(multipoint: ArcGISMultipoint) -> Self {
        ArcGISGeometry::Multipoint(multipoint)
    }
}

impl From<ArcGISEnvelope> for ArcGISGeometry {
    fn from(envelope: ArcGISEnvelope) -> Self {
        ArcGISGeometry::Envelope(envelope)
    }
}

// ============================================================================
// ArcGISEnvelope
// ============================================================================

/// ESRI Envelope (bounding box).
///
/// # Examples
///
/// ```
/// # use arcgis::ArcGISEnvelope;
/// let envelope = ArcGISEnvelope::new(-120.0, 38.0, -119.0, 39.0);
/// assert_eq!(*envelope.xmin(), -120.0);
/// assert_eq!(*envelope.xmax(), -119.0);
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Getters,
    Setters,
    derive_builder::Builder,
    derive_new::new,
)]
#[builder(setter(into, strip_option))]
#[setters(prefix = "with_")]
#[serde(rename_all = "camelCase")]
pub struct ArcGISEnvelope {
    /// Minimum X coordinate.
    xmin: f64,

    /// Minimum Y coordinate.
    ymin: f64,

    /// Maximum X coordinate.
    xmax: f64,

    /// Maximum Y coordinate.
    ymax: f64,

    /// Minimum Z coordinate.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[new(default)]
    zmin: Option<f64>,

    /// Maximum Z coordinate.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[new(default)]
    zmax: Option<f64>,

    /// Minimum M value.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[new(default)]
    mmin: Option<f64>,

    /// Maximum M value.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[new(default)]
    mmax: Option<f64>,

    /// Spatial reference system.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    #[new(default)]
    spatial_reference: Option<SpatialReference>,
}

impl From<ArcGISEnvelope> for geo_types::Rect {
    #[instrument(skip(env))]
    fn from(env: ArcGISEnvelope) -> Self {
        tracing::debug!("Converting ArcGISEnvelope to geo_types::Rect");
        geo_types::Rect::new(
            geo_types::Coord {
                x: env.xmin,
                y: env.ymin,
            },
            geo_types::Coord {
                x: env.xmax,
                y: env.ymax,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
            )
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn test_esri_point_new() -> anyhow::Result<()> {
        init_tracing();
        let point = ArcGISPoint::new(-118.2437, 34.0522);

        assert_eq!(*point.x(), -118.2437);
        assert_eq!(*point.y(), 34.0522);
        assert_eq!(*point.z(), None);
        assert_eq!(*point.m(), None);
        assert!(point.spatial_reference().is_some());

        Ok(())
    }

    #[test]
    fn test_esri_point_with_z() -> anyhow::Result<()> {
        init_tracing();
        let point = ArcGISPoint::with_z(-118.2437, 34.0522, 100.0);

        assert_eq!(*point.x(), -118.2437);
        assert_eq!(*point.y(), 34.0522);
        assert_eq!(*point.z(), Some(100.0));

        Ok(())
    }

    #[test]
    fn test_point_to_geo_types() -> anyhow::Result<()> {
        init_tracing();
        let esri_pt = ArcGISPoint::new(-118.2437, 34.0522);

        let geo_pt: geo_types::Point = esri_pt.into();
        assert_eq!(geo_pt.x(), -118.2437);
        assert_eq!(geo_pt.y(), 34.0522);

        Ok(())
    }

    #[test]
    fn test_geo_types_to_point() -> anyhow::Result<()> {
        init_tracing();
        let geo_pt = geo_types::Point::new(-120.0, 38.0);

        let esri_pt: ArcGISPoint = geo_pt.into();
        assert_eq!(*esri_pt.x(), -120.0);
        assert_eq!(*esri_pt.y(), 38.0);

        Ok(())
    }

    #[test]
    fn test_envelope_new() -> anyhow::Result<()> {
        init_tracing();
        let env = ArcGISEnvelope::new(-120.0, 38.0, -119.0, 39.0);

        assert_eq!(*env.xmin(), -120.0);
        assert_eq!(*env.ymin(), 38.0);
        assert_eq!(*env.xmax(), -119.0);
        assert_eq!(*env.ymax(), 39.0);

        Ok(())
    }

    #[test]
    fn test_envelope_to_rect() -> anyhow::Result<()> {
        init_tracing();
        let env = ArcGISEnvelope::new(-120.0, 38.0, -119.0, 39.0);

        let rect: geo_types::Rect = env.into();
        assert_eq!(rect.min().x, -120.0);
        assert_eq!(rect.min().y, 38.0);
        assert_eq!(rect.max().x, -119.0);
        assert_eq!(rect.max().y, 39.0);

        Ok(())
    }

    #[test]
    fn test_polyline_new() -> anyhow::Result<()> {
        init_tracing();
        let paths = vec![vec![vec![-118.0, 34.0], vec![-117.0, 33.0]]];
        let polyline = ArcGISPolyline::new(paths);

        assert_eq!(polyline.paths().len(), 1);
        assert_eq!(polyline.paths()[0].len(), 2);
        assert!(polyline.spatial_reference().is_some());

        Ok(())
    }

    #[test]
    fn test_polyline_to_multilinestring() -> anyhow::Result<()> {
        init_tracing();
        let paths = vec![
            vec![vec![-118.0, 34.0], vec![-117.0, 33.0]],
            vec![vec![-116.0, 32.0], vec![-115.0, 31.0]],
        ];
        let polyline = ArcGISPolyline::new(paths);

        let mls: geo_types::MultiLineString = polyline.try_into()?;
        assert_eq!(mls.0.len(), 2);
        assert_eq!(mls.0[0].0.len(), 2);

        Ok(())
    }

    #[test]
    fn test_linestring_to_polyline() -> anyhow::Result<()> {
        init_tracing();
        let coords = vec![
            geo_types::Coord { x: -118.0, y: 34.0 },
            geo_types::Coord { x: -117.0, y: 33.0 },
        ];
        let line = geo_types::LineString::new(coords);

        let polyline: ArcGISPolyline = line.into();
        assert_eq!(polyline.paths().len(), 1);
        assert_eq!(polyline.paths()[0].len(), 2);

        Ok(())
    }

    #[test]
    fn test_multilinestring_to_polyline() -> anyhow::Result<()> {
        init_tracing();
        let line1 = geo_types::LineString::new(vec![
            geo_types::Coord { x: -118.0, y: 34.0 },
            geo_types::Coord { x: -117.0, y: 33.0 },
        ]);
        let line2 = geo_types::LineString::new(vec![
            geo_types::Coord { x: -116.0, y: 32.0 },
            geo_types::Coord { x: -115.0, y: 31.0 },
        ]);
        let mls = geo_types::MultiLineString::new(vec![line1, line2]);

        let polyline: ArcGISPolyline = mls.into();
        assert_eq!(polyline.paths().len(), 2);

        Ok(())
    }

    #[test]
    fn test_polyline_empty_error() -> anyhow::Result<()> {
        init_tracing();
        let polyline = ArcGISPolyline::new(vec![]);

        let result: Result<geo_types::MultiLineString, _> = polyline.try_into();
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_polygon_new() -> anyhow::Result<()> {
        init_tracing();
        let rings = vec![vec![
            vec![-118.0, 34.0],
            vec![-117.0, 34.0],
            vec![-117.0, 33.0],
            vec![-118.0, 34.0],
        ]];
        let polygon = ArcGISPolygon::new(rings);

        assert_eq!(polygon.rings().len(), 1);
        assert_eq!(polygon.rings()[0].len(), 4);
        assert!(polygon.spatial_reference().is_some());

        Ok(())
    }

    #[test]
    fn test_polygon_to_geo_polygon() -> anyhow::Result<()> {
        init_tracing();
        let rings = vec![vec![
            vec![-118.0, 34.0],
            vec![-117.0, 34.0],
            vec![-117.0, 33.0],
            vec![-118.0, 34.0],
        ]];
        let esri_polygon = ArcGISPolygon::new(rings);

        let geo_polygon: geo_types::Polygon = esri_polygon.try_into()?;
        assert_eq!(geo_polygon.exterior().0.len(), 4);

        Ok(())
    }

    #[test]
    fn test_geo_polygon_to_esri() -> anyhow::Result<()> {
        init_tracing();
        let exterior = geo_types::LineString::new(vec![
            geo_types::Coord { x: -118.0, y: 34.0 },
            geo_types::Coord { x: -117.0, y: 34.0 },
            geo_types::Coord { x: -117.0, y: 33.0 },
            geo_types::Coord { x: -118.0, y: 34.0 },
        ]);
        let geo_polygon = geo_types::Polygon::new(exterior, vec![]);

        let esri_polygon: ArcGISPolygon = geo_polygon.into();
        assert_eq!(esri_polygon.rings().len(), 1);
        assert_eq!(esri_polygon.rings()[0].len(), 4);

        Ok(())
    }

    #[test]
    fn test_polygon_with_holes() -> anyhow::Result<()> {
        init_tracing();
        let exterior = geo_types::LineString::new(vec![
            geo_types::Coord { x: -118.0, y: 34.0 },
            geo_types::Coord { x: -117.0, y: 34.0 },
            geo_types::Coord { x: -117.0, y: 33.0 },
            geo_types::Coord { x: -118.0, y: 34.0 },
        ]);
        let hole = geo_types::LineString::new(vec![
            geo_types::Coord {
                x: -117.8,
                y: 33.8,
            },
            geo_types::Coord {
                x: -117.2,
                y: 33.8,
            },
            geo_types::Coord {
                x: -117.2,
                y: 33.2,
            },
            geo_types::Coord {
                x: -117.8,
                y: 33.8,
            },
        ]);
        let geo_polygon = geo_types::Polygon::new(exterior, vec![hole]);

        let esri_polygon: ArcGISPolygon = geo_polygon.into();
        assert_eq!(esri_polygon.rings().len(), 2);

        Ok(())
    }

    #[test]
    fn test_polygon_empty_error() -> anyhow::Result<()> {
        init_tracing();
        let polygon = ArcGISPolygon::new(vec![]);

        let result: Result<geo_types::Polygon, _> = polygon.try_into();
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_multipoint_new() -> anyhow::Result<()> {
        init_tracing();
        let points = vec![vec![-118.0, 34.0], vec![-117.0, 33.0]];
        let multipoint = ArcGISMultipoint::new(points);

        assert_eq!(multipoint.points().len(), 2);
        assert!(multipoint.spatial_reference().is_some());

        Ok(())
    }

    #[test]
    fn test_multipoint_to_geo() -> anyhow::Result<()> {
        init_tracing();
        let points = vec![vec![-118.0, 34.0], vec![-117.0, 33.0]];
        let esri_mp = ArcGISMultipoint::new(points);

        let geo_mp: geo_types::MultiPoint = esri_mp.try_into()?;
        assert_eq!(geo_mp.0.len(), 2);

        Ok(())
    }

    #[test]
    fn test_geo_multipoint_to_esri() -> anyhow::Result<()> {
        init_tracing();
        let points = vec![
            geo_types::Point::new(-118.0, 34.0),
            geo_types::Point::new(-117.0, 33.0),
        ];
        let geo_mp = geo_types::MultiPoint::new(points);

        let esri_mp: ArcGISMultipoint = geo_mp.into();
        assert_eq!(esri_mp.points().len(), 2);

        Ok(())
    }

    #[test]
    fn test_multipoint_empty_error() -> anyhow::Result<()> {
        init_tracing();
        let multipoint = ArcGISMultipoint::new(vec![]);

        let result: Result<geo_types::MultiPoint, _> = multipoint.try_into();
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_invalid_coordinate_length() -> anyhow::Result<()> {
        init_tracing();
        let points = vec![vec![-118.0]]; // Only 1 coordinate
        let multipoint = ArcGISMultipoint::new(points);

        let result: Result<geo_types::MultiPoint, _> = multipoint.try_into();
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_esri_geometry_point() -> anyhow::Result<()> {
        init_tracing();
        let point = ArcGISPoint::new(-118.2437, 34.0522);
        let geometry = ArcGISGeometry::Point(point);

        assert!(geometry.spatial_reference().is_some());

        Ok(())
    }

    #[test]
    fn test_esri_geometry_from_point() -> anyhow::Result<()> {
        init_tracing();
        let point = ArcGISPoint::new(-118.2437, 34.0522);
        let geometry: ArcGISGeometry = point.into();

        match geometry {
            ArcGISGeometry::Point(_) => (),
            _ => panic!("Expected Point variant"),
        }

        Ok(())
    }

    #[test]
    fn test_esri_geometry_serialization() -> anyhow::Result<()> {
        init_tracing();
        let point = ArcGISPoint::new(-118.2437, 34.0522);
        let geometry = ArcGISGeometry::Point(point);

        let json = serde_json::to_string(&geometry)?;
        assert!(json.contains(r#""type":"point""#));

        Ok(())
    }

    #[test]
    fn test_esri_geometry_deserialization() -> anyhow::Result<()> {
        init_tracing();
        let json = r#"{"type":"point","x":-118.2437,"y":34.0522,"spatialReference":{"wkid":4326,"latestWkid":4326}}"#;
        let geometry: ArcGISGeometry = serde_json::from_str(json)?;

        match geometry {
            ArcGISGeometry::Point(p) => {
                assert_eq!(*p.x(), -118.2437);
                assert_eq!(*p.y(), 34.0522);
            }
            _ => panic!("Expected Point variant"),
        }

        Ok(())
    }

    #[test]
    fn test_geo_geometry_to_esri() -> anyhow::Result<()> {
        init_tracing();
        let geo_point = geo_types::Point::new(-118.2437, 34.0522);
        let geo_geom = geo_types::Geometry::Point(geo_point);

        let esri_geom: ArcGISGeometry = geo_geom.into();

        match esri_geom {
            ArcGISGeometry::Point(p) => {
                assert_eq!(*p.x(), -118.2437);
                assert_eq!(*p.y(), 34.0522);
            }
            _ => panic!("Expected Point variant"),
        }

        Ok(())
    }

    #[test]
    fn test_esri_geometry_to_geo() -> anyhow::Result<()> {
        init_tracing();
        let point = ArcGISPoint::new(-118.2437, 34.0522);
        let esri_geom = ArcGISGeometry::Point(point);

        let geo_geom: geo_types::Geometry = esri_geom.try_into()?;

        match geo_geom {
            geo_types::Geometry::Point(p) => {
                assert_eq!(p.x(), -118.2437);
                assert_eq!(p.y(), 34.0522);
            }
            _ => panic!("Expected Point variant"),
        }

        Ok(())
    }

    #[test]
    fn test_geo_polygon_to_esri_geometry() -> anyhow::Result<()> {
        init_tracing();
        let exterior = geo_types::LineString::new(vec![
            geo_types::Coord { x: -118.0, y: 34.0 },
            geo_types::Coord { x: -117.0, y: 34.0 },
            geo_types::Coord { x: -117.0, y: 33.0 },
            geo_types::Coord { x: -118.0, y: 34.0 },
        ]);
        let geo_polygon = geo_types::Polygon::new(exterior, vec![]);
        let geo_geom = geo_types::Geometry::Polygon(geo_polygon);

        let esri_geom: ArcGISGeometry = geo_geom.into();

        match esri_geom {
            ArcGISGeometry::Polygon(p) => {
                assert_eq!(p.rings().len(), 1);
            }
            _ => panic!("Expected Polygon variant"),
        }

        Ok(())
    }

    #[test]
    fn test_geo_rect_to_esri_envelope() -> anyhow::Result<()> {
        init_tracing();
        let rect = geo_types::Rect::new(
            geo_types::Coord { x: -120.0, y: 38.0 },
            geo_types::Coord { x: -119.0, y: 39.0 },
        );
        let geo_geom = geo_types::Geometry::Rect(rect);

        let esri_geom: ArcGISGeometry = geo_geom.into();

        match esri_geom {
            ArcGISGeometry::Envelope(e) => {
                assert_eq!(*e.xmin(), -120.0);
                assert_eq!(*e.ymin(), 38.0);
                assert_eq!(*e.xmax(), -119.0);
                assert_eq!(*e.ymax(), 39.0);
            }
            _ => panic!("Expected Envelope variant"),
        }

        Ok(())
    }
}
