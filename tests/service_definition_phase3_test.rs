//! Tests for Phase 3 service definition advanced features.
//!
//! These tests validate that our types match ESRI's real-world JSON responses
//! from production Feature Services.

use arcgis::{
    CodedValueCode, CodedValueDomain, CodedValueDomainBuilder, DomainCodedValue, DrawingTool,
    EditFieldsInfo, EditFieldsInfoBuilder, FeatureTemplate, FeatureTemplateBuilder,
    FieldDefinition, FieldType, Index, IndexBuilder, MergePolicy, RangeDomain, RangeDomainBuilder,
    SplitPolicy, TemplatePrototypeBuilder,
};
use std::collections::HashMap;

/// Test CodedValueDomain deserialization with real ESRI JSON (integer codes).
///
/// Source: https://sampleserver6.arcgisonline.com/arcgis/rest/services/Military/FeatureServer/6
/// Field: "echelon"
#[test]
fn test_coded_value_domain_integer_codes() {
    let esri_json = r#"{
        "type": "codedValue",
        "name": "Echelon",
        "description": "Echelon or command level",
        "codedValues": [
            {"name": "Team", "code": 0},
            {"name": "Squad", "code": 1},
            {"name": "Section", "code": 11},
            {"name": "Platoon/Detachment", "code": 111},
            {"name": "Company/Battery/Troop", "code": 2},
            {"name": "Battalion/Squadron", "code": 22}
        ],
        "mergePolicy": "esriMPTDefaultValue",
        "splitPolicy": "esriSPTDuplicate"
    }"#;

    // Deserialize from ESRI JSON
    let domain: CodedValueDomain =
        serde_json::from_str(esri_json).expect("Should deserialize real ESRI coded value domain");

    // Verify structure
    assert_eq!(domain.name(), "Echelon");
    assert_eq!(
        domain.description(),
        &Some("Echelon or command level".to_string())
    );
    assert_eq!(domain.coded_values().len(), 6);
    assert_eq!(domain.merge_policy(), &Some(MergePolicy::DefaultValue));
    assert_eq!(domain.split_policy(), &Some(SplitPolicy::Duplicate));

    // Verify first coded value
    let first_value = &domain.coded_values()[0];
    assert_eq!(first_value.name(), "Team");
    assert_eq!(first_value.code(), &CodedValueCode::Number(0.0));

    // Round-trip serialization
    let json_output = serde_json::to_string(&domain).expect("Should serialize");
    let roundtrip: CodedValueDomain =
        serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(domain, roundtrip);
}

/// Test CodedValueDomain with string codes.
///
/// Source: https://sampleserver6.arcgisonline.com/arcgis/rest/services/Military/FeatureServer/6
/// Field: "reliability"
#[test]
fn test_coded_value_domain_string_codes() {
    let esri_json = r#"{
        "type": "codedValue",
        "name": "Reliability",
        "description": "Reliability Rating",
        "codedValues": [
            {"name": "Completely Reliable", "code": "A"},
            {"name": "Usually Reliable", "code": "B"},
            {"name": "Fairly Reliable", "code": "C"},
            {"name": "Not Usually Reliable", "code": "D"},
            {"name": "Unreliable", "code": "E"},
            {"name": "Reliability Cannot Be Judged", "code": "F"}
        ],
        "mergePolicy": "esriMPTDefaultValue",
        "splitPolicy": "esriSPTDuplicate"
    }"#;

    let domain: CodedValueDomain = serde_json::from_str(esri_json)
        .expect("Should deserialize coded value domain with string codes");

    assert_eq!(domain.name(), "Reliability");
    assert_eq!(domain.coded_values().len(), 6);

    // Verify string code
    let first_value = &domain.coded_values()[0];
    assert_eq!(first_value.name(), "Completely Reliable");
    assert_eq!(first_value.code(), &CodedValueCode::String("A".to_string()));

    // Round-trip
    let json_output = serde_json::to_string(&domain).expect("Should serialize");
    let roundtrip: CodedValueDomain =
        serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(domain, roundtrip);
}

/// Test RangeDomain deserialization with real ESRI JSON.
///
/// Source: https://sampleserver6.arcgisonline.com/arcgis/rest/services/Military/FeatureServer/6
/// Field: "direction"
#[test]
fn test_range_domain() {
    let esri_json = r#"{
        "type": "range",
        "name": "Direction",
        "description": "Direction of Movement",
        "range": [-1, 360],
        "mergePolicy": "esriMPTDefaultValue",
        "splitPolicy": "esriSPTDuplicate"
    }"#;

    let domain: RangeDomain =
        serde_json::from_str(esri_json).expect("Should deserialize real ESRI range domain");

    assert_eq!(domain.name(), "Direction");
    assert_eq!(
        domain.description(),
        &Some("Direction of Movement".to_string())
    );
    assert_eq!(domain.range(), &[-1.0, 360.0]);
    assert_eq!(domain.merge_policy(), &Some(MergePolicy::DefaultValue));
    assert_eq!(domain.split_policy(), &Some(SplitPolicy::Duplicate));

    // Round-trip
    let json_output = serde_json::to_string(&domain).expect("Should serialize");
    let roundtrip: RangeDomain = serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(domain, roundtrip);
}

