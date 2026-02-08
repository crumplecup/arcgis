//! Tests for service definition retrieval (Phase 6).
//!
//! These tests verify that our types can deserialize ESRI Feature Service REST API
//! responses correctly, including handling of extra/missing fields from schema evolution.

use arcgis::{
    FieldType, GeometryTypeDefinition, LayerDefinition, ServiceDefinition, TableDefinition,
};

// ==================== ServiceDefinition deserialization ====================

/// Deserialize a realistic ESRI service root response.
///
/// The service root (`GET {serviceUrl}?f=json`) returns service metadata
/// and layer stubs (id, name, geometry type) but NOT full field definitions.
#[test]
fn test_deserialize_service_root() {
    let json = r#"{
        "currentVersion": 11.3,
        "serviceDescription": "Sample feature service for testing",
        "hasVersionedData": false,
        "supportsDisconnectedEditing": false,
        "hasStaticData": false,
        "maxRecordCount": 2000,
        "supportedQueryFormats": "JSON, geoJSON, PBF",
        "capabilities": "Create,Delete,Query,Update,Editing",
        "description": "",
        "copyrightText": "",
        "allowGeometryUpdates": true,
        "units": "esriDecimalDegrees",
        "syncEnabled": false,
        "layers": [
            {
                "id": 0,
                "name": "Points",
                "parentLayerId": -1,
                "defaultVisibility": true,
                "subLayerIds": null,
                "minScale": 0,
                "maxScale": 0,
                "type": "Feature Layer",
                "geometryType": "esriGeometryPoint"
            },
            {
                "id": 1,
                "name": "Buildings",
                "parentLayerId": -1,
                "defaultVisibility": true,
                "subLayerIds": null,
                "type": "Feature Layer",
                "geometryType": "esriGeometryPolygon"
            }
        ],
        "tables": []
    }"#;

    let def: ServiceDefinition = serde_json::from_str(json).expect("Should deserialize");

    // Service-level fields
    assert_eq!(def.max_record_count(), &Some(2000));
    assert_eq!(
        def.service_description(),
        &Some("Sample feature service for testing".to_string())
    );
    assert_eq!(
        def.capabilities(),
        &Some("Create,Delete,Query,Update,Editing".to_string())
    );

    // Layers from stubs
    assert_eq!(def.layers().len(), 2);
    assert_eq!(def.layers()[0].id(), &0u32);
    assert_eq!(def.layers()[0].name(), "Points");
    assert_eq!(
        def.layers()[0].geometry_type(),
        &GeometryTypeDefinition::Point
    );
    assert_eq!(
        def.layers()[1].geometry_type(),
        &GeometryTypeDefinition::Polygon
    );

    // Fields are empty (stubs don't include them)
    assert!(def.layers()[0].fields().is_empty());

    // Name is empty (not in ESRI service root JSON)
    assert_eq!(def.name(), "");

    // Tables list is empty
    assert!(def.tables().is_empty());
}

/// Extra/unknown fields in ESRI response are silently ignored.
///
/// ESRI adds new fields over time (schema evolution). Our types should
/// handle forward compatibility gracefully.
#[test]
fn test_deserialize_service_root_with_extra_fields() {
    let json = r#"{
        "currentVersion": 11.3,
        "serviceItemId": "abc123def456",
        "isView": false,
        "isUpdatableView": false,
        "sourceSchemaChangesAllowed": true,
        "maxRecordCount": 1000,
        "layers": [
            {
                "id": 0,
                "name": "Roads",
                "geometryType": "esriGeometryPolyline",
                "hasAttachments": false,
                "htmlPopupType": "esriServerHTMLPopupTypeAsHTMLText",
                "drawingInfo": {"renderer": {"type": "simple"}},
                "minScale": 0,
                "maxScale": 0,
                "defaultVisibility": true,
                "extent": {"xmin": -180, "ymin": -90, "xmax": 180, "ymax": 90},
                "type": "Feature Layer"
            }
        ],
        "tables": []
    }"#;

    let def: ServiceDefinition = serde_json::from_str(json).expect("Should handle extra fields");

    assert_eq!(def.max_record_count(), &Some(1000));
    assert_eq!(def.layers().len(), 1);
    assert_eq!(
        def.layers()[0].geometry_type(),
        &GeometryTypeDefinition::Polyline
    );
}

// ==================== LayerDefinition deserialization ====================

