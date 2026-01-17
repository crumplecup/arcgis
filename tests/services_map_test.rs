//! Tests for Map Service types and client.

mod common;

use arcgis::{
    ApiKeyAuth, ArcGISClient, ExportMapParams, ExportResult, ExportTarget, GeometryType,
    IdentifyParams, ImageFormat, LayerOperation, LayerSelection, MapServiceClient,
    TileCoordinate, TimeRelation,
};

#[test]
fn test_image_format_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_image_format_serialization: Starting");

    tracing::info!("test_image_format_serialization: Testing Png format");
    let png = ImageFormat::Png;
    let serialized = serde_json::to_string(&png)?;
    assert_eq!(serialized, "\"png\"");

    tracing::info!("test_image_format_serialization: Testing Png32 format");
    let png32 = ImageFormat::Png32;
    let serialized = serde_json::to_string(&png32)?;
    assert_eq!(serialized, "\"png32\"");

    tracing::info!("test_image_format_serialization: Testing Jpg format");
    let jpg = ImageFormat::Jpg;
    let serialized = serde_json::to_string(&jpg)?;
    assert_eq!(serialized, "\"jpg\"");

    tracing::info!("test_image_format_serialization: Testing Pdf format");
    let pdf = ImageFormat::Pdf;
    let serialized = serde_json::to_string(&pdf)?;
    assert_eq!(serialized, "\"pdf\"");

    tracing::info!("test_image_format_serialization: Completed");
    Ok(())
}

#[test]
fn test_image_format_default() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_image_format_default: Starting");

    tracing::info!("test_image_format_default: Checking default format");
    let format = ImageFormat::default();
    
    tracing::info!(
        format = ?format,
        "test_image_format_default: Verifying default"
    );
    assert_eq!(format, ImageFormat::Png);

    tracing::info!("test_image_format_default: Completed");
    Ok(())
}

#[test]
fn test_layer_operation_as_str() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_layer_operation_as_str: Starting");

    tracing::info!("test_layer_operation_as_str: Testing layer operation string conversions");
    assert_eq!(LayerOperation::Show.as_str(), "show");
    assert_eq!(LayerOperation::Hide.as_str(), "hide");
    assert_eq!(LayerOperation::Include.as_str(), "include");
    assert_eq!(LayerOperation::Exclude.as_str(), "exclude");

    tracing::info!("test_layer_operation_as_str: Completed");
    Ok(())
}

#[test]
fn test_time_relation_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_time_relation_serialization: Starting");

    tracing::info!("test_time_relation_serialization: Testing Overlaps serialization");
    let overlaps = TimeRelation::Overlaps;
    let serialized = serde_json::to_string(&overlaps)?;
    assert_eq!(serialized, "\"esriTimeRelationOverlaps\"");

    tracing::info!("test_time_relation_serialization: Testing After serialization");
    let after = TimeRelation::After;
    let serialized = serde_json::to_string(&after)?;
    assert_eq!(serialized, "\"esriTimeRelationAfter\"");

    tracing::info!("test_time_relation_serialization: Testing Before serialization");
    let before = TimeRelation::Before;
    let serialized = serde_json::to_string(&before)?;
    assert_eq!(serialized, "\"esriTimeRelationBefore\"");

    tracing::info!("test_time_relation_serialization: Completed");
    Ok(())
}

#[test]
fn test_time_relation_default() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_time_relation_default: Starting");

    tracing::info!("test_time_relation_default: Checking default relation");
    let relation = TimeRelation::default();
    
    tracing::info!(
        relation = ?relation,
        "test_time_relation_default: Verifying default"
    );
    assert_eq!(relation, TimeRelation::Overlaps);

    tracing::info!("test_time_relation_default: Completed");
    Ok(())
}

#[test]
fn test_layer_selection_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_layer_selection_serialization: Starting");

    tracing::info!("test_layer_selection_serialization: Testing Top serialization");
    let top = LayerSelection::Top;
    let serialized = serde_json::to_string(&top)?;
    assert_eq!(serialized, "\"top\"");

    tracing::info!("test_layer_selection_serialization: Testing Visible serialization");
    let visible = LayerSelection::Visible;
    let serialized = serde_json::to_string(&visible)?;
    assert_eq!(serialized, "\"visible\"");

    tracing::info!("test_layer_selection_serialization: Testing All serialization");
    let all = LayerSelection::All;
    let serialized = serde_json::to_string(&all)?;
    assert_eq!(serialized, "\"all\"");

    tracing::info!("test_layer_selection_serialization: Completed");
    Ok(())
}

