//! Tests for service definition validation (Phase 5).
//!
//! Each test validates a specific ESRI constraint, verifies the correct error
//! variant is returned, and checks that the Display message contains enough
//! context for an agent to fix the issue.

use arcgis::{
    FieldDefinitionBuilder, FieldType, GeometryTypeDefinition, LayerDefinitionBuilder,
    ServiceDefinitionBuilder, ServiceDefinitionValidationError, TableDefinitionBuilder,
};

// ==================== LayerDefinition validation ====================

/// A fully valid layer should pass all checks.
#[test]
fn test_layer_valid_with_oid() {
    let oid = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("Valid OID field");

    let mut builder = LayerDefinitionBuilder::default();
    builder
        .id(0u32)
        .name("Buildings")
        .geometry_type(GeometryTypeDefinition::Polygon);
    let layer = builder.add_field(oid).build().expect("Valid layer");

    assert!(layer.validate().is_ok());
}

/// A layer with branch versioning requires a GlobalID field.
#[test]
fn test_layer_valid_with_versioning() {
    let oid = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("Valid OID");

    let gid = FieldDefinitionBuilder::default()
        .name("GlobalID")
        .field_type(FieldType::GlobalId)
        .nullable(false)
        .editable(false)
        .length(38)
        .build()
        .expect("Valid GlobalID");

    let mut builder = LayerDefinitionBuilder::default();
    builder
        .id(0u32)
        .name("Buildings")
        .geometry_type(GeometryTypeDefinition::Polygon)
        .is_data_branch_versioned(true);
    let layer = builder
        .add_field(oid)
        .add_field(gid)
        .build()
        .expect("Valid versioned layer");

    assert!(layer.validate().is_ok());
}

/// Missing ObjectID field produces a clear, actionable error.
#[test]
fn test_layer_missing_oid() {
    let layer = LayerDefinitionBuilder::default()
        .id(0u32)
        .name("Buildings")
        .geometry_type(GeometryTypeDefinition::Polygon)
        .build()
        .expect("Valid layer struct");

    let errors = layer.validate().unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0],
        ServiceDefinitionValidationError::MissingObjectId {
            entity_type: "Layer",
            name: "Buildings".to_string(),
            id: 0,
        }
    );

    // Message should be agent-actionable
    let msg = errors[0].to_string();
    assert!(msg.contains("Layer 'Buildings' (id=0)"));
    assert!(msg.contains("FieldType::Oid"));
    assert!(msg.contains("developers.arcgis.com"));
}

/// Multiple ObjectID fields is an error.
#[test]
fn test_layer_multiple_oids() {
    let oid1 = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("OID 1");

    let oid2 = FieldDefinitionBuilder::default()
        .name("OID2")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("OID 2");

    let mut builder = LayerDefinitionBuilder::default();
    builder
        .id(0u32)
        .name("BadLayer")
        .geometry_type(GeometryTypeDefinition::Point);
    let layer = builder
        .add_field(oid1)
        .add_field(oid2)
        .build()
        .expect("Layer");

    let errors = layer.validate().unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ServiceDefinitionValidationError::MultipleObjectIds { count: 2, .. }
    )));
}

/// ObjectID with nullable=true is caught.
#[test]
fn test_layer_oid_nullable() {
    let bad_oid = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(true) // wrong!
        .editable(false)
        .build()
        .expect("Bad OID");

    let mut builder = LayerDefinitionBuilder::default();
    builder
        .id(0u32)
        .name("Buildings")
        .geometry_type(GeometryTypeDefinition::Polygon);
    let layer = builder.add_field(bad_oid).build().expect("Layer");

    let errors = layer.validate().unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ServiceDefinitionValidationError::OidFieldInvalidConfig {
            field_name,
            ..
        } if field_name == "OBJECTID"
    )));

    let msg = errors
        .iter()
        .find(|e| {
            matches!(
                e,
                ServiceDefinitionValidationError::OidFieldInvalidConfig { .. }
            )
        })
        .unwrap()
        .to_string();
    assert!(msg.contains("nullable=false"));
    assert!(msg.contains("editable=false"));
}

