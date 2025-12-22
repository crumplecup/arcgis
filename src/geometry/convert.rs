//! Conversion functions between ArcGIS JSON and geo-types.

use crate::geometry::arcgis::{
    ArcGISEnvelope, ArcGISGeometry, ArcGISMultipoint, ArcGISPoint, ArcGISPolygon, ArcGISPolyline,
    SpatialReference,
};
use crate::{Error, Result};
use geo_types::{
    Coord, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect,
};
use tracing::instrument;

// ============================================================================
// Point Conversions
// ============================================================================

/// Converts an ArcGIS Point to a geo-types Point.
#[instrument(skip(point))]
pub fn from_arcgis_point(point: &ArcGISPoint) -> Result<Point> {
    tracing::debug!(
        x = point.x,
        y = point.y,
        "Converting ArcGIS Point to geo-types"
    );
    Ok(Point::new(point.x, point.y))
}

/// Converts a geo-types Point to an ArcGIS Point.
#[instrument(skip(point))]
pub fn to_arcgis_point(point: &Point) -> Result<ArcGISPoint> {
    tracing::debug!(
        x = point.x(),
        y = point.y(),
        "Converting geo-types Point to ArcGIS"
    );
    Ok(ArcGISPoint {
        x: point.x(),
        y: point.y(),
        z: None,
        m: None,
        spatial_reference: Some(SpatialReference::wgs84()),
    })
}

// ============================================================================
// MultiPoint Conversions
// ============================================================================

/// Converts an ArcGIS Multipoint to a geo-types MultiPoint.
#[instrument(skip(multipoint))]
pub fn from_arcgis_multipoint(multipoint: &ArcGISMultipoint) -> Result<MultiPoint> {
    tracing::debug!(
        point_count = multipoint.points.len(),
        "Converting ArcGIS Multipoint"
    );
    let points: Vec<Point> = multipoint
        .points
        .iter()
        .map(|[x, y]| Point::new(*x, *y))
        .collect();
    Ok(MultiPoint::new(points))
}

/// Converts a geo-types MultiPoint to an ArcGIS Multipoint.
#[instrument(skip(multipoint))]
pub fn to_arcgis_multipoint(multipoint: &MultiPoint) -> Result<ArcGISMultipoint> {
    tracing::debug!(
        point_count = multipoint.0.len(),
        "Converting geo-types MultiPoint"
    );
    let points: Vec<[f64; 2]> = multipoint.0.iter().map(|p| [p.x(), p.y()]).collect();
    Ok(ArcGISMultipoint {
        points,
        spatial_reference: Some(SpatialReference::wgs84()),
    })
}

// ============================================================================
// LineString/Polyline Conversions
// ============================================================================

/// Converts an ArcGIS Polyline to a geo-types LineString (first path only).
///
/// If the polyline has multiple paths, only the first path is converted.
/// Use `from_arcgis_polyline_multi` to convert all paths.
#[instrument(skip(polyline))]
pub fn from_arcgis_polyline(polyline: &ArcGISPolyline) -> Result<LineString> {
    tracing::debug!(
        path_count = polyline.paths.len(),
        "Converting ArcGIS Polyline"
    );
    if polyline.paths.is_empty() {
        return Err(Error::geometry("Polyline has no paths"));
    }

    let coords: Vec<Coord> = polyline.paths[0]
        .iter()
        .map(|[x, y]| Coord { x: *x, y: *y })
        .collect();

    Ok(LineString::new(coords))
}

/// Converts an ArcGIS Polyline to a geo-types MultiLineString (all paths).
#[instrument(skip(polyline))]
pub fn from_arcgis_polyline_multi(polyline: &ArcGISPolyline) -> Result<MultiLineString> {
    tracing::debug!(
        path_count = polyline.paths.len(),
        "Converting ArcGIS Polyline to MultiLineString"
    );
    let line_strings: Vec<LineString> = polyline
        .paths
        .iter()
        .map(|path| {
            let coords: Vec<Coord> = path.iter().map(|[x, y]| Coord { x: *x, y: *y }).collect();
            LineString::new(coords)
        })
        .collect();

    Ok(MultiLineString::new(line_strings))
}