#[test]
fn test_layer_selection_default() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_layer_selection_default: Starting");

    tracing::info!("test_layer_selection_default: Checking default selection");
    let selection = LayerSelection::default();
    
    tracing::info!(
        selection = ?selection,
        "test_layer_selection_default: Verifying default"
    );
    assert_eq!(selection, LayerSelection::Visible);

    tracing::info!("test_layer_selection_default: Completed");
    Ok(())
}

#[test]
fn test_export_map_params_builder() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_export_map_params_builder: Starting");

    tracing::info!("test_export_map_params_builder: Building ExportMapParams");
    let params = ExportMapParams::builder()
        .bbox("-118.0,34.0,-117.0,35.0")
        .size("800,600")
        .dpi(96)
        .format(ImageFormat::Png32)
        .transparent(true)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!(
        bbox = %params.bbox(),
        size = ?params.size(),
        dpi = ?params.dpi(),
        format = ?params.format(),
        transparent = ?params.transparent(),
        "test_export_map_params_builder: Verifying params"
    );
    assert_eq!(*params.bbox(), "-118.0,34.0,-117.0,35.0");
    assert_eq!(*params.size(), Some("800,600".to_string()));
    assert_eq!(*params.dpi(), Some(96));
    assert_eq!(*params.format(), Some(ImageFormat::Png32));
    assert_eq!(*params.transparent(), Some(true));

    tracing::info!("test_export_map_params_builder: Completed");
    Ok(())
}

#[test]
fn test_export_map_params_default() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_export_map_params_default: Starting");

    tracing::info!("test_export_map_params_default: Creating default params");
    let params = ExportMapParams::default();
    
    tracing::info!(
        bbox = %params.bbox(),
        size = ?params.size(),
        dpi = ?params.dpi(),
        format = ?params.format(),
        transparent = ?params.transparent(),
        "test_export_map_params_default: Verifying defaults"
    );
    assert_eq!(*params.bbox(), "");
    assert_eq!(*params.size(), Some("400,400".to_string()));
    assert_eq!(*params.dpi(), Some(96));
    assert_eq!(*params.format(), Some(ImageFormat::Png));
    assert_eq!(*params.transparent(), Some(false));

    tracing::info!("test_export_map_params_default: Completed");
    Ok(())
}

#[test]
fn test_identify_params_builder() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_identify_params_builder: Starting");

    tracing::info!("test_identify_params_builder: Building IdentifyParams");
    let params = IdentifyParams::builder()
        .geometry("{\"x\":-118.0,\"y\":34.0}")
        .geometry_type(GeometryType::Point)
        .tolerance(5)
        .map_extent("-120,32,-116,36")
        .image_display("800,600,96")
        .layers(LayerSelection::Visible)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!(
        geometry = %params.geometry(),
        geometry_type = ?params.geometry_type(),
        tolerance = params.tolerance(),
        map_extent = %params.map_extent(),
        image_display = %params.image_display(),
        layers = ?params.layers(),
        "test_identify_params_builder: Verifying params"
    );
    assert_eq!(params.geometry(), "{\"x\":-118.0,\"y\":34.0}");
    assert_eq!(*params.geometry_type(), GeometryType::Point);
    assert_eq!(*params.tolerance(), 5);
    assert_eq!(params.map_extent(), "-120,32,-116,36");
    assert_eq!(params.image_display(), "800,600,96");
    assert_eq!(*params.layers(), Some(LayerSelection::Visible));

    tracing::info!("test_identify_params_builder: Completed");
    Ok(())
}

#[test]
fn test_tile_coordinate_creation() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_tile_coordinate_creation: Starting");

    tracing::info!("test_tile_coordinate_creation: Creating TileCoordinate");
    let coord = TileCoordinate::new(5, 10, 15);
    
    tracing::info!(
        level = coord.level(),
        row = coord.row(),
        col = coord.col(),
        "test_tile_coordinate_creation: Verifying coordinate"
    );
    assert_eq!(*coord.level(), 5);
    assert_eq!(*coord.row(), 10);
    assert_eq!(*coord.col(), 15);

    tracing::info!("test_tile_coordinate_creation: Completed");
    Ok(())
}

