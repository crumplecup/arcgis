//! Tests for geometry-related enums (GeometryType, SpatialRel).

mod common;

use arcgis::{GeometryType, SpatialRel};

#[test]
fn test_geometry_type_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_geometry_type_serialization: Starting");

    let geom_type = GeometryType::Point;
    tracing::info!("test_geometry_type_serialization: Serializing to JSON");
    let json = serde_json::to_string(&geom_type)?;

    tracing::info!(
        json = %json,
        "test_geometry_type_serialization: Verifying serialization"
    );
    assert_eq!(json, r#""esriGeometryPoint""#);

    tracing::info!("test_geometry_type_serialization: Completed");
    Ok(())
}

#[test]
fn test_geometry_type_deserialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_geometry_type_deserialization: Starting");

    let json = r#""esriGeometryPolyline""#;
    tracing::info!("test_geometry_type_deserialization: Deserializing from JSON");
    let geom_type: GeometryType = serde_json::from_str(json)?;

    tracing::info!(
        geom_type = ?geom_type,
        "test_geometry_type_deserialization: Verifying deserialization"
    );
    assert_eq!(geom_type, GeometryType::Polyline);

    tracing::info!("test_geometry_type_deserialization: Completed");
    Ok(())
}

#[test]
fn test_spatial_rel_round_trip() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_spatial_rel_round_trip: Starting");

    let spatial_rel = SpatialRel::Intersects;
    tracing::info!("test_spatial_rel_round_trip: Serializing to JSON");
    let json = serde_json::to_string(&spatial_rel)?;

    tracing::info!("test_spatial_rel_round_trip: Deserializing from JSON");
    let deserialized: SpatialRel = serde_json::from_str(&json)?;

    tracing::info!(
        original = ?spatial_rel,
        deserialized = ?deserialized,
        "test_spatial_rel_round_trip: Verifying round trip"
    );
    assert_eq!(spatial_rel, deserialized);

    tracing::info!("test_spatial_rel_round_trip: Completed");
    Ok(())
}
