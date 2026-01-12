//! Tests for Map Service types and client.

use arcgis::{
    ApiKeyAuth, ArcGISClient, ExportMapParams, ExportResult, ExportTarget, GeometryType,
    IdentifyParams, ImageFormat, LayerOperation, LayerSelection, MapServiceClient, Result,
    TileCoordinate, TimeRelation,
};

#[test]
fn test_image_format_serialization() -> Result<()> {
    let png = ImageFormat::Png;
    let serialized = serde_json::to_string(&png)?;
    assert_eq!(serialized, "\"png\"");

    let png32 = ImageFormat::Png32;
    let serialized = serde_json::to_string(&png32)?;
    assert_eq!(serialized, "\"png32\"");

    let jpg = ImageFormat::Jpg;
    let serialized = serde_json::to_string(&jpg)?;
    assert_eq!(serialized, "\"jpg\"");

    let pdf = ImageFormat::Pdf;
    let serialized = serde_json::to_string(&pdf)?;
    assert_eq!(serialized, "\"pdf\"");

    Ok(())
}

#[test]
fn test_image_format_default() {
    let format = ImageFormat::default();
    assert_eq!(format, ImageFormat::Png);
}

#[test]
fn test_layer_operation_as_str() {
    assert_eq!(LayerOperation::Show.as_str(), "show");
    assert_eq!(LayerOperation::Hide.as_str(), "hide");
    assert_eq!(LayerOperation::Include.as_str(), "include");
    assert_eq!(LayerOperation::Exclude.as_str(), "exclude");
}

#[test]
fn test_time_relation_serialization() -> Result<()> {
    let overlaps = TimeRelation::Overlaps;
    let serialized = serde_json::to_string(&overlaps)?;
    assert_eq!(serialized, "\"esriTimeRelationOverlaps\"");

    let after = TimeRelation::After;
    let serialized = serde_json::to_string(&after)?;
    assert_eq!(serialized, "\"esriTimeRelationAfter\"");

    let before = TimeRelation::Before;
    let serialized = serde_json::to_string(&before)?;
    assert_eq!(serialized, "\"esriTimeRelationBefore\"");

    Ok(())
}

#[test]
fn test_time_relation_default() {
    let relation = TimeRelation::default();
    assert_eq!(relation, TimeRelation::Overlaps);
}

#[test]
fn test_layer_selection_serialization() -> Result<()> {
    let top = LayerSelection::Top;
    let serialized = serde_json::to_string(&top)?;
    assert_eq!(serialized, "\"top\"");

    let visible = LayerSelection::Visible;
    let serialized = serde_json::to_string(&visible)?;
    assert_eq!(serialized, "\"visible\"");

    let all = LayerSelection::All;
    let serialized = serde_json::to_string(&all)?;
    assert_eq!(serialized, "\"all\"");

    Ok(())
}

#[test]
fn test_layer_selection_default() {
    let selection = LayerSelection::default();
    assert_eq!(selection, LayerSelection::Visible);
}

#[test]
fn test_export_map_params_builder() -> Result<()> {
    let params = ExportMapParams::builder()
        .bbox("-118.0,34.0,-117.0,35.0")
        .size("800,600")
        .dpi(96)
        .format(ImageFormat::Png32)
        .transparent(true)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    assert_eq!(*params.bbox(), "-118.0,34.0,-117.0,35.0");
    assert_eq!(*params.size(), Some("800,600".to_string()));
    assert_eq!(*params.dpi(), Some(96));
    assert_eq!(*params.format(), Some(ImageFormat::Png32));
    assert_eq!(*params.transparent(), Some(true));

    Ok(())
}

#[test]
fn test_export_map_params_default() {
    let params = ExportMapParams::default();
    assert_eq!(*params.bbox(), "");
    assert_eq!(*params.size(), Some("400,400".to_string()));
    assert_eq!(*params.dpi(), Some(96));
    assert_eq!(*params.format(), Some(ImageFormat::Png));
    assert_eq!(*params.transparent(), Some(false));
}

