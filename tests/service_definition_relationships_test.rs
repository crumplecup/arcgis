//! Tests for relationship class types in service definitions.
//!
//! These tests validate that our types match ESRI's real-world JSON responses
//! from production Feature Services.

use arcgis::{
    LayerRelationship, LayerRelationshipBuilder, RelationshipCardinality, RelationshipClass,
    RelationshipRole, RelationshipRule, RelationshipsResponse,
};

/// Test LayerRelationship deserialization with real ESRI JSON (layer-level).
///
/// Source: ESRI Feature Service layer definition spec
/// <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
#[test]
fn test_layer_relationship_origin() {
    let esri_json = r#"{
        "id": 2,
        "role": "esriRelRoleOrigin",
        "keyField": "GlobalID",
        "cardinality": "esriRelCardinalityOneToMany",
        "relatedTableId": 3,
        "name": "Buildings_Permits"
    }"#;

    let rel: LayerRelationship =
        serde_json::from_str(esri_json).expect("Should deserialize layer relationship");

    assert_eq!(*rel.id(), 2);
    assert_eq!(rel.name(), &Some("Buildings_Permits".to_string()));
    assert_eq!(*rel.role(), RelationshipRole::Origin);
    assert_eq!(*rel.cardinality(), RelationshipCardinality::OneToMany);
    assert_eq!(*rel.related_table_id(), 3);
    assert_eq!(rel.key_field(), "GlobalID");

    // Round-trip
    let json_output = serde_json::to_string(&rel).expect("Should serialize");
    let roundtrip: LayerRelationship =
        serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(rel, roundtrip);
}

/// Test LayerRelationship with destination role.
#[test]
fn test_layer_relationship_destination() {
    let esri_json = r#"{
        "id": 2,
        "role": "esriRelRoleDestination",
        "keyField": "BuildingGUID",
        "cardinality": "esriRelCardinalityOneToMany",
        "relatedTableId": 0
    }"#;

    let rel: LayerRelationship =
        serde_json::from_str(esri_json).expect("Should deserialize destination relationship");

    assert_eq!(*rel.role(), RelationshipRole::Destination);
    assert_eq!(rel.name(), &None);
    assert_eq!(rel.key_field(), "BuildingGUID");

    // Round-trip
    let json_output = serde_json::to_string(&rel).expect("Should serialize");
    let roundtrip: LayerRelationship =
        serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(rel, roundtrip);
}