/// Branch versioning without GlobalID is caught.
#[test]
fn test_layer_missing_global_id_for_versioning() {
    let oid = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("OID");

    let mut builder = LayerDefinitionBuilder::default();
    builder
        .id(0u32)
        .name("VersionedLayer")
        .geometry_type(GeometryTypeDefinition::Point)
        .is_data_branch_versioned(true);
    let layer = builder.add_field(oid).build().expect("Layer");

    let errors = layer.validate().unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ServiceDefinitionValidationError::MissingGlobalIdForVersioning { .. }
    )));

    let msg = errors
        .iter()
        .find(|e| {
            matches!(
                e,
                ServiceDefinitionValidationError::MissingGlobalIdForVersioning { .. }
            )
        })
        .unwrap()
        .to_string();
    assert!(msg.contains("GlobalID"));
    assert!(msg.contains("FieldType::GlobalId"));
    assert!(msg.contains("branch-version-scenarios"));
}

/// Duplicate field names (case-insensitive) are caught.
#[test]
fn test_layer_duplicate_field_names() {
    let f1 = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("OID");

    let f2 = FieldDefinitionBuilder::default()
        .name("Status")
        .field_type(FieldType::Integer)
        .build()
        .expect("Status");

    let f3 = FieldDefinitionBuilder::default()
        .name("status") // duplicate of Status (case-insensitive)
        .field_type(FieldType::Integer)
        .build()
        .expect("status dupe");

    let mut builder = LayerDefinitionBuilder::default();
    builder
        .id(0u32)
        .name("DupLayer")
        .geometry_type(GeometryTypeDefinition::Point);
    let layer = builder
        .add_field(f1)
        .add_field(f2)
        .add_field(f3)
        .build()
        .expect("Layer");

    let errors = layer.validate().unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ServiceDefinitionValidationError::DuplicateFieldName { .. }
    )));
}

/// Named field references to non-existent fields are caught.
#[test]
fn test_layer_field_ref_not_found() {
    let oid = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("OID");

    let mut builder = LayerDefinitionBuilder::default();
    builder
        .id(0u32)
        .name("BadRefs")
        .geometry_type(GeometryTypeDefinition::Point)
        .display_field("NonExistentField"); // doesn't exist
    let layer = builder.add_field(oid).build().expect("Layer");

    let errors = layer.validate().unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ServiceDefinitionValidationError::FieldRefNotFound {
            ref_type: "display",
            field_name,
            ..
        } if field_name == "NonExistentField"
    )));

    let msg = errors
        .iter()
        .find(|e| {
            matches!(
                e,
                ServiceDefinitionValidationError::FieldRefNotFound {
                    ref_type: "display",
                    ..
                }
            )
        })
        .unwrap()
        .to_string();
    assert!(msg.contains("display_field"));
    assert!(msg.contains("NonExistentField"));
}

// ==================== TableDefinition validation ====================

/// Valid table passes all checks.
#[test]
fn test_table_valid() {
    let oid = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("OID");

    let mut builder = TableDefinitionBuilder::default();
    builder.id(1u32).name("Permits");
    let table = builder.add_field(oid).build().expect("Table");

    assert!(table.validate().is_ok());
}

/// Table validation uses "Table" in error messages, not "Layer".
#[test]
fn test_table_error_context() {
    let table = TableDefinitionBuilder::default()
        .id(1u32)
        .name("Permits")
        .build()
        .expect("Table");

    let errors = table.validate().unwrap_err();
    let msg = errors[0].to_string();
    assert!(msg.contains("Table 'Permits' (id=1)"), "Got: {}", msg);
}

// ==================== ServiceDefinition validation ====================

/// Valid service with a proper layer passes.
#[test]
fn test_service_valid() {
    let oid = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .nullable(false)
        .editable(false)
        .build()
        .expect("OID");

    let mut layer_builder = LayerDefinitionBuilder::default();
    layer_builder
        .id(0u32)
        .name("Points")
        .geometry_type(GeometryTypeDefinition::Point);
    let layer = layer_builder.add_field(oid).build().expect("Layer");

    let mut svc_builder = ServiceDefinitionBuilder::default();
    svc_builder.name("MyService");
    let svc = svc_builder.add_layer(layer).build().expect("Service");

    assert!(svc.validate().is_ok());
}