/// Deserialize a complete ESRI layer response.
///
/// The layer endpoint (`GET {serviceUrl}/{layerId}?f=json`) returns the full
/// layer definition including all fields, relationships, etc.
#[test]
fn test_deserialize_layer_full() {
    let json = r#"{
        "currentVersion": 11.3,
        "id": 0,
        "name": "Points",
        "type": "Feature Layer",
        "geometryType": "esriGeometryPoint",
        "description": "Sample points layer",
        "copyrightText": "",
        "defaultVisibility": true,
        "objectIdField": "OBJECTID",
        "globalIdField": "GlobalID",
        "displayField": "NAME",
        "fields": [
            {
                "name": "OBJECTID",
                "type": "esriFieldTypeOID",
                "alias": "OBJECTID",
                "nullable": false,
                "editable": false,
                "length": 4
            },
            {
                "name": "GlobalID",
                "type": "esriFieldTypeGlobalID",
                "alias": "GlobalID",
                "nullable": false,
                "editable": false,
                "length": 38
            },
            {
                "name": "NAME",
                "type": "esriFieldTypeString",
                "alias": "Name",
                "nullable": true,
                "editable": true,
                "length": 255
            }
        ],
        "relationships": [],
        "templates": [],
        "indexes": []
    }"#;

    let layer: LayerDefinition = serde_json::from_str(json).expect("Should deserialize");

    assert_eq!(layer.id(), &0u32);
    assert_eq!(layer.name(), "Points");
    assert_eq!(layer.geometry_type(), &GeometryTypeDefinition::Point);
    assert_eq!(layer.object_id_field(), &Some("OBJECTID".to_string()));
    assert_eq!(layer.global_id_field(), &Some("GlobalID".to_string()));
    assert_eq!(layer.display_field(), &Some("NAME".to_string()));
    assert_eq!(layer.fields().len(), 3);

    // Verify field types
    assert_eq!(layer.fields()[0].field_type(), &FieldType::Oid);
    assert_eq!(layer.fields()[1].field_type(), &FieldType::GlobalId);
    assert_eq!(layer.fields()[2].field_type(), &FieldType::String);

    // Verify field properties
    assert_eq!(layer.fields()[0].nullable(), &Some(false));
    assert_eq!(layer.fields()[0].editable(), &Some(false));
    assert_eq!(layer.fields()[2].length(), &Some(255));
}

/// Deserialize a layer stub (as returned in service root response).
///
/// Layer stubs have no `fields` array — this should default to empty Vec.
#[test]
fn test_deserialize_layer_stub() {
    let json = r#"{
        "id": 0,
        "name": "Points",
        "type": "Feature Layer",
        "geometryType": "esriGeometryPoint",
        "defaultVisibility": true,
        "parentLayerId": -1,
        "subLayerIds": null,
        "minScale": 0,
        "maxScale": 0
    }"#;

    let layer: LayerDefinition = serde_json::from_str(json).expect("Should deserialize stub");

    assert_eq!(layer.id(), &0u32);
    assert_eq!(layer.name(), "Points");
    assert_eq!(layer.geometry_type(), &GeometryTypeDefinition::Point);
    // Fields default to empty when not present
    assert!(layer.fields().is_empty());
}

// ==================== TableDefinition deserialization ====================

/// Deserialize a complete ESRI table response.
#[test]
fn test_deserialize_table_full() {
    let json = r#"{
        "currentVersion": 11.3,
        "id": 1,
        "name": "Permits",
        "type": "Table",
        "objectIdField": "OBJECTID",
        "displayField": "PERMIT_NO",
        "fields": [
            {
                "name": "OBJECTID",
                "type": "esriFieldTypeOID",
                "nullable": false,
                "editable": false,
                "length": 4
            },
            {
                "name": "PERMIT_NO",
                "type": "esriFieldTypeString",
                "nullable": true,
                "editable": true,
                "length": 50
            },
            {
                "name": "ISSUE_DATE",
                "type": "esriFieldTypeDate",
                "nullable": true,
                "editable": true,
                "length": 8
            }
        ],
        "relationships": []
    }"#;

    let table: TableDefinition = serde_json::from_str(json).expect("Should deserialize");

    assert_eq!(table.id(), &1u32);
    assert_eq!(table.name(), "Permits");
    assert_eq!(table.object_id_field(), &Some("OBJECTID".to_string()));
    assert_eq!(table.fields().len(), 3);
    assert_eq!(table.fields()[0].field_type(), &FieldType::Oid);
    assert_eq!(table.fields()[1].field_type(), &FieldType::String);
    assert_eq!(table.fields()[2].field_type(), &FieldType::Date);
}

// ==================== Round-trip ====================

/// ServiceDefinition created with builder round-trips through JSON.
///
/// Verifies that what we create can be serialized and deserialized
/// without data loss for the fields we populate.
#[test]
fn test_service_definition_round_trip() {
    use arcgis::{FieldDefinitionBuilder, LayerDefinitionBuilder, ServiceDefinitionBuilder};

    let oid = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("Valid OID field");

    let mut layer_builder = LayerDefinitionBuilder::default();
    layer_builder
        .id(0u32)
        .name("Buildings")
        .geometry_type(GeometryTypeDefinition::Polygon);
    let layer = layer_builder.add_field(oid).build().expect("Valid layer");

    let mut svc_builder = ServiceDefinitionBuilder::default();
    svc_builder
        .name("TestService")
        .max_record_count(2000i32)
        .capabilities("Create,Delete,Query,Update,Editing");
    let original = svc_builder.add_layer(layer).build().expect("Valid service");

    // Serialize to JSON
    let json = serde_json::to_string(&original).expect("Serialize");

    // Deserialize back
    let restored: ServiceDefinition = serde_json::from_str(&json).expect("Deserialize");

    assert_eq!(restored.name(), "TestService");
    assert_eq!(restored.max_record_count(), &Some(2000));
    assert_eq!(restored.layers().len(), 1);
    assert_eq!(restored.layers()[0].name(), "Buildings");
    assert_eq!(restored.layers()[0].fields().len(), 1);
    assert_eq!(restored.layers()[0].fields()[0].name(), "OBJECTID");
}
