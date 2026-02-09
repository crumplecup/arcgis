//! Integration tests for strongly-typed service definition usage in Portal client.
//!
//! Validates that ServiceDefinition can be used with CreateServiceParams and
//! UpdateServiceDefinitionParams, and that serialization produces valid ESRI JSON.

use arcgis::{
    CreateServiceParams, FieldDefinition, FieldDefinitionBuilder, FieldType,
    GeometryTypeDefinition, LayerDefinitionBuilder, ServiceDefinitionBuilder,
    UpdateServiceDefinitionParams,
};

#[test]
fn test_create_service_params_with_service_definition() {
    // Build a service definition
    let oid_field = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("Valid OID field");

    let name_field = FieldDefinitionBuilder::default()
        .name("NAME")
        .field_type(FieldType::String)
        .length(255)
        .build()
        .expect("Valid name field");

    let layer = LayerDefinitionBuilder::default()
        .name("TestLayer")
        .geometry_type(GeometryTypeDefinition::Point)
        .object_id_field("OBJECTID")
        .fields(vec![oid_field, name_field])
        .build()
        .expect("Valid layer");

    let service_def = ServiceDefinitionBuilder::default()
        .name("TestService")
        .layers(vec![layer])
        .build()
        .expect("Valid service definition");

    // Use with CreateServiceParams
    let params = CreateServiceParams::new("TestService")
        .with_description("Test service with strongly-typed definition")
        .with_capabilities("Query,Create,Update,Delete,Editing")
        .with_service_definition(service_def);

    // Verify params were created successfully
    assert_eq!(params.name(), "TestService");
    assert_eq!(
        params.description().as_deref(),
        Some("Test service with strongly-typed definition")
    );
    assert!(params.service_definition().is_some());

    // Verify the service definition can be accessed
    let retrieved_def = params
        .service_definition()
        .as_ref()
        .expect("Should have definition");
    assert_eq!(retrieved_def.name(), "TestService");
    assert_eq!(retrieved_def.layers().len(), 1);
    assert_eq!(retrieved_def.layers()[0].name(), "TestLayer");
}

#[test]
fn test_update_service_definition_params_with_service_definition() {
    // Build a minimal service definition for update
    let layer = LayerDefinitionBuilder::default()
        .name("UpdatedLayer")
        .geometry_type(GeometryTypeDefinition::Polyline)
        .build()
        .expect("Valid layer");

    let service_def = ServiceDefinitionBuilder::default()
        .name("UpdatedService")
        .layers(vec![layer])
        .build()
        .expect("Valid service definition");

    // Use with UpdateServiceDefinitionParams
    let params = UpdateServiceDefinitionParams::new()
        .with_service_definition(service_def)
        .with_description("Updated description")
        .with_max_record_count(2000);

    // Verify params were created successfully
    assert!(params.service_definition().is_some());
    assert_eq!(params.description().as_deref(), Some("Updated description"));
    assert_eq!(params.max_record_count(), &Some(2000));

    // Verify the service definition can be accessed
    let retrieved_def = params
        .service_definition()
        .as_ref()
        .expect("Should have definition");
    assert_eq!(retrieved_def.name(), "UpdatedService");
    assert_eq!(retrieved_def.layers().len(), 1);
}

#[test]
fn test_service_definition_serialization_for_api() {
    // Build a complete service definition with branch versioning requirements
    let oid_field = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .alias("Object ID")
        .nullable(false)
        .editable(false)
        .build()
        .expect("Valid OID field");

    let globalid_field = FieldDefinitionBuilder::default()
        .name("GlobalID")
        .field_type(FieldType::GlobalId)
        .alias("Global ID")
        .nullable(false)
        .editable(false)
        .length(38)
        .build()
        .expect("Valid GlobalID field");

    let layer = LayerDefinitionBuilder::default()
        .name("VersionedLayer")
        .geometry_type(GeometryTypeDefinition::Point)
        .object_id_field("OBJECTID")
        .global_id_field("GlobalID")
        .fields(vec![oid_field, globalid_field])
        .build()
        .expect("Valid layer");

    let service_def = ServiceDefinitionBuilder::default()
        .name("VersionedService")
        .service_description("Branch versioned service")
        .layers(vec![layer])
        .allow_geometry_updates(true)
        .build()
        .expect("Valid service definition");

    // Serialize to JSON (simulating what Portal client does)
    let json = serde_json::to_value(&service_def).expect("Should serialize");

    // Verify JSON structure matches ESRI expectations
    assert_eq!(json["name"], "VersionedService");
    assert_eq!(json["serviceDescription"], "Branch versioned service");
    assert_eq!(json["allowGeometryUpdates"], true);

    // Verify layers array
    assert!(json["layers"].is_array());
    let layers = json["layers"].as_array().expect("Should be array");
    assert_eq!(layers.len(), 1);

    // Verify layer structure
    assert_eq!(layers[0]["name"], "VersionedLayer");
    assert_eq!(layers[0]["geometryType"], "esriGeometryPoint");
    assert_eq!(layers[0]["objectIdField"], "OBJECTID");
    assert_eq!(layers[0]["globalIdField"], "GlobalID");

    // Verify fields array
    let fields = layers[0]["fields"].as_array().expect("Should have fields");
    assert_eq!(fields.len(), 2);

    // Verify OBJECTID field
    assert_eq!(fields[0]["name"], "OBJECTID");
    assert_eq!(fields[0]["type"], "esriFieldTypeOID");
    assert_eq!(fields[0]["nullable"], false);
    assert_eq!(fields[0]["editable"], false);

    // Verify GlobalID field
    assert_eq!(fields[1]["name"], "GlobalID");
    assert_eq!(fields[1]["type"], "esriFieldTypeGlobalID");
}

#[test]
fn test_create_service_params_without_definition() {
    // Verify backward compatibility - can create params without service definition
    let params = CreateServiceParams::new("SimpleService")
        .with_description("A simple service")
        .with_capabilities("Query");

    assert_eq!(params.name(), "SimpleService");
    assert!(params.service_definition().is_none());
}

#[test]
fn test_roundtrip_service_definition_through_params() {
    // Create a service definition
    let field: FieldDefinition = FieldDefinitionBuilder::default()
        .name("ID")
        .field_type(FieldType::Integer)
        .build()
        .expect("Valid field");

    let original_layer = LayerDefinitionBuilder::default()
        .name("OriginalLayer")
        .geometry_type(GeometryTypeDefinition::Polygon)
        .fields(vec![field])
        .build()
        .expect("Valid layer");

    let original_def = ServiceDefinitionBuilder::default()
        .name("OriginalService")
        .layers(vec![original_layer])
        .build()
        .expect("Valid service definition");

    // Store in CreateServiceParams
    let params =
        CreateServiceParams::new("OriginalService").with_service_definition(original_def.clone());

    // Retrieve and verify it's the same
    let retrieved_def = params
        .service_definition()
        .as_ref()
        .expect("Should have definition");

    // Serialize both and compare
    let original_json = serde_json::to_value(&original_def).expect("Should serialize");
    let retrieved_json = serde_json::to_value(retrieved_def).expect("Should serialize");

    assert_eq!(original_json, retrieved_json);
}