/// Test FeatureTemplate deserialization with real ESRI JSON.
///
/// Source: https://sampleserver6.arcgisonline.com/arcgis/rest/services/Military/FeatureServer/6
/// Subtype template
#[test]
fn test_feature_template() {
    let esri_json = r#"{
        "name": "Mechanized Infantry BTG",
        "description": "Brigade Tactical Group",
        "prototype": {
            "attributes": {
                "countrycode": "US",
                "reinforced": 0,
                "direction": -1,
                "symbolname": "Mechanized Infantry BTG"
            }
        },
        "drawingTool": "esriFeatureEditToolPoint"
    }"#;

    let template: FeatureTemplate =
        serde_json::from_str(esri_json).expect("Should deserialize real ESRI feature template");

    assert_eq!(template.name(), "Mechanized Infantry BTG");
    assert_eq!(
        template.description(),
        &Some("Brigade Tactical Group".to_string())
    );
    assert_eq!(template.drawing_tool(), &DrawingTool::Point);

    // Verify prototype attributes
    let attrs = template.prototype().attributes();
    assert_eq!(attrs.len(), 4);
    assert_eq!(attrs.get("countrycode").unwrap().as_str().unwrap(), "US");
    assert_eq!(attrs.get("reinforced").unwrap().as_i64().unwrap(), 0);

    // Round-trip
    let json_output = serde_json::to_string(&template).expect("Should serialize");
    let roundtrip: FeatureTemplate = serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(template, roundtrip);
}

/// Test Index deserialization with real ESRI JSON.
///
/// Source: https://sampleserver6.arcgisonline.com/arcgis/rest/services/Wildfire/FeatureServer/0
#[test]
fn test_index_single_field() {
    let esri_json = r#"{
        "name": "r147_sde_rowid_uk",
        "fields": "objectid",
        "isAscending": true,
        "isUnique": true,
        "description": ""
    }"#;

    let index: Index = serde_json::from_str(esri_json).expect("Should deserialize real ESRI index");

    assert_eq!(index.name(), "r147_sde_rowid_uk");
    assert_eq!(index.fields().len(), 1);
    assert_eq!(index.fields()[0], "objectid");
    assert_eq!(index.is_ascending(), &Some(true));
    assert_eq!(index.is_unique(), &Some(true));
    assert_eq!(index.description(), &Some("".to_string()));

    // Round-trip (should serialize back to comma-separated string)
    let json_output = serde_json::to_string(&index).expect("Should serialize");
    let roundtrip: Index = serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(index, roundtrip);
}

/// Test Index with multiple fields (comma-separated).
#[test]
fn test_index_multiple_fields() {
    let esri_json = r#"{
        "name": "composite_index",
        "fields": "field1,field2,field3",
        "isAscending": true,
        "isUnique": false
    }"#;

    let index: Index =
        serde_json::from_str(esri_json).expect("Should deserialize multi-field index");

    assert_eq!(index.fields().len(), 3);
    assert_eq!(index.fields()[0], "field1");
    assert_eq!(index.fields()[1], "field2");
    assert_eq!(index.fields()[2], "field3");

    // Verify serialization produces comma-separated string
    let json_output = serde_json::to_value(&index).expect("Should serialize");
    assert_eq!(
        json_output["fields"].as_str().unwrap(),
        "field1,field2,field3"
    );
}

/// Test EditFieldsInfo deserialization with real ESRI JSON.
///
/// Source: https://sampleserver6.arcgisonline.com/arcgis/rest/services/Wildfire/FeatureServer/0
#[test]
fn test_edit_fields_info() {
    let esri_json = r#"{
        "creationDateField": "created_date",
        "creatorField": "created_user",
        "editDateField": "last_edited_date",
        "editorField": "last_edited_user"
    }"#;

    let edit_fields: EditFieldsInfo =
        serde_json::from_str(esri_json).expect("Should deserialize real ESRI edit fields info");

    assert_eq!(
        edit_fields.creation_date_field(),
        &Some("created_date".to_string())
    );
    assert_eq!(
        edit_fields.creator_field(),
        &Some("created_user".to_string())
    );
    assert_eq!(
        edit_fields.edit_date_field(),
        &Some("last_edited_date".to_string())
    );
    assert_eq!(
        edit_fields.editor_field(),
        &Some("last_edited_user".to_string())
    );

    // Round-trip
    let json_output = serde_json::to_string(&edit_fields).expect("Should serialize");
    let roundtrip: EditFieldsInfo = serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(edit_fields, roundtrip);
}