/// Converts a geo-types LineString to an ArcGIS Polyline (single path).
#[instrument(skip(line_string))]
pub fn to_arcgis_polyline(line_string: &LineString) -> Result<ArcGISPolyline> {
    tracing::debug!(
        coord_count = line_string.0.len(),
        "Converting geo-types LineString"
    );
    let path: Vec<[f64; 2]> = line_string.0.iter().map(|c| [c.x, c.y]).collect();

    Ok(ArcGISPolyline {
        paths: vec![path],
        spatial_reference: Some(SpatialReference::wgs84()),
    })
}

/// Converts a geo-types MultiLineString to an ArcGIS Polyline (multiple paths).
#[instrument(skip(multi_line_string))]
pub fn to_arcgis_polyline_multi(multi_line_string: &MultiLineString) -> Result<ArcGISPolyline> {
    tracing::debug!(
        line_count = multi_line_string.0.len(),
        "Converting geo-types MultiLineString"
    );
    let paths: Vec<Vec<[f64; 2]>> = multi_line_string
        .0
        .iter()
        .map(|ls| ls.0.iter().map(|c| [c.x, c.y]).collect())
        .collect();

    Ok(ArcGISPolyline {
        paths,
        spatial_reference: Some(SpatialReference::wgs84()),
    })
}

// ============================================================================
// Polygon Conversions
// ============================================================================

/// Converts an ArcGIS Polygon to a geo-types Polygon.
#[instrument(skip(polygon))]
pub fn from_arcgis_polygon(polygon: &ArcGISPolygon) -> Result<Polygon> {
    tracing::debug!(
        ring_count = polygon.rings.len(),
        "Converting ArcGIS Polygon"
    );
    if polygon.rings.is_empty() {
        return Err(Error::geometry("Polygon has no rings"));
    }

    // First ring is exterior
    let exterior_coords: Vec<Coord> = polygon.rings[0]
        .iter()
        .map(|[x, y]| Coord { x: *x, y: *y })
        .collect();
    let exterior = LineString::new(exterior_coords);

    // Remaining rings are holes
    let interiors: Vec<LineString> = polygon.rings[1..]
        .iter()
        .map(|ring| {
            let coords: Vec<Coord> = ring.iter().map(|[x, y]| Coord { x: *x, y: *y }).collect();
            LineString::new(coords)
        })
        .collect();

    Ok(Polygon::new(exterior, interiors))
}

/// Converts a geo-types Polygon to an ArcGIS Polygon.
#[instrument(skip(polygon))]
pub fn to_arcgis_polygon(polygon: &Polygon) -> Result<ArcGISPolygon> {
    tracing::debug!(
        interior_count = polygon.interiors().len(),
        "Converting geo-types Polygon"
    );
    let mut rings: Vec<Vec<[f64; 2]>> = Vec::new();

    // Exterior ring
    let exterior: Vec<[f64; 2]> = polygon.exterior().0.iter().map(|c| [c.x, c.y]).collect();
    rings.push(exterior);

    // Interior rings (holes)
    for interior in polygon.interiors() {
        let ring: Vec<[f64; 2]> = interior.0.iter().map(|c| [c.x, c.y]).collect();
        rings.push(ring);
    }

    Ok(ArcGISPolygon {
        rings,
        spatial_reference: Some(SpatialReference::wgs84()),
    })
}

/// Converts an ArcGIS Polygon to a geo-types MultiPolygon (single polygon).
#[instrument(skip(polygon))]
pub fn from_arcgis_polygon_multi(polygon: &ArcGISPolygon) -> Result<MultiPolygon> {
    let poly = from_arcgis_polygon(polygon)?;
    Ok(MultiPolygon::new(vec![poly]))
}

/// Converts a geo-types MultiPolygon to an ArcGIS Polygon.
///
/// Note: ArcGIS Polygon can represent multiple polygons via separate rings,
/// but this conversion treats each polygon in the MultiPolygon as separate.
/// For proper multi-polygon support, consider returning multiple ArcGIS Polygons.
#[instrument(skip(multi_polygon))]
pub fn to_arcgis_polygon_multi(multi_polygon: &MultiPolygon) -> Result<Vec<ArcGISPolygon>> {
    tracing::debug!(
        polygon_count = multi_polygon.0.len(),
        "Converting geo-types MultiPolygon"
    );
    let polygons: Result<Vec<ArcGISPolygon>> =
        multi_polygon.0.iter().map(to_arcgis_polygon).collect();
    polygons
}