/// Test all RelationshipCardinality variants.
#[test]
fn test_relationship_cardinality_variants() {
    let one_to_one: RelationshipCardinality =
        serde_json::from_str(r#""esriRelCardinalityOneToOne""#).unwrap();
    assert_eq!(one_to_one, RelationshipCardinality::OneToOne);

    let one_to_many: RelationshipCardinality =
        serde_json::from_str(r#""esriRelCardinalityOneToMany""#).unwrap();
    assert_eq!(one_to_many, RelationshipCardinality::OneToMany);

    let many_to_many: RelationshipCardinality =
        serde_json::from_str(r#""esriRelCardinalityManyToMany""#).unwrap();
    assert_eq!(many_to_many, RelationshipCardinality::ManyToMany);

    // Verify serialization
    assert_eq!(
        serde_json::to_string(&RelationshipCardinality::OneToOne).unwrap(),
        r#""esriRelCardinalityOneToOne""#
    );
    assert_eq!(
        serde_json::to_string(&RelationshipCardinality::ManyToMany).unwrap(),
        r#""esriRelCardinalityManyToMany""#
    );
}

/// Test RelationshipRule deserialization.
///
/// Note: ESRI uses non-standard "ruleID" (not camelCase "ruleId").
#[test]
fn test_relationship_rule() {
    let esri_json = r#"{
        "ruleID": 1,
        "originSubtypeCode": 0,
        "originMinimumCardinality": 0,
        "originMaximumCardinality": -1,
        "destinationSubtypeCode": 0,
        "destinationMinimumCardinality": 0,
        "destinationMaximumCardinality": -1
    }"#;

    let rule: RelationshipRule =
        serde_json::from_str(esri_json).expect("Should deserialize relationship rule");

    assert_eq!(*rule.rule_id(), 1);
    assert_eq!(*rule.origin_subtype_code(), 0);
    assert_eq!(*rule.origin_minimum_cardinality(), 0);
    assert_eq!(*rule.origin_maximum_cardinality(), -1);
    assert_eq!(*rule.destination_subtype_code(), 0);
    assert_eq!(*rule.destination_minimum_cardinality(), 0);
    assert_eq!(*rule.destination_maximum_cardinality(), -1);

    // Verify ruleID serializes correctly (not ruleId)
    let json_output = serde_json::to_value(&rule).expect("Should serialize");
    assert!(
        json_output.get("ruleID").is_some(),
        "Should have ruleID key"
    );
    assert!(
        json_output.get("ruleId").is_none(),
        "Should NOT have ruleId key"
    );

    // Round-trip
    let json_output = serde_json::to_string(&rule).expect("Should serialize");
    let roundtrip: RelationshipRule =
        serde_json::from_str(&json_output).expect("Should round-trip");
    assert_eq!(rule, roundtrip);
}

/// Test RelationshipClass deserialization (service-level).
///
/// Source: ESRI Relationships resource spec
/// <https://developers.arcgis.com/rest/services-reference/enterprise/relationships-feature-service/>
#[test]
fn test_relationship_class_simple() {
    let esri_json = r#"{
        "id": 2,
        "name": "Buildings_Permits",
        "cardinality": "esriRelCardinalityOneToMany",
        "originLayerId": 0,
        "originPrimaryKey": "GlobalID",
        "originForeignKey": "BuildingGUID",
        "destinationLayerId": 1,
        "role": "esriRelRoleOrigin",
        "attributed": false,
        "catalogID": "{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}",
        "rules": []
    }"#;

    let rel: RelationshipClass =
        serde_json::from_str(esri_json).expect("Should deserialize relationship class");

    assert_eq!(*rel.id(), 2);
    assert_eq!(rel.name(), &Some("Buildings_Permits".to_string()));
    assert_eq!(*rel.cardinality(), RelationshipCardinality::OneToMany);
    assert_eq!(*rel.origin_layer_id(), 0);
    assert_eq!(rel.origin_primary_key(), "GlobalID");
    assert_eq!(rel.origin_foreign_key(), &Some("BuildingGUID".to_string()));
    assert_eq!(*rel.destination_layer_id(), Some(1));
    assert_eq!(*rel.role(), RelationshipRole::Origin);
    assert!(!*rel.attributed());
    assert_eq!(
        rel.catalog_id(),
        &Some("{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}".to_string())
    );
    assert_eq!(rel.rules().len(), 0);

    // Verify catalogID serializes correctly (not catalogId)
    let json_output = serde_json::to_value(&rel).expect("Should serialize");
    assert!(
        json_output.get("catalogID").is_some(),
        "Should have catalogID key"
    );
    assert!(
        json_output.get("catalogId").is_none(),
        "Should NOT have catalogId key"
    );

    // Round-trip
    let json_str = serde_json::to_string(&rel).expect("Should serialize");
    let roundtrip: RelationshipClass = serde_json::from_str(&json_str).expect("Should round-trip");
    assert_eq!(rel, roundtrip);
}

/// Test RelationshipClass with rules.
#[test]
fn test_relationship_class_with_rules() {
    let esri_json = r#"{
        "id": 1,
        "name": "Parcels_Owners",
        "cardinality": "esriRelCardinalityManyToMany",
        "originLayerId": 0,
        "originPrimaryKey": "OBJECTID",
        "destinationLayerId": 2,
        "role": "esriRelRoleOrigin",
        "attributed": false,
        "rules": [
            {
                "ruleID": 1,
                "originSubtypeCode": 0,
                "originMinimumCardinality": 0,
                "originMaximumCardinality": -1,
                "destinationSubtypeCode": 0,
                "destinationMinimumCardinality": 0,
                "destinationMaximumCardinality": -1
            }
        ]
    }"#;

    let rel: RelationshipClass =
        serde_json::from_str(esri_json).expect("Should deserialize relationship class with rules");

    assert_eq!(*rel.cardinality(), RelationshipCardinality::ManyToMany);
    assert_eq!(rel.rules().len(), 1);
    assert_eq!(*rel.rules()[0].rule_id(), 1);
    assert_eq!(*rel.rules()[0].origin_maximum_cardinality(), -1);

    // Round-trip
    let json_str = serde_json::to_string(&rel).expect("Should serialize");
    let roundtrip: RelationshipClass = serde_json::from_str(&json_str).expect("Should round-trip");
    assert_eq!(rel, roundtrip);
}

