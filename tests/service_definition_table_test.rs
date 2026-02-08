//! Tests for TableDefinition types in service definitions.
//!
//! These tests validate that our types match ESRI's real-world JSON responses
//! from production Feature Services (non-spatial tables).

use arcgis::{
    EditFieldsInfoBuilder, FieldDefinitionBuilder, FieldType, IndexBuilder,
    LayerRelationshipBuilder, RelationshipCardinality, RelationshipRole, TableDefinition,
    TableDefinitionBuilder,
};

/// Test basic TableDefinition deserialization with real ESRI JSON.
///
/// Source: ESRI Feature Service table definition spec
/// <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
#[test]
fn test_table_definition_minimal() {
    let esri_json = r#"{
        "id": 1,
        "name": "Permits"
    }"#;

    let table: TableDefinition =
        serde_json::from_str(esri_json).expect("Should deserialize minimal table definition");

    assert_eq!(*table.id(), 1u32);
    assert_eq!(table.name(), "Permits");
    assert!(table.fields().is_empty());
    assert!(table.table_type().is_none());

    // Round-trip
    let json_output = serde_json::to_string(&table).expect("Should serialize");
    let roundtrip: TableDefinition = serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(*roundtrip.id(), 1u32);
    assert_eq!(roundtrip.name(), "Permits");
}

/// Test TableDefinition with full set of fields.
#[test]
fn test_table_definition_with_fields() {
    let esri_json = r#"{
        "id": 2,
        "name": "Inspections",
        "type": "Table",
        "objectIdField": "OBJECTID",
        "globalIdField": "GlobalID",
        "displayField": "InspectionDate",
        "description": "Building inspection records",
        "copyrightText": "City of Springfield",
        "fields": [
            {
                "name": "OBJECTID",
                "type": "esriFieldTypeOID",
                "nullable": false,
                "editable": false
            },
            {
                "name": "GlobalID",
                "type": "esriFieldTypeGlobalID",
                "nullable": false,
                "editable": false
            },
            {
                "name": "InspectionDate",
                "type": "esriFieldTypeDate",
                "nullable": true,
                "editable": true
            }
        ],
        "isDataBranchVersioned": true
    }"#;

    let table: TableDefinition =
        serde_json::from_str(esri_json).expect("Should deserialize full table definition");

    assert_eq!(*table.id(), 2u32);
    assert_eq!(table.name(), "Inspections");
    assert_eq!(table.table_type(), &Some("Table".to_string()));
    assert_eq!(table.object_id_field(), &Some("OBJECTID".to_string()));
    assert_eq!(table.global_id_field(), &Some("GlobalID".to_string()));
    assert_eq!(table.display_field(), &Some("InspectionDate".to_string()));
    assert_eq!(
        table.description(),
        &Some("Building inspection records".to_string())
    );
    assert_eq!(
        table.copyright_text(),
        &Some("City of Springfield".to_string())
    );
    assert_eq!(table.fields().len(), 3);
    assert_eq!(table.fields()[0].name(), "OBJECTID");
    assert_eq!(*table.fields()[0].field_type(), FieldType::Oid);
    assert_eq!(table.is_data_branch_versioned(), &Some(true));

    // Round-trip
    let json_output = serde_json::to_string(&table).expect("Should serialize");
    let roundtrip: TableDefinition = serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(*roundtrip.id(), 2u32);
    assert_eq!(roundtrip.fields().len(), 3);
}

/// Test TableDefinition with relationships.
///
/// Tables participate in relationships just like layers.
#[test]
fn test_table_definition_with_relationships() {
    let esri_json = r#"{
        "id": 3,
        "name": "Permits",
        "relationships": [
            {
                "id": 0,
                "name": "Buildings_Permits",
                "role": "esriRelRoleDestination",
                "keyField": "BuildingGUID",
                "cardinality": "esriRelCardinalityManyToMany",
                "relatedTableId": 0
            }
        ]
    }"#;

    let table: TableDefinition =
        serde_json::from_str(esri_json).expect("Should deserialize table with relationships");

    assert_eq!(table.relationships().len(), 1);
    assert_eq!(*table.relationships()[0].id(), 0);
    assert_eq!(
        *table.relationships()[0].role(),
        RelationshipRole::Destination
    );
    assert_eq!(
        *table.relationships()[0].cardinality(),
        RelationshipCardinality::ManyToMany
    );

    // Round-trip
    let json_output = serde_json::to_string(&table).expect("Should serialize");
    let roundtrip: TableDefinition = serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(roundtrip.relationships().len(), 1);
}

/// Test that TableDefinition does NOT have geometry-related fields.
///
/// Tables should serialize without geometryType or defaultVisibility.
#[test]
fn test_table_definition_no_geometry_fields() {
    let table = TableDefinitionBuilder::default()
        .id(1u32)
        .name("NonSpatialTable")
        .build()
        .expect("Should build table");

    let json_value = serde_json::to_value(&table).expect("Should serialize");

    // Tables must NOT have geometry-related fields
    assert!(
        json_value.get("geometryType").is_none(),
        "Tables must not have geometryType"
    );
    assert!(
        json_value.get("defaultVisibility").is_none(),
        "Tables must not have defaultVisibility"
    );
}