/// Test FieldDefinition with domain (integration test).
///
/// This tests that domains properly integrate into field definitions.
#[test]
fn test_field_definition_with_coded_value_domain() {
    let esri_json = r#"{
        "name": "status",
        "type": "esriFieldTypeInteger",
        "alias": "Status",
        "domain": {
            "type": "codedValue",
            "name": "StatusDomain",
            "codedValues": [
                {"name": "Active", "code": 1},
                {"name": "Inactive", "code": 0}
            ],
            "mergePolicy": "esriMPTDefaultValue",
            "splitPolicy": "esriSPTDuplicate"
        }
    }"#;

    let field: FieldDefinition =
        serde_json::from_str(esri_json).expect("Should deserialize field with domain");

    assert_eq!(field.name(), "status");
    assert_eq!(*field.field_type(), FieldType::Integer);
    assert!(field.domain().is_some());

    // Round-trip
    let json_output = serde_json::to_string(&field).expect("Should serialize");
    let roundtrip: FieldDefinition = serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(field, roundtrip);
}

/// Test builder pattern for CodedValueDomain.
#[test]
fn test_coded_value_domain_builder() {
    let coded_values = vec![
        DomainCodedValue::new("Low".to_string(), CodedValueCode::Number(1.0)),
        DomainCodedValue::new("Medium".to_string(), CodedValueCode::Number(2.0)),
        DomainCodedValue::new("High".to_string(), CodedValueCode::Number(3.0)),
    ];

    let domain = CodedValueDomainBuilder::default()
        .name("Priority")
        .description("Task priority levels")
        .coded_values(coded_values)
        .merge_policy(MergePolicy::DefaultValue)
        .split_policy(SplitPolicy::Duplicate)
        .build()
        .expect("Should build coded value domain");

    assert_eq!(domain.name(), "Priority");
    assert_eq!(domain.coded_values().len(), 3);
    assert_eq!(domain.coded_values()[0].name(), "Low");
}

/// Test builder pattern for RangeDomain.
#[test]
fn test_range_domain_builder() {
    let domain = RangeDomainBuilder::default()
        .name("Temperature")
        .description("Temperature range in Celsius")
        .range([-40.0, 50.0])
        .merge_policy(MergePolicy::SumValues)
        .split_policy(SplitPolicy::GeometryRatio)
        .build()
        .expect("Should build range domain");

    assert_eq!(domain.name(), "Temperature");
    assert_eq!(domain.range(), &[-40.0, 50.0]);
    assert_eq!(domain.merge_policy(), &Some(MergePolicy::SumValues));
}

/// Test builder pattern for FeatureTemplate.
#[test]
fn test_feature_template_builder() {
    let mut attributes = HashMap::new();
    attributes.insert("BuildingType".to_string(), serde_json::json!("Residential"));
    attributes.insert("Status".to_string(), serde_json::json!("Planned"));

    let prototype = TemplatePrototypeBuilder::default()
        .attributes(attributes)
        .build()
        .expect("Should build prototype");

    let template = FeatureTemplateBuilder::default()
        .name("Residential Building")
        .description("Single-family home")
        .prototype(prototype)
        .drawing_tool(DrawingTool::Polygon)
        .build()
        .expect("Should build feature template");

    assert_eq!(template.name(), "Residential Building");
    assert_eq!(template.drawing_tool(), &DrawingTool::Polygon);
}

/// Test builder pattern for Index.
#[test]
fn test_index_builder() {
    let index = IndexBuilder::default()
        .name("name_index")
        .fields(vec!["lastname".to_string(), "firstname".to_string()])
        .is_ascending(true)
        .is_unique(false)
        .description("Composite index on name fields")
        .build()
        .expect("Should build index");

    assert_eq!(index.name(), "name_index");
    assert_eq!(index.fields().len(), 2);
    assert_eq!(index.is_unique(), &Some(false));
}

/// Test builder pattern for EditFieldsInfo.
#[test]
fn test_edit_fields_info_builder() {
    let edit_fields = EditFieldsInfoBuilder::default()
        .creation_date_field("CreatedDate")
        .creator_field("CreatedBy")
        .edit_date_field("ModifiedDate")
        .editor_field("ModifiedBy")
        .build()
        .expect("Should build edit fields info");

    assert_eq!(
        edit_fields.creation_date_field(),
        &Some("CreatedDate".to_string())
    );
    assert_eq!(edit_fields.editor_field(), &Some("ModifiedBy".to_string()));
}