#[test]
fn test_export_target_helpers() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_export_target_helpers: Starting");

    tracing::info!("test_export_target_helpers: Testing Path target");
    let path_target = ExportTarget::to_path("/tmp/map.png");
    match path_target {
        ExportTarget::Path(p) => {
            tracing::info!(path = %p.display(), "test_export_target_helpers: Path target created");
            assert_eq!(p.to_str().unwrap(), "/tmp/map.png");
        }
        _ => anyhow::bail!("Expected Path variant"),
    }

    tracing::info!("test_export_target_helpers: Testing Bytes target");
    let bytes_target = ExportTarget::to_bytes();
    match bytes_target {
        ExportTarget::Bytes => {
            tracing::info!("test_export_target_helpers: Bytes target created");
        }
        _ => anyhow::bail!("Expected Bytes variant"),
    }

    tracing::info!("test_export_target_helpers: Completed");
    Ok(())
}

#[test]
fn test_export_result_helpers() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_export_result_helpers: Starting");

    use std::path::PathBuf;

    tracing::info!("test_export_result_helpers: Testing Path result");
    let path_result = ExportResult::Path(PathBuf::from("/tmp/map.png"));
    tracing::info!(
        path = ?path_result.path(),
        "test_export_result_helpers: Verifying Path result"
    );
    assert_eq!(
        path_result.path().unwrap().to_str().unwrap(),
        "/tmp/map.png"
    );
    assert!(path_result.bytes().is_none());
    assert!(path_result.written().is_none());

    tracing::info!("test_export_result_helpers: Testing Bytes result");
    let bytes_result = ExportResult::Bytes(vec![1, 2, 3, 4, 5]);
    tracing::info!(
        bytes_len = ?bytes_result.bytes().map(|b| b.len()),
        "test_export_result_helpers: Verifying Bytes result"
    );
    assert!(bytes_result.path().is_none());
    assert_eq!(bytes_result.bytes().unwrap().len(), 5);
    assert!(bytes_result.written().is_none());

    tracing::info!("test_export_result_helpers: Testing Written result");
    let written_result = ExportResult::Written(1024);
    tracing::info!(
        bytes_written = ?written_result.written(),
        "test_export_result_helpers: Verifying Written result"
    );
    assert!(written_result.path().is_none());
    assert!(written_result.bytes().is_none());
    assert_eq!(written_result.written().unwrap(), 1024);

    tracing::info!("test_export_result_helpers: Completed");
    Ok(())
}

#[test]
fn test_map_service_client_creation() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_map_service_client_creation: Starting");

    tracing::info!("test_map_service_client_creation: Creating API key auth");
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);

    tracing::info!("test_map_service_client_creation: Creating MapServiceClient");
    let map_service = MapServiceClient::new(
        "https://services.arcgis.com/test/arcgis/rest/services/World/MapServer",
        &client,
    );

    // Just verify it compiles and constructs correctly
    drop(map_service);

    tracing::info!("test_map_service_client_creation: Completed");
    Ok(())
}

#[test]
fn test_export_builder_fluent_api() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_export_builder_fluent_api: Starting");

    tracing::info!("test_export_builder_fluent_api: Creating test client");
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let map_service = MapServiceClient::new(
        "https://services.arcgis.com/test/arcgis/rest/services/World/MapServer",
        &client,
    );

    // Test builder pattern compiles (don't execute)
    tracing::info!("test_export_builder_fluent_api: Building export request");
    let builder = map_service
        .export()
        .bbox("-118.0,34.0,-117.0,35.0")
        .size(800, 600)
        .format(ImageFormat::Png32)
        .transparent(true)
        .dpi(96);

    // Verify builder exists
    drop(builder);

    tracing::info!("test_export_builder_fluent_api: Completed");
    Ok(())
}

#[test]
fn test_layer_visibility_formatting() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_layer_visibility_formatting: Starting");

    tracing::info!("test_layer_visibility_formatting: Creating test client");
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let map_service = MapServiceClient::new(
        "https://services.arcgis.com/test/arcgis/rest/services/World/MapServer",
        &client,
    );

    // Test layer_visibility method compiles with different operations
    tracing::info!("test_layer_visibility_formatting: Testing Show operation");
    let show_builder = map_service
        .export()
        .bbox("-118.0,34.0,-117.0,35.0")
        .layer_visibility(LayerOperation::Show, &[0, 1, 2]);

    drop(show_builder);

    tracing::info!("test_layer_visibility_formatting: Testing Hide operation");
    let hide_builder = map_service
        .export()
        .bbox("-118.0,34.0,-117.0,35.0")
        .layer_visibility(LayerOperation::Hide, &[3, 4]);

    drop(hide_builder);

    tracing::info!("test_layer_visibility_formatting: Completed");
    Ok(())
}
