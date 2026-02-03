//! Tests for strongly-typed service definition structures.
//!
//! Validates that our Rust types correctly serialize to and deserialize from
//! ESRI's JSON format for service definitions.

use arcgis::{
    FieldDefinition, FieldDefinitionBuilder, FieldType, GeometryTypeDefinition, LayerDefinition,
    LayerDefinitionBuilder, ServiceDefinitionBuilder, SpatialReferenceDefinition,
};

#[test]
fn test_field_type_serialization() {
    // Test that all FieldType variants serialize to correct ESRI enum strings
    let test_cases = vec![
        (FieldType::SmallInteger, "esriFieldTypeSmallInteger"),
        (FieldType::Integer, "esriFieldTypeInteger"),
        (FieldType::BigInteger, "esriFieldTypeBigInteger"),
        (FieldType::Single, "esriFieldTypeSingle"),
        (FieldType::Double, "esriFieldTypeDouble"),
        (FieldType::String, "esriFieldTypeString"),
        (FieldType::Date, "esriFieldTypeDate"),
        (FieldType::Oid, "esriFieldTypeOID"),
        (FieldType::Geometry, "esriFieldTypeGeometry"),
        (FieldType::Blob, "esriFieldTypeBlob"),
        (FieldType::Raster, "esriFieldTypeRaster"),
        (FieldType::Guid, "esriFieldTypeGUID"),
        (FieldType::GlobalId, "esriFieldTypeGlobalID"),
        (FieldType::Xml, "esriFieldTypeXML"),
        (FieldType::DateOnly, "esriFieldTypeDateOnly"),
        (FieldType::TimeOnly, "esriFieldTypeTimeOnly"),
        (FieldType::TimestampOffset, "esriFieldTypeTimestampOffset"),
    ];

    for (field_type, expected_json) in test_cases {
        let json = serde_json::to_string(&field_type).expect("Serialization failed");
        assert_eq!(
            json,
            format!("\"{}\"", expected_json),
            "FieldType::{:?} should serialize to {}",
            field_type,
            expected_json
        );

        // Test deserialization round-trip
        let deserialized: FieldType = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(
            deserialized, field_type,
            "Round-trip failed for FieldType::{:?}",
            field_type
        );
    }
}

#[test]
fn test_geometry_type_serialization() {
    // Test that all GeometryTypeDefinition variants serialize to correct ESRI enum strings
    let test_cases = vec![
        (GeometryTypeDefinition::Point, "esriGeometryPoint"),
        (GeometryTypeDefinition::Multipoint, "esriGeometryMultipoint"),
        (GeometryTypeDefinition::Polyline, "esriGeometryPolyline"),
        (GeometryTypeDefinition::Polygon, "esriGeometryPolygon"),
        (GeometryTypeDefinition::Envelope, "esriGeometryEnvelope"),
    ];

    for (geom_type, expected_json) in test_cases {
        let json = serde_json::to_string(&geom_type).expect("Serialization failed");
        assert_eq!(
            json,
            format!("\"{}\"", expected_json),
            "GeometryTypeDefinition::{:?} should serialize to {}",
            geom_type,
            expected_json
        );

        // Test deserialization round-trip
        let deserialized: GeometryTypeDefinition =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(
            deserialized, geom_type,
            "Round-trip failed for GeometryTypeDefinition::{:?}",
            geom_type
        );
    }
}

#[test]
fn test_spatial_reference_wkid_serialization() {
    let wgs84 = SpatialReferenceDefinition::Wkid { wkid: 4326 };

    let json = serde_json::to_value(&wgs84).expect("Serialization failed");
    assert_eq!(json["wkid"], 4326);
    assert!(!json.as_object().unwrap().contains_key("latestWkid"));

    // Round-trip
    let deserialized: SpatialReferenceDefinition =
        serde_json::from_value(json).expect("Deserialization failed");
    assert_eq!(deserialized, wgs84);
}

#[test]
fn test_spatial_reference_wkt_serialization() {
    let wkt_sr = SpatialReferenceDefinition::Wkt {
        wkt: "GEOGCS[\"GCS_WGS_1984\",DATUM[\"D_WGS_1984\",SPHEROID[\"WGS_1984\",6378137,298.257223563]],PRIMEM[\"Greenwich\",0],UNIT[\"Degree\",0.017453292519943295]]".to_string(),
    };

    let json = serde_json::to_value(&wkt_sr).expect("Serialization failed");
    assert!(json["wkt"].is_string());

    // Round-trip
    let deserialized: SpatialReferenceDefinition =
        serde_json::from_value(json).expect("Deserialization failed");
    assert_eq!(deserialized, wkt_sr);
}

#[test]
fn test_field_definition_builder() {
    let field = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .alias("Object ID")
        .nullable(false)
        .editable(false)
        .build()
        .expect("Builder should succeed");

    assert_eq!(field.name(), "OBJECTID");
    assert_eq!(*field.field_type(), FieldType::Oid);
    assert_eq!(field.alias().as_deref(), Some("Object ID"));
    assert_eq!(*field.nullable(), Some(false));
    assert_eq!(*field.editable(), Some(false));
}

#[test]
fn test_field_definition_serialization() {
    let field = FieldDefinitionBuilder::default()
        .name("NAME")
        .field_type(FieldType::String)
        .alias("Feature Name")
        .length(255)
        .nullable(true)
        .editable(true)
        .build()
        .expect("Builder should succeed");

    let json = serde_json::to_value(&field).expect("Serialization failed");

    assert_eq!(json["name"], "NAME");
    assert_eq!(json["type"], "esriFieldTypeString");
    assert_eq!(json["alias"], "Feature Name");
    assert_eq!(json["length"], 255);
    assert_eq!(json["nullable"], true);
    assert_eq!(json["editable"], true);

    // Round-trip
    let deserialized: FieldDefinition =
        serde_json::from_value(json).expect("Deserialization failed");
    assert_eq!(deserialized.name(), field.name());
    assert_eq!(deserialized.field_type(), field.field_type());
}

