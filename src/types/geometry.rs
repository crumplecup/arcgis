//! Geometry-related types.

use serde::{Deserialize, Serialize};

/// ArcGIS geometry types.
///
/// These correspond to the `esriGeometry*` constants in the ArcGIS REST API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GeometryType {
    /// Point geometry (`esriGeometryPoint`)
    #[serde(rename = "esriGeometryPoint")]
    Point,

    /// Multipoint geometry (`esriGeometryMultipoint`)
    #[serde(rename = "esriGeometryMultipoint")]
    Multipoint,

    /// Polyline geometry (`esriGeometryPolyline`)
    #[serde(rename = "esriGeometryPolyline")]
    Polyline,

    /// Polygon geometry (`esriGeometryPolygon`)
    #[serde(rename = "esriGeometryPolygon")]
    Polygon,

    /// Envelope (bounding box) geometry (`esriGeometryEnvelope`)
    #[serde(rename = "esriGeometryEnvelope")]
    Envelope,
}

/// Spatial relationship types for queries.
///
/// These define how geometries relate to each other in spatial queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SpatialRel {
    /// Geometries intersect (`esriSpatialRelIntersects`)
    #[serde(rename = "esriSpatialRelIntersects")]
    Intersects,

    /// First geometry contains the second (`esriSpatialRelContains`)
    #[serde(rename = "esriSpatialRelContains")]
    Contains,

    /// Geometries cross (`esriSpatialRelCrosses`)
    #[serde(rename = "esriSpatialRelCrosses")]
    Crosses,

    /// Envelopes intersect (`esriSpatialRelEnvelopeIntersects`)
    #[serde(rename = "esriSpatialRelEnvelopeIntersects")]
    EnvelopeIntersects,

    /// Index-based intersection (`esriSpatialRelIndexIntersects`)
    #[serde(rename = "esriSpatialRelIndexIntersects")]
    IndexIntersects,

    /// Geometries overlap (`esriSpatialRelOverlaps`)
    #[serde(rename = "esriSpatialRelOverlaps")]
    Overlaps,

    /// Geometries touch (`esriSpatialRelTouches`)
    #[serde(rename = "esriSpatialRelTouches")]
    Touches,

    /// First geometry is within the second (`esriSpatialRelWithin`)
    #[serde(rename = "esriSpatialRelWithin")]
    Within,

    /// Geometric relationship (`esriSpatialRelRelation`)
    #[serde(rename = "esriSpatialRelRelation")]
    Relation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn test_geometry_type_serialization() -> Result<()> {
        let geom_type = GeometryType::Point;
        let json = serde_json::to_string(&geom_type)?;
        assert_eq!(json, r#""esriGeometryPoint""#);
        Ok(())
    }

    #[test]
    fn test_geometry_type_deserialization() -> Result<()> {
        let json = r#""esriGeometryPolyline""#;
        let geom_type: GeometryType = serde_json::from_str(json)?;
        assert_eq!(geom_type, GeometryType::Polyline);
        Ok(())
    }

    #[test]
    fn test_spatial_rel_round_trip() -> Result<()> {
        let spatial_rel = SpatialRel::Intersects;
        let json = serde_json::to_string(&spatial_rel)?;
        let deserialized: SpatialRel = serde_json::from_str(&json)?;
        assert_eq!(spatial_rel, deserialized);
        Ok(())
    }
}