/// Test attributed relationship class.
#[test]
fn test_relationship_class_attributed() {
    let esri_json = r#"{
        "id": 3,
        "name": "Transformers_Phases",
        "cardinality": "esriRelCardinalityManyToMany",
        "originLayerId": 0,
        "originPrimaryKey": "OBJECTID",
        "destinationLayerId": 1,
        "role": "esriRelRoleOrigin",
        "attributed": true,
        "relationshipTableId": 5,
        "keyFieldInRelationshipTable": "PhaseCode",
        "rules": []
    }"#;

    let rel: RelationshipClass =
        serde_json::from_str(esri_json).expect("Should deserialize attributed relationship class");

    assert!(*rel.attributed());
    assert_eq!(*rel.relationship_table_id(), Some(5));
    assert_eq!(
        rel.key_field_in_relationship_table(),
        &Some("PhaseCode".to_string())
    );

    // Round-trip
    let json_str = serde_json::to_string(&rel).expect("Should serialize");
    let roundtrip: RelationshipClass = serde_json::from_str(&json_str).expect("Should round-trip");
    assert_eq!(rel, roundtrip);
}

/// Test RelationshipsResponse deserialization.
#[test]
fn test_relationships_response() {
    let esri_json = r#"{
        "relationships": [
            {
                "id": 0,
                "name": "Parcels_Owners",
                "cardinality": "esriRelCardinalityOneToMany",
                "originLayerId": 0,
                "originPrimaryKey": "GlobalID",
                "originForeignKey": "ParcelGUID",
                "destinationLayerId": 1,
                "role": "esriRelRoleOrigin",
                "attributed": false,
                "rules": []
            },
            {
                "id": 1,
                "name": "Owners_Parcels",
                "cardinality": "esriRelCardinalityOneToMany",
                "originLayerId": 1,
                "originPrimaryKey": "OBJECTID",
                "destinationLayerId": 0,
                "role": "esriRelRoleDestination",
                "attributed": false,
                "rules": []
            }
        ]
    }"#;

    let response: RelationshipsResponse =
        serde_json::from_str(esri_json).expect("Should deserialize relationships response");

    assert_eq!(response.relationships().len(), 2);
    assert_eq!(*response.relationships()[0].id(), 0);
    assert_eq!(*response.relationships()[1].id(), 1);
    assert_eq!(
        *response.relationships()[1].role(),
        RelationshipRole::Destination
    );

    // Round-trip
    let json_str = serde_json::to_string(&response).expect("Should serialize");
    let roundtrip: RelationshipsResponse =
        serde_json::from_str(&json_str).expect("Should round-trip");
    assert_eq!(response, roundtrip);
}

/// Test builder pattern for LayerRelationship.
#[test]
fn test_layer_relationship_builder() {
    let rel = LayerRelationshipBuilder::default()
        .id(2_i32)
        .name("Buildings_Permits")
        .role(RelationshipRole::Origin)
        .cardinality(RelationshipCardinality::OneToMany)
        .related_table_id(3_i32)
        .key_field("GlobalID")
        .build()
        .expect("Should build layer relationship");

    assert_eq!(*rel.id(), 2);
    assert_eq!(rel.name(), &Some("Buildings_Permits".to_string()));
    assert_eq!(*rel.role(), RelationshipRole::Origin);
    assert_eq!(*rel.cardinality(), RelationshipCardinality::OneToMany);
    assert_eq!(*rel.related_table_id(), 3);
    assert_eq!(rel.key_field(), "GlobalID");
}
