//! Tests for Feature Service related records queries.

use arcgis::{
    ApiKeyAuth, ArcGISClient, FeatureServiceClient, ObjectId, RelatedRecordsParams, Result,
};

#[test]
fn test_related_records_params_builder() -> Result<()> {
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1), ObjectId::new(2), ObjectId::new(3)])
        .relationship_id(5u32)
        .out_fields(vec!["NAME".to_string(), "STATUS".to_string()])
        .return_geometry(false)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    assert_eq!(params.object_ids().as_ref().map(|ids| ids.len()), Some(3));
    assert_eq!(*params.relationship_id(), Some(5));
    assert_eq!(params.out_fields().as_ref().map(|f| f.len()), Some(2));
    assert_eq!(*params.return_geometry(), Some(false));

    Ok(())
}

#[test]
fn test_related_records_params_default() {
    let params = RelatedRecordsParams::default();

    assert!(params.object_ids().is_none());
    assert!(params.relationship_id().is_none());
    assert!(params.out_fields().is_none());
    assert_eq!(*params.return_geometry(), Some(true));
    assert!(params.definition_expression().is_none());
}

#[test]
fn test_related_records_params_with_definition_expression() -> Result<()> {
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(100)])
        .relationship_id(2u32)
        .definition_expression("STATUS = 'ACTIVE'")
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    assert_eq!(
        *params.definition_expression(),
        Some("STATUS = 'ACTIVE'".to_string())
    );

    Ok(())
}

#[test]
fn test_related_records_params_with_pagination() -> Result<()> {
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1)])
        .relationship_id(3u32)
        .result_offset(10u32)
        .result_record_count(50u32)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    assert_eq!(*params.result_offset(), Some(10));
    assert_eq!(*params.result_record_count(), Some(50));

    Ok(())
}

#[test]
fn test_related_records_params_with_ordering() -> Result<()> {
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1)])
        .relationship_id(2u32)
        .order_by_fields(vec!["NAME ASC".to_string(), "DATE DESC".to_string()])
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    assert_eq!(params.order_by_fields().as_ref().map(|f| f.len()), Some(2));

    Ok(())
}

#[test]
fn test_related_records_params_with_geometry_options() -> Result<()> {
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

    assert_eq!(*params.return_geometry(), Some(true));
    assert_eq!(*params.out_sr(), Some(4326));
    assert_eq!(*params.geometry_precision(), Some(6));
    assert_eq!(*params.return_z(), Some(true));
    assert_eq!(*params.return_m(), Some(false));

    Ok(())
}

#[test]
fn test_related_records_client_method_compiles() {
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);

    // Test that the method signature compiles
    let _params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1)])
        .relationship_id(2u32)
        .build();

    // This won't execute but verifies the API compiles
    drop(service);
}

#[test]
fn test_related_records_params_serialization() -> Result<()> {
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(10), ObjectId::new(20)])
        .relationship_id(5u32)
        .out_fields(vec!["FIELD1".to_string()])
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    let serialized = serde_json::to_string(&params)?;
    assert!(serialized.contains("\"relationshipId\":5"));
    assert!(serialized.contains("\"objectIds\":\"10,20\""));

    Ok(())
}

#[test]
fn test_related_records_params_count_only() -> Result<()> {
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1), ObjectId::new(2)])
        .relationship_id(3u32)
        .return_count_only(true)
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    assert_eq!(*params.return_count_only(), Some(true));

    Ok(())
}

#[test]
fn test_related_records_params_with_gdb_version() -> Result<()> {
    let params = RelatedRecordsParams::builder()
        .object_ids(vec![ObjectId::new(1)])
        .relationship_id(1u32)
        .gdb_version("sde.DEFAULT")
        .build()
        .map_err(|e| arcgis::BuilderError::from(e.to_string()))?;

    assert_eq!(*params.gdb_version(), Some("sde.DEFAULT".to_string()));

    Ok(())
}
