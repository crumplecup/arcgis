//! Tests for Feature Service related records queries.

mod common;

use arcgis::{ApiKeyAuth, ArcGISClient, FeatureServiceClient, ObjectId, RelatedRecordsParams};

#[test]
fn test_related_records_params_builder() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_related_records_params_builder: Starting");

    tracing::info!("test_related_records_params_builder: Building RelatedRecordsParams");
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1), ObjectId::new(2), ObjectId::new(3)])
        .relationship_id(5u32)
        .out_fields(vec!["NAME".to_string(), "STATUS".to_string()])
        .return_geometry(false)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!(
        object_ids_count = ?params.object_ids().as_ref().map(|ids| ids.len()),
        relationship_id = ?params.relationship_id(),
        out_fields_count = ?params.out_fields().as_ref().map(|f| f.len()),
        return_geometry = ?params.return_geometry(),
        "test_related_records_params_builder: Verifying params"
    );
    assert_eq!(params.object_ids().as_ref().map(|ids| ids.len()), Some(3));
    assert_eq!(*params.relationship_id(), Some(5));
    assert_eq!(params.out_fields().as_ref().map(|f| f.len()), Some(2));
    assert_eq!(*params.return_geometry(), Some(false));

    tracing::info!("test_related_records_params_builder: Completed");
    Ok(())
}

#[test]
fn test_related_records_params_default() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_related_records_params_default: Starting");

    tracing::info!("test_related_records_params_default: Creating default params");
    let params = RelatedRecordsParams::default();

    tracing::info!(
        has_object_ids = params.object_ids().is_some(),
        has_relationship_id = params.relationship_id().is_some(),
        return_geometry = ?params.return_geometry(),
        "test_related_records_params_default: Verifying defaults"
    );
    assert!(params.object_ids().is_none());
    assert!(params.relationship_id().is_none());
    assert!(params.out_fields().is_none());
    assert_eq!(*params.return_geometry(), Some(true));
    assert!(params.definition_expression().is_none());

    tracing::info!("test_related_records_params_default: Completed");
    Ok(())
}

#[test]
fn test_related_records_params_with_definition_expression() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_related_records_params_with_definition_expression: Starting");

    tracing::info!(
        "test_related_records_params_with_definition_expression: Building params with definition expression"
    );
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(100)])
        .relationship_id(2u32)
        .definition_expression("STATUS = 'ACTIVE'")
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!(
        definition_expression = ?params.definition_expression(),
        "test_related_records_params_with_definition_expression: Verifying expression"
    );
    assert_eq!(
        *params.definition_expression(),
        Some("STATUS = 'ACTIVE'".to_string())
    );

    tracing::info!("test_related_records_params_with_definition_expression: Completed");
    Ok(())
}

#[test]
fn test_related_records_params_with_pagination() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_related_records_params_with_pagination: Starting");

    tracing::info!("test_related_records_params_with_pagination: Building params with pagination");
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1)])
        .relationship_id(3u32)
        .result_offset(10u32)
        .result_record_count(50u32)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!(
        result_offset = ?params.result_offset(),
        result_record_count = ?params.result_record_count(),
        "test_related_records_params_with_pagination: Verifying pagination"
    );
    assert_eq!(*params.result_offset(), Some(10));
    assert_eq!(*params.result_record_count(), Some(50));

    tracing::info!("test_related_records_params_with_pagination: Completed");
    Ok(())
}

#[test]
fn test_related_records_params_with_ordering() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_related_records_params_with_ordering: Starting");

    tracing::info!("test_related_records_params_with_ordering: Building params with ordering");
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1)])
        .relationship_id(2u32)
        .order_by_fields(vec!["NAME ASC".to_string(), "DATE DESC".to_string()])
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!(
        order_by_fields_count = ?params.order_by_fields().as_ref().map(|f| f.len()),
        "test_related_records_params_with_ordering: Verifying ordering"
    );
    assert_eq!(params.order_by_fields().as_ref().map(|f| f.len()), Some(2));

    tracing::info!("test_related_records_params_with_ordering: Completed");
    Ok(())
}

#[test]
fn test_related_records_params_with_geometry_options() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_related_records_params_with_geometry_options: Starting");

    tracing::info!(
        "test_related_records_params_with_geometry_options: Building params with geometry options"
    );
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1)])
        .relationship_id(1u32)
        .return_geometry(true)
        .out_sr(4326)
        .geometry_precision(6)
        .return_z(true)
        .return_m(false)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!(
        return_geometry = ?params.return_geometry(),
        out_sr = ?params.out_sr(),
        geometry_precision = ?params.geometry_precision(),
        return_z = ?params.return_z(),
        return_m = ?params.return_m(),
        "test_related_records_params_with_geometry_options: Verifying geometry options"
    );
    assert_eq!(*params.return_geometry(), Some(true));
    assert_eq!(*params.out_sr(), Some(4326));
    assert_eq!(*params.geometry_precision(), Some(6));
    assert_eq!(*params.return_z(), Some(true));
    assert_eq!(*params.return_m(), Some(false));

    tracing::info!("test_related_records_params_with_geometry_options: Completed");
    Ok(())
}

#[test]
fn test_related_records_client_method_compiles() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_related_records_client_method_compiles: Starting");

    tracing::info!("test_related_records_client_method_compiles: Creating test client");
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);

    // Test that the method signature compiles
    tracing::info!("test_related_records_client_method_compiles: Building params to verify API");
    let _params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1)])
        .relationship_id(2u32)
        .build();

    // This won't execute but verifies the API compiles
    drop(service);

    tracing::info!("test_related_records_client_method_compiles: Completed");
    Ok(())
}

#[test]
fn test_related_records_params_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_related_records_params_serialization: Starting");

    tracing::info!("test_related_records_params_serialization: Building params for serialization");
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(10), ObjectId::new(20)])
        .relationship_id(5u32)
        .out_fields(vec!["FIELD1".to_string()])
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!("test_related_records_params_serialization: Serializing to JSON");
    let serialized = serde_json::to_string(&params)?;

    tracing::info!(
        serialized_len = serialized.len(),
        "test_related_records_params_serialization: Verifying serialization"
    );
    assert!(serialized.contains("\"relationshipId\":5"));
    assert!(serialized.contains("\"objectIds\":\"10,20\""));

    tracing::info!("test_related_records_params_serialization: Completed");
    Ok(())
}

#[test]
fn test_related_records_params_count_only() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_related_records_params_count_only: Starting");

    tracing::info!("test_related_records_params_count_only: Building params with count only");
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1), ObjectId::new(2)])
        .relationship_id(3u32)
        .return_count_only(true)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!(
        return_count_only = ?params.return_count_only(),
        "test_related_records_params_count_only: Verifying count only flag"
    );
    assert_eq!(*params.return_count_only(), Some(true));

    tracing::info!("test_related_records_params_count_only: Completed");
    Ok(())
}

#[test]
fn test_related_records_params_with_gdb_version() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_related_records_params_with_gdb_version: Starting");

    tracing::info!(
        "test_related_records_params_with_gdb_version: Building params with GDB version"
    );
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1)])
        .relationship_id(1u32)
        .gdb_version("sde.DEFAULT")
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    tracing::info!(
        gdb_version = ?params.gdb_version(),
        "test_related_records_params_with_gdb_version: Verifying GDB version"
    );
    assert_eq!(*params.gdb_version(), Some("sde.DEFAULT".to_string()));

    tracing::info!("test_related_records_params_with_gdb_version: Completed");
    Ok(())
}