/// Duplicate layer IDs are caught at the service level.
#[test]
fn test_service_duplicate_layer_ids() {
    let make_layer = |name: &str| {
        let oid = FieldDefinitionBuilder::default()
            .name("OBJECTID")
            .field_type(FieldType::Oid)
            .nullable(false)
            .editable(false)
            .build()
            .expect("OID");
        let mut b = LayerDefinitionBuilder::default();
        b.id(0u32) // same ID for both!
            .name(name)
            .geometry_type(GeometryTypeDefinition::Point);
        b.add_field(oid).build().expect("Layer")
    };

    let mut svc_builder = ServiceDefinitionBuilder::default();
    svc_builder.name("DupService");
    let svc = svc_builder
        .add_layer(make_layer("Alpha"))
        .add_layer(make_layer("Beta"))
        .build()
        .expect("Service");

    let errors = svc.validate().unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ServiceDefinitionValidationError::DuplicateId { id: 0, .. }
    )));

    let msg = errors
        .iter()
        .find(|e| matches!(e, ServiceDefinitionValidationError::DuplicateId { .. }))
        .unwrap()
        .to_string();
    assert!(msg.contains("Duplicate ID 0"));
    assert!(msg.contains("Alpha"));
    assert!(msg.contains("Beta"));
}

/// Layer and table with same ID triggers duplicate ID error.
#[test]
fn test_service_layer_table_id_overlap() {
    let make_oid = || {
        FieldDefinitionBuilder::default()
            .name("OBJECTID")
            .field_type(FieldType::Oid)
            .nullable(false)
            .editable(false)
            .build()
            .expect("OID")
    };

    let mut layer_b = LayerDefinitionBuilder::default();
    layer_b
        .id(0u32)
        .name("Buildings")
        .geometry_type(GeometryTypeDefinition::Polygon);
    let layer = layer_b.add_field(make_oid()).build().expect("Layer");

    let mut table_b = TableDefinitionBuilder::default();
    table_b.id(0u32).name("Permits"); // same ID as layer!
    let table = table_b.add_field(make_oid()).build().expect("Table");

    let mut svc_builder = ServiceDefinitionBuilder::default();
    svc_builder.name("Conflict");
    let svc = svc_builder
        .add_layer(layer)
        .add_table(table)
        .build()
        .expect("Service");

    let errors = svc.validate().unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ServiceDefinitionValidationError::DuplicateId { id: 0, .. }
    )));
}

/// Service validation propagates errors from nested layers.
#[test]
fn test_service_propagates_layer_errors() {
    // Layer with no fields
    let layer = LayerDefinitionBuilder::default()
        .id(0u32)
        .name("Empty")
        .geometry_type(GeometryTypeDefinition::Point)
        .build()
        .expect("Layer");

    let mut svc_builder = ServiceDefinitionBuilder::default();
    svc_builder.name("BrokenService");
    let svc = svc_builder.add_layer(layer).build().expect("Service");

    let errors = svc.validate().unwrap_err();
    assert!(!errors.is_empty());
    // The MissingObjectId error from the layer should surface here
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ServiceDefinitionValidationError::MissingObjectId { .. }))
    );
}

/// All errors are collected, not just the first.
#[test]
fn test_multiple_errors_collected() {
    // Layer with: no OID, versioning enabled (no GlobalID), bad display_field ref
    let mut builder = LayerDefinitionBuilder::default();
    builder
        .id(0u32)
        .name("Broken")
        .geometry_type(GeometryTypeDefinition::Point)
        .is_data_branch_versioned(true)
        .display_field("NoSuchField");
    let layer = builder.build().expect("Layer");

    let errors = layer.validate().unwrap_err();
    // Expect: MissingObjectId, MissingGlobalIdForVersioning, FieldRefNotFound(display)
    assert!(
        errors.len() >= 3,
        "Expected at least 3 errors, got {}",
        errors.len()
    );
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ServiceDefinitionValidationError::MissingObjectId { .. }))
    );
    assert!(errors.iter().any(|e| matches!(
        e,
        ServiceDefinitionValidationError::MissingGlobalIdForVersioning { .. }
    )));
    assert!(errors.iter().any(|e| matches!(
        e,
        ServiceDefinitionValidationError::FieldRefNotFound {
            ref_type: "display",
            ..
        }
    )));
}
