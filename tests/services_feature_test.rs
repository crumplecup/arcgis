//! Tests for Feature Service types and client.

mod common;

use arcgis::ApiKeyAuth;
use arcgis::{
    ArcGISClient, Feature, FeatureQueryParams, FeatureServiceClient, FeatureSet, GeometryType,
    ResponseFormat,
};
use std::collections::HashMap;

#[test]
fn test_response_format_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_response_format_serialization: Starting");

    tracing::info!("test_response_format_serialization: Testing Json format");
    let json_format = ResponseFormat::Json;
    let serialized = serde_json::to_string(&json_format)?;
    assert_eq!(serialized, "\"json\"");

    tracing::info!("test_response_format_serialization: Testing GeoJson format");
    let geojson_format = ResponseFormat::GeoJson;
    let serialized = serde_json::to_string(&geojson_format)?;
    assert_eq!(serialized, "\"geojson\"");

    tracing::info!("test_response_format_serialization: Testing Pbf format");
    let pbf_format = ResponseFormat::Pbf;
    let serialized = serde_json::to_string(&pbf_format)?;
    assert_eq!(serialized, "\"pbf\"");
    
    tracing::info!("test_response_format_serialization: Completed");
    Ok(())
}

#[test]
fn test_feature_query_params_builder() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_feature_query_params_builder: Starting");

    tracing::info!("test_feature_query_params_builder: Building FeatureQueryParams");
    let params = FeatureQueryParams::builder()
        .where_clause("POPULATION > 100000")
        .out_fields(vec!["NAME".to_string(), "POPULATION".to_string()])
        .return_geometry(true)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!(
        where_clause = %params.where_clause(),
        out_fields_count = ?params.out_fields().as_ref().map(|f| f.len()),
        return_geometry = params.return_geometry(),
        "test_feature_query_params_builder: Verifying params"
    );
    assert_eq!(params.where_clause(), "POPULATION > 100000");
    assert_eq!(params.out_fields().as_ref().map(|f| f.len()), Some(2));
    assert!(params.return_geometry());
    
    tracing::info!("test_feature_query_params_builder: Completed");
    Ok(())
}

#[test]
fn test_feature_query_params_default() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_feature_query_params_default: Starting");

    tracing::info!("test_feature_query_params_default: Creating default params");
    let params = FeatureQueryParams::default();
    
    tracing::info!(
        where_clause = %params.where_clause(),
        return_geometry = params.return_geometry(),
        format = ?params.format(),
        "test_feature_query_params_default: Verifying defaults"
    );
    assert_eq!(params.where_clause(), "1=1");
    assert!(params.return_geometry());
    assert_eq!(*params.format(), ResponseFormat::Json);
    
    tracing::info!("test_feature_query_params_default: Completed");
    Ok(())
}

#[test]
fn test_feature_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_feature_serialization: Starting");

    tracing::info!("test_feature_serialization: Creating feature attributes");
    let mut attributes = HashMap::new();
    attributes.insert("NAME".to_string(), serde_json::json!("Test City"));
    attributes.insert("POPULATION".to_string(), serde_json::json!(100000));

    tracing::info!("test_feature_serialization: Creating feature");
    let feature = Feature::new(attributes, None);

    tracing::info!("test_feature_serialization: Serializing to JSON");
    let json = serde_json::to_string(&feature)?;
    
    tracing::info!(
        json_len = json.len(),
        "test_feature_serialization: Verifying JSON content"
    );
    assert!(json.contains("NAME"));
    assert!(json.contains("Test City"));
    
    tracing::info!("test_feature_serialization: Completed");
    Ok(())
}

#[test]
fn test_feature_set_deserialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_feature_set_deserialization: Starting");

    let json = r#"{
        "geometryType": "esriGeometryPoint",
        "features": [
            {
                "attributes": {
                    "OBJECTID": 1,
                    "NAME": "Test"
                }
            }
        ],
        "exceededTransferLimit": false
    }"#;

    tracing::info!("test_feature_set_deserialization: Deserializing FeatureSet");
    let feature_set: FeatureSet = serde_json::from_str(json)?;
    
    tracing::info!(
        geometry_type = ?feature_set.geometry_type(),
        features_count = feature_set.features().len(),
        exceeded_transfer_limit = feature_set.exceeded_transfer_limit(),
        "test_feature_set_deserialization: Verifying feature set"
    );
    assert_eq!(*feature_set.geometry_type(), Some(GeometryType::Point));
    assert_eq!(feature_set.features().len(), 1);
    assert!(!feature_set.exceeded_transfer_limit());
    
    tracing::info!("test_feature_set_deserialization: Completed");
    Ok(())
}

#[test]
fn test_feature_service_client_creation() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_feature_service_client_creation: Starting");

    tracing::info!("test_feature_service_client_creation: Creating API key auth");
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);

    tracing::info!("test_feature_service_client_creation: Creating FeatureServiceClient");
    let feature_service =
        FeatureServiceClient::new("https://services.arcgis.com/test/FeatureServer", &client);

    // Just verify it compiles and constructs correctly
    drop(feature_service);
    
    tracing::info!("test_feature_service_client_creation: Completed");
    Ok(())
}