#[test]
fn test_identify_params_builder() -> Result<()> {
    let params = IdentifyParams::builder()
        .geometry("{\"x\":-118.0,\"y\":34.0}")
        .geometry_type(GeometryType::Point)
        .tolerance(5)
        .map_extent("-120,32,-116,36")
        .image_display("800,600,96")
        .layers(LayerSelection::Visible)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    assert_eq!(params.geometry(), "{\"x\":-118.0,\"y\":34.0}");
    assert_eq!(*params.geometry_type(), GeometryType::Point);
    assert_eq!(*params.tolerance(), 5);
    assert_eq!(params.map_extent(), "-120,32,-116,36");
    assert_eq!(params.image_display(), "800,600,96");
    assert_eq!(*params.layers(), Some(LayerSelection::Visible));

    Ok(())
}

#[test]
fn test_tile_coordinate_creation() {
    let coord = TileCoordinate::new(5, 10, 15);
    assert_eq!(*coord.level(), 5);
    assert_eq!(*coord.row(), 10);
    assert_eq!(*coord.col(), 15);
}

#[test]
fn test_export_target_helpers() {
    let path_target = ExportTarget::to_path("/tmp/map.png");
    match path_target {
        ExportTarget::Path(p) => assert_eq!(p.to_str().unwrap(), "/tmp/map.png"),
        _ => panic!("Expected Path variant"),
    }

    let bytes_target = ExportTarget::to_bytes();
    match bytes_target {
        ExportTarget::Bytes => {}
        _ => panic!("Expected Bytes variant"),
    }
}

#[test]
fn test_export_result_helpers() {
    use std::path::PathBuf;

    let path_result = ExportResult::Path(PathBuf::from("/tmp/map.png"));
    assert_eq!(
        path_result.path().unwrap().to_str().unwrap(),
        "/tmp/map.png"
    );
    assert!(path_result.bytes().is_none());
    assert!(path_result.written().is_none());

    let bytes_result = ExportResult::Bytes(vec![1, 2, 3, 4, 5]);
    assert!(bytes_result.path().is_none());
    assert_eq!(bytes_result.bytes().unwrap().len(), 5);
    assert!(bytes_result.written().is_none());

    let written_result = ExportResult::Written(1024);
    assert!(written_result.path().is_none());
    assert!(written_result.bytes().is_none());
    assert_eq!(written_result.written().unwrap(), 1024);
}

#[test]
fn test_map_service_client_creation() {
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);

    let map_service = MapServiceClient::new(
        "https://services.arcgis.com/test/arcgis/rest/services/World/MapServer",
        &client,
    );

    // Just verify it compiles and constructs correctly
    drop(map_service);
}

#[test]
fn test_export_builder_fluent_api() {
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let map_service = MapServiceClient::new(
        "https://services.arcgis.com/test/arcgis/rest/services/World/MapServer",
        &client,
    );

    // Test builder pattern compiles (don't execute)
    let builder = map_service
        .export()
        .bbox("-118.0,34.0,-117.0,35.0")
        .size(800, 600)
        .format(ImageFormat::Png32)
        .transparent(true)
        .dpi(96);

    // Verify builder exists
    drop(builder);
}

#[test]
fn test_layer_visibility_formatting() {
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let map_service = MapServiceClient::new(
        "https://services.arcgis.com/test/arcgis/rest/services/World/MapServer",
        &client,
    );

    // Test layer_visibility method compiles with different operations
    let show_builder = map_service
        .export()
        .bbox("-118.0,34.0,-117.0,35.0")
        .layer_visibility(LayerOperation::Show, &[0, 1, 2]);

    drop(show_builder);

    let hide_builder = map_service
        .export()
        .bbox("-118.0,34.0,-117.0,35.0")
        .layer_visibility(LayerOperation::Hide, &[3, 4]);

    drop(hide_builder);
}
