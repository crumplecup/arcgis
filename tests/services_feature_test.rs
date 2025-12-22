//! Tests for Feature Service types and client.

use arcgis::{
    ArcGISClient, Feature, FeatureQueryParams, FeatureServiceClient, FeatureSet, GeometryType,
    ResponseFormat,
};
use arcgis::ApiKeyAuth;
use std::collections::HashMap;

#[test]
fn test_response_format_serialization() {
    let json_format = ResponseFormat::Json;
    let serialized = serde_json::to_string(&json_format).unwrap();
    assert_eq!(serialized, "\"json\"");

    let geojson_format = ResponseFormat::GeoJson;
    let serialized = serde_json::to_string(&geojson_format).unwrap();
    assert_eq!(serialized, "\"geojson\"");

    let pbf_format = ResponseFormat::Pbf;
    let serialized = serde_json::to_string(&pbf_format).unwrap();
    assert_eq!(serialized, "\"pbf\"");
}

#[test]
fn test_feature_query_params_builder() {
    let params = FeatureQueryParams::builder()
        .where_clause("POPULATION > 100000")
        .out_fields(vec!["NAME".to_string(), "POPULATION".to_string()])
        .return_geometry(true)
        .build()
        .unwrap();

    assert_eq!(params.where_clause, "POPULATION > 100000");
    assert_eq!(params.out_fields.as_ref().unwrap().len(), 2);
    assert!(params.return_geometry);
}

#[test]
fn test_feature_query_params_default() {
    let params = FeatureQueryParams::default();
    assert_eq!(params.where_clause, "1=1");
    assert!(params.return_geometry);
    assert_eq!(params.format, ResponseFormat::Json);
}

#[test]
fn test_feature_serialization() {
    let mut attributes = HashMap::new();
    attributes.insert("NAME".to_string(), serde_json::json!("Test City"));
    attributes.insert("POPULATION".to_string(), serde_json::json!(100000));

    let feature = Feature {
        attributes,
        geometry: None,
    };

    let json = serde_json::to_string(&feature).unwrap();
    assert!(json.contains("NAME"));
    assert!(json.contains("Test City"));
}

#[test]
fn test_feature_set_deserialization() {
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

    let feature_set: FeatureSet = serde_json::from_str(json).unwrap();
    assert_eq!(feature_set.geometry_type, Some(GeometryType::Point));
    assert_eq!(feature_set.features.len(), 1);
    assert!(!feature_set.exceeded_transfer_limit);
}

#[test]
fn test_feature_service_client_creation() {
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);

    let feature_service = FeatureServiceClient::new(
        "https://services.arcgis.com/test/FeatureServer",
        &client,
    );

    // Just verify it compiles and constructs correctly
    drop(feature_service);
}