/// Test builder pattern for TableDefinition.
#[test]
fn test_table_definition_builder() {
    let oid_field = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("Should build OID field");

    let global_id_field = FieldDefinitionBuilder::default()
        .name("GlobalID")
        .field_type(FieldType::GlobalId)
        .nullable(false)
        .editable(false)
        .build()
        .expect("Should build GlobalID field");

    let table = TableDefinitionBuilder::default()
        .id(1u32)
        .name("Permits")
        .table_type("Table")
        .object_id_field("OBJECTID")
        .global_id_field("GlobalID")
        .display_field("PermitNumber")
        .description("Building permits")
        .is_data_branch_versioned(true)
        .build()
        .expect("Should build table");

    assert_eq!(*table.id(), 1u32);
    assert_eq!(table.name(), "Permits");
    assert_eq!(table.table_type(), &Some("Table".to_string()));
    assert_eq!(table.object_id_field(), &Some("OBJECTID".to_string()));
    assert_eq!(table.global_id_field(), &Some("GlobalID".to_string()));
    assert_eq!(table.display_field(), &Some("PermitNumber".to_string()));
    assert_eq!(table.description(), &Some("Building permits".to_string()));
    assert_eq!(table.is_data_branch_versioned(), &Some(true));

    // Test add_* builder methods
    let mut builder = TableDefinitionBuilder::default();
    builder.id(2u32).name("Inspections");
    let table_with_fields = builder
        .add_field(oid_field)
        .add_field(global_id_field)
        .build()
        .expect("Should build table with fields");

    assert_eq!(table_with_fields.fields().len(), 2);
}

/// Test TableDefinitionBuilder add_* methods for all collection fields.
#[test]
fn test_table_builder_add_methods() {
    let field = FieldDefinitionBuilder::default()
        .name("Status")
        .field_type(FieldType::Integer)
        .build()
        .expect("Valid field");

    let index = IndexBuilder::default()
        .name("status_idx")
        .fields(vec!["Status".to_string()])
        .is_unique(false)
        .build()
        .expect("Valid index");

    let edit_fields = EditFieldsInfoBuilder::default()
        .creation_date_field("CreatedDate")
        .creator_field("CreatedBy")
        .build()
        .expect("Valid edit fields");

    let relationship = LayerRelationshipBuilder::default()
        .id(0_i32)
        .role(RelationshipRole::Origin)
        .cardinality(RelationshipCardinality::OneToMany)
        .related_table_id(1_i32)
        .key_field("GlobalID")
        .build()
        .expect("Valid relationship");

    let mut builder = TableDefinitionBuilder::default();
    builder
        .id(1u32)
        .name("Permits")
        .edit_fields_info(edit_fields);
    let table = builder
        .add_field(field)
        .add_index(index)
        .add_relationship(relationship)
        .build()
        .expect("Should build table with all collections");

    assert_eq!(table.fields().len(), 1);
    assert_eq!(table.indexes().len(), 1);
    assert_eq!(table.relationships().len(), 1);
    assert!(table.edit_fields_info().is_some());
}

/// Test round-trip of a complex table definition.
///
/// Simulates a real-world related table with editor tracking and relationships.
#[test]
fn test_table_definition_round_trip() {
    let esri_json = r#"{
        "id": 10,
        "name": "InspectionRecords",
        "type": "Table",
        "objectIdField": "OBJECTID",
        "globalIdField": "GlobalID",
        "displayField": "InspectionType",
        "description": "Property inspection records",
        "fields": [
            {
                "name": "OBJECTID",
                "type": "esriFieldTypeOID",
                "nullable": false,
                "editable": false
            },
            {
                "name": "GlobalID",
                "type": "esriFieldTypeGlobalID",
                "nullable": false,
                "editable": false
            },
            {
                "name": "InspectionType",
                "type": "esriFieldTypeString",
                "nullable": true,
                "editable": true,
                "length": 50
            }
        ],
        "indexes": [
            {
                "name": "pk_idx",
                "fields": "OBJECTID",
                "isAscending": true,
                "isUnique": true,
                "description": ""
            }
        ],
        "editFieldsInfo": {
            "creationDateField": "created_date",
            "creatorField": "created_user",
            "editDateField": "last_edited_date",
            "editorField": "last_edited_user"
        },
        "relationships": [
            {
                "id": 0,
                "name": "Properties_Inspections",
                "role": "esriRelRoleDestination",
                "keyField": "ParcelGUID",
                "cardinality": "esriRelCardinalityOneToMany",
                "relatedTableId": 0
            }
        ],
        "isDataBranchVersioned": true
    }"#;

    let table: TableDefinition =
        serde_json::from_str(esri_json).expect("Should deserialize complex table");

    assert_eq!(*table.id(), 10u32);
    assert_eq!(table.name(), "InspectionRecords");
    assert_eq!(table.fields().len(), 3);
    assert_eq!(table.indexes().len(), 1);
    assert!(table.edit_fields_info().is_some());
    assert_eq!(table.relationships().len(), 1);
    assert_eq!(table.is_data_branch_versioned(), &Some(true));

    // Full round-trip
    let json_output = serde_json::to_string(&table).expect("Should serialize");
    let roundtrip: TableDefinition = serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(*roundtrip.id(), 10u32);
    assert_eq!(roundtrip.fields().len(), 3);
    assert_eq!(roundtrip.indexes().len(), 1);
    assert!(roundtrip.edit_fields_info().is_some());
    assert_eq!(roundtrip.relationships().len(), 1);
}