// ============================================================================
// Envelope/Rect Conversions
// ============================================================================

/// Converts an ArcGIS Envelope to a geo-types Rect.
#[instrument(skip(envelope))]
pub fn from_arcgis_envelope(envelope: &ArcGISEnvelope) -> Result<Rect> {
    tracing::debug!(
        xmin = envelope.xmin,
        ymin = envelope.ymin,
        xmax = envelope.xmax,
        ymax = envelope.ymax,
        "Converting ArcGIS Envelope"
    );
    Ok(Rect::new(
        Coord {
            x: envelope.xmin,
            y: envelope.ymin,
        },
        Coord {
            x: envelope.xmax,
            y: envelope.ymax,
        },
    ))
}

/// Converts a geo-types Rect to an ArcGIS Envelope.
#[instrument(skip(rect))]
pub fn to_arcgis_envelope(rect: &Rect) -> Result<ArcGISEnvelope> {
    tracing::debug!("Converting geo-types Rect to ArcGIS Envelope");
    Ok(ArcGISEnvelope {
        xmin: rect.min().x,
        ymin: rect.min().y,
        xmax: rect.max().x,
        ymax: rect.max().y,
        spatial_reference: Some(SpatialReference::wgs84()),
    })
}

// ============================================================================
// Geometry Enum Conversions
// ============================================================================

/// Converts an ArcGIS Geometry to a geo-types Geometry.
#[instrument(skip(geometry))]
pub fn from_arcgis_geometry(geometry: &ArcGISGeometry) -> Result<Geometry> {
    match geometry {
        ArcGISGeometry::Point(p) => Ok(Geometry::Point(from_arcgis_point(p)?)),
        ArcGISGeometry::Multipoint(mp) => Ok(Geometry::MultiPoint(from_arcgis_multipoint(mp)?)),
        ArcGISGeometry::Polyline(pl) => {
            Ok(Geometry::MultiLineString(from_arcgis_polyline_multi(pl)?))
        }
        ArcGISGeometry::Polygon(pg) => Ok(Geometry::Polygon(from_arcgis_polygon(pg)?)),
        ArcGISGeometry::Envelope(e) => Ok(Geometry::Rect(from_arcgis_envelope(e)?)),
    }
}