#[test]
fn test_layer_definition_builder() {
    let oid_field = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("OID field builder");

    let globalid_field = FieldDefinitionBuilder::default()
        .name("GlobalID")
        .field_type(FieldType::GlobalId)
        .nullable(false)
        .editable(false)
        .length(38)
        .build()
        .expect("GlobalID field builder");

    let name_field = FieldDefinitionBuilder::default()
        .name("NAME")
        .field_type(FieldType::String)
        .length(100)
        .build()
        .expect("Name field builder");

    let layer = LayerDefinitionBuilder::default()
        .name("Points")
        .geometry_type(GeometryTypeDefinition::Point)
        .object_id_field("OBJECTID")
        .global_id_field("GlobalID")
        .fields(vec![oid_field, globalid_field, name_field])
        .build()
        .expect("Layer builder should succeed");

    assert_eq!(layer.name(), "Points");
    assert_eq!(layer.geometry_type(), &GeometryTypeDefinition::Point);
    assert_eq!(layer.object_id_field(), &Some("OBJECTID".to_string()));
    assert_eq!(layer.global_id_field(), &Some("GlobalID".to_string()));
    assert_eq!(layer.fields().len(), 3);
}

#[test]
fn test_layer_definition_serialization() {
    let oid_field = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .build()
        .expect("OID field");

    let layer = LayerDefinitionBuilder::default()
        .name("TestLayer")
        .geometry_type(GeometryTypeDefinition::Polygon)
        .object_id_field("OBJECTID")
        .fields(vec![oid_field])
        .build()
        .expect("Layer builder");

    let json = serde_json::to_value(&layer).expect("Serialization failed");

    assert_eq!(json["name"], "TestLayer");
    assert_eq!(json["geometryType"], "esriGeometryPolygon");
    assert_eq!(json["objectIdField"], "OBJECTID");
    assert_eq!(json["fields"].as_array().unwrap().len(), 1);

    // Round-trip
    let deserialized: LayerDefinition =
        serde_json::from_value(json).expect("Deserialization failed");
    assert_eq!(deserialized.name(), layer.name());
    assert_eq!(deserialized.geometry_type(), layer.geometry_type());
}

#[test]
fn test_service_definition_builder_minimal() {
    let service = ServiceDefinitionBuilder::default()
        .name("MinimalService")
        .build()
        .expect("Minimal service should build");

    assert_eq!(service.name(), "MinimalService");
    assert!(service.layers().is_empty());
    assert!(service.tables().is_empty());
}

#[test]
fn test_service_definition_builder_with_layers() {
    let oid_field = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .build()
        .expect("OID field");

    let layer = LayerDefinitionBuilder::default()
        .name("Points")
        .geometry_type(GeometryTypeDefinition::Point)
        .object_id_field("OBJECTID")
        .fields(vec![oid_field])
        .build()
        .expect("Layer");

    let service = ServiceDefinitionBuilder::default()
        .name("FeatureService")
        .service_description("Test feature service")
        .layers(vec![layer])
        .build()
        .expect("Service with layers should build");

    assert_eq!(service.name(), "FeatureService");
    assert_eq!(
        service.service_description().as_deref(),
        Some("Test feature service")
    );
    assert_eq!(service.layers().len(), 1);
    assert_eq!(service.layers()[0].name(), "Points");
}

#[test]
fn test_service_definition_empty_collections_not_serialized() {
    // Test that empty vectors are not serialized (using skip_serializing_if)
    let service = ServiceDefinitionBuilder::default()
        .name("EmptyService")
        .build()
        .expect("Empty service");

    let json = serde_json::to_string(&service).expect("Serialization failed");

    // Should not contain "layers" or "tables" keys when empty
    assert!(
        !json.contains("\"layers\""),
        "Empty layers should be skipped"
    );
    assert!(
        !json.contains("\"tables\""),
        "Empty tables should be skipped"
    );
}

#[test]
fn test_field_definition_optional_fields_omitted() {
    // Test that None values are not serialized
    let field = FieldDefinitionBuilder::default()
        .name("SIMPLE")
        .field_type(FieldType::String)
        .build()
        .expect("Simple field");

    let json = serde_json::to_value(&field).expect("Serialization failed");

    // Should only have name and type, no optional fields
    assert_eq!(json["name"], "SIMPLE");
    assert_eq!(json["type"], "esriFieldTypeString");
    assert!(!json.as_object().unwrap().contains_key("alias"));
    assert!(!json.as_object().unwrap().contains_key("length"));
    assert!(!json.as_object().unwrap().contains_key("domain"));
}

#[test]
fn test_esri_spec_compliance_field_names() {
    // Verify that our field names match ESRI's camelCase convention exactly
    let layer = LayerDefinitionBuilder::default()
        .name("Test")
        .object_id_field("OBJECTID")
        .display_field("NAME")
        .build()
        .expect("Layer");

    let json = serde_json::to_value(&layer).expect("Serialization failed");

    // Check that Rust snake_case fields serialize to ESRI's camelCase
    assert!(json.as_object().unwrap().contains_key("objectIdField"));
    assert!(json.as_object().unwrap().contains_key("displayField"));
    assert!(!json.as_object().unwrap().contains_key("object_id_field"));
    assert!(!json.as_object().unwrap().contains_key("display_field"));
}