/// Converts a geo-types Geometry to an ArcGIS Geometry.
#[instrument(skip(geometry))]
pub fn to_arcgis_geometry(geometry: &Geometry) -> Result<ArcGISGeometry> {
    match geometry {
        Geometry::Point(p) => Ok(ArcGISGeometry::Point(to_arcgis_point(p)?)),
        Geometry::MultiPoint(mp) => Ok(ArcGISGeometry::Multipoint(to_arcgis_multipoint(mp)?)),
        Geometry::LineString(ls) => Ok(ArcGISGeometry::Polyline(to_arcgis_polyline(ls)?)),
        Geometry::MultiLineString(mls) => {
            Ok(ArcGISGeometry::Polyline(to_arcgis_polyline_multi(mls)?))
        }
        Geometry::Polygon(pg) => Ok(ArcGISGeometry::Polygon(to_arcgis_polygon(pg)?)),
        Geometry::MultiPolygon(_mp) => Err(Error::geometry(
            "MultiPolygon to single ArcGIS Polygon not supported - use to_arcgis_polygon_multi",
        )),
        Geometry::Rect(r) => Ok(ArcGISGeometry::Envelope(to_arcgis_envelope(r)?)),
        Geometry::Line(_) => Err(Error::geometry("Line geometry not supported by ArcGIS")),
        Geometry::Triangle(_) => Err(Error::geometry("Triangle geometry not supported by ArcGIS")),
        Geometry::GeometryCollection(_) => Err(Error::geometry(
            "GeometryCollection not supported by ArcGIS",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_conversion() {
        // geo-types -> ArcGIS
        let geo_point = Point::new(-118.15, 33.80);
        let arcgis_point = to_arcgis_point(&geo_point).unwrap();

        assert_eq!(arcgis_point.x, -118.15);
        assert_eq!(arcgis_point.y, 33.80);

        // ArcGIS -> geo-types (round trip)
        let geo_point2 = from_arcgis_point(&arcgis_point).unwrap();
        assert_eq!(geo_point, geo_point2);
    }

    #[test]
    fn test_multipoint_conversion() {
        let points = vec![Point::new(-97.06, 32.84), Point::new(-97.06, 32.85)];
        let geo_multipoint = MultiPoint::new(points);

        let arcgis_mp = to_arcgis_multipoint(&geo_multipoint).unwrap();
        assert_eq!(arcgis_mp.points.len(), 2);
        assert_eq!(arcgis_mp.points[0], [-97.06, 32.84]);

        let geo_multipoint2 = from_arcgis_multipoint(&arcgis_mp).unwrap();
        assert_eq!(geo_multipoint, geo_multipoint2);
    }

    #[test]
    fn test_linestring_conversion() {
        let coords = vec![
            Coord {
                x: -97.06,
                y: 32.84,
            },
            Coord {
                x: -97.06,
                y: 32.85,
            },
        ];
        let geo_line = LineString::new(coords);

        let arcgis_polyline = to_arcgis_polyline(&geo_line).unwrap();
        assert_eq!(arcgis_polyline.paths.len(), 1);
        assert_eq!(arcgis_polyline.paths[0].len(), 2);

        let geo_line2 = from_arcgis_polyline(&arcgis_polyline).unwrap();
        assert_eq!(geo_line, geo_line2);
    }

    #[test]
    fn test_polygon_conversion() {
        let exterior = LineString::new(vec![
            Coord {
                x: -97.06,
                y: 32.84,
            },
            Coord {
                x: -97.06,
                y: 32.85,
            },
            Coord {
                x: -97.07,
                y: 32.85,
            },
            Coord {
                x: -97.06,
                y: 32.84,
            },
        ]);
        let geo_polygon = Polygon::new(exterior, vec![]);

        let arcgis_polygon = to_arcgis_polygon(&geo_polygon).unwrap();
        assert_eq!(arcgis_polygon.rings.len(), 1);
        assert_eq!(arcgis_polygon.rings[0].len(), 4);

        let geo_polygon2 = from_arcgis_polygon(&arcgis_polygon).unwrap();
        assert_eq!(geo_polygon, geo_polygon2);
    }

    #[test]
    fn test_envelope_conversion() {
        let rect = Rect::new(
            Coord {
                x: -109.55,
                y: 25.76,
            },
            Coord {
                x: -86.39,
                y: 49.94,
            },
        );

        let envelope = to_arcgis_envelope(&rect).unwrap();
        assert_eq!(envelope.xmin, -109.55);
        assert_eq!(envelope.ymin, 25.76);
        assert_eq!(envelope.xmax, -86.39);
        assert_eq!(envelope.ymax, 49.94);

        let rect2 = from_arcgis_envelope(&envelope).unwrap();
        assert_eq!(rect, rect2);
    }

    #[test]
    fn test_polygon_with_holes() {
        let exterior = LineString::new(vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 10.0, y: 0.0 },
            Coord { x: 10.0, y: 10.0 },
            Coord { x: 0.0, y: 10.0 },
            Coord { x: 0.0, y: 0.0 },
        ]);
        let hole = LineString::new(vec![
            Coord { x: 2.0, y: 2.0 },
            Coord { x: 8.0, y: 2.0 },
            Coord { x: 8.0, y: 8.0 },
            Coord { x: 2.0, y: 8.0 },
            Coord { x: 2.0, y: 2.0 },
        ]);
        let geo_polygon = Polygon::new(exterior, vec![hole]);

        let arcgis_polygon = to_arcgis_polygon(&geo_polygon).unwrap();
        assert_eq!(arcgis_polygon.rings.len(), 2); // Exterior + 1 hole

        let geo_polygon2 = from_arcgis_polygon(&arcgis_polygon).unwrap();
        assert_eq!(geo_polygon, geo_polygon2);
    }
}
