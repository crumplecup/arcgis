//! Tests for Feature Service top features queries.

use arcgis::{
    ApiKeyAuth, ArcGISClient, FeatureServiceClient, TopFeaturesParams, TopFilter, Result,
};

#[test]
fn test_top_filter_creation() {
    let filter = TopFilter::new(
        vec!["State".to_string()],
        3,
        vec!["Population DESC".to_string()],
    );

    assert_eq!(filter.group_by_fields, vec!["State".to_string()]);
    assert_eq!(filter.top_count, 3);
    assert_eq!(filter.order_by_fields, vec!["Population DESC".to_string()]);
}

#[test]
fn test_top_filter_multiple_group_by_fields() {
    let filter = TopFilter::new(
        vec!["State".to_string(), "County".to_string()],
        5,
        vec!["Revenue DESC".to_string()],
    );

    assert_eq!(filter.group_by_fields.len(), 2);
    assert_eq!(filter.top_count, 5);
}

#[test]
fn test_top_filter_multiple_order_by_fields() {
    let filter = TopFilter::new(
        vec!["Category".to_string()],
        10,
        vec!["Score DESC".to_string(), "Name ASC".to_string()],
    );

    assert_eq!(filter.order_by_fields.len(), 2);
    assert_eq!(filter.order_by_fields[0], "Score DESC");
    assert_eq!(filter.order_by_fields[1], "Name ASC");
}

#[test]
fn test_top_features_params_builder() -> Result<()> {
    let filter = TopFilter::new(
        vec!["State".to_string()],
        3,
        vec!["Population DESC".to_string()],
    );

    let params = TopFeaturesParams::builder()
        .top_filter(filter.clone())
        .where_("Population > 100000")
        .out_fields(vec!["Name".to_string(), "Population".to_string()])
        .return_geometry(false)
        .build()
        .expect("Valid params");

    assert!(params.top_filter.is_some());
    assert_eq!(
        params.top_filter.unwrap().group_by_fields,
        vec!["State"]
    );
    assert_eq!(params.where_, Some("Population > 100000".to_string()));
    assert_eq!(params.out_fields.as_ref().map(|f| f.len()), Some(2));
    assert_eq!(params.return_geometry, Some(false));

    Ok(())
}

#[test]
fn test_top_features_params_default() {
    let params = TopFeaturesParams::default();

    assert!(params.top_filter.is_none());
    assert!(params.where_.is_none());
    assert_eq!(params.return_geometry, Some(true));
    assert_eq!(params.f, Some("json".to_string()));
}

#[test]
fn test_top_features_params_with_spatial_filter() -> Result<()> {
    let filter = TopFilter::new(
        vec!["Region".to_string()],
        5,
        vec!["Sales DESC".to_string()],
    );

    let params = TopFeaturesParams::builder()
        .top_filter(filter)
        .spatial_rel(arcgis::SpatialRel::Intersects)
        .distance(1000.0)
        .units("esriSRUnit_Meter")
        .build()
        .expect("Valid params");

    assert_eq!(
        params.spatial_rel,
        Some(arcgis::SpatialRel::Intersects)
    );
    assert_eq!(params.distance, Some(1000.0));
    assert_eq!(params.units, Some("esriSRUnit_Meter".to_string()));

    Ok(())
}

#[test]
fn test_top_features_params_with_geometry_options() -> Result<()> {
    let filter = TopFilter::new(
        vec!["Type".to_string()],
        10,
        vec!["Elevation DESC".to_string()],
    );

    let params = TopFeaturesParams::builder()
        .top_filter(filter)
        .return_geometry(true)
        .out_sr(4326)
        .geometry_precision(6)
        .return_z(true)
        .return_m(false)
        .build()
        .expect("Valid params");

    assert_eq!(params.return_geometry, Some(true));
    assert_eq!(params.out_sr, Some(4326));
    assert_eq!(params.geometry_precision, Some(6));
    assert_eq!(params.return_z, Some(true));
    assert_eq!(params.return_m, Some(false));

    Ok(())
}

#[test]
fn test_top_features_params_return_options() -> Result<()> {
    let filter = TopFilter::new(
        vec!["Category".to_string()],
        1,
        vec!["Value DESC".to_string()],
    );

    let params = TopFeaturesParams::builder()
        .top_filter(filter)
        .return_ids_only(true)
        .build()
        .expect("Valid params");

    assert_eq!(params.return_ids_only, Some(true));

    Ok(())
}

#[test]
fn test_top_features_client_method_compiles() {
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);

    // Test that the method signature compiles
    let filter = TopFilter::new(
        vec!["State".to_string()],
        3,
        vec!["Population DESC".to_string()],
    );

    let _params = TopFeaturesParams::builder()
        .top_filter(filter)
        .build();

    // This won't execute but verifies the API compiles
    drop(service);
}

#[test]
fn test_top_features_params_serialization() -> Result<()> {
    let filter = TopFilter::new(
        vec!["State".to_string()],
        5,
        vec!["Population DESC".to_string()],
    );

    let params = TopFeaturesParams::builder()
        .top_filter(filter)
        .where_("Population > 50000")
        .build()
        .expect("Valid params");

    let serialized = serde_json::to_string(&params)?;
    assert!(serialized.contains("\"topFilter\""));
    assert!(serialized.contains("\"groupByFields\""));
    assert!(serialized.contains("\"topCount\":5"));

    Ok(())
}

#[test]
fn test_top_filter_serialization() -> Result<()> {
    let filter = TopFilter::new(
        vec!["State".to_string(), "County".to_string()],
        3,
        vec!["Revenue DESC".to_string(), "Name ASC".to_string()],
    );

    let serialized = serde_json::to_string(&filter)?;
    assert!(serialized.contains("\"groupByFields\""));
    assert!(serialized.contains("\"State\""));
    assert!(serialized.contains("\"County\""));
    assert!(serialized.contains("\"topCount\":3"));
    assert!(serialized.contains("\"orderByFields\""));
    assert!(serialized.contains("\"Revenue DESC\""));
    assert!(serialized.contains("\"Name ASC\""));

    Ok(())
}

#[test]
fn test_top_features_params_with_time_filter() -> Result<()> {
    let filter = TopFilter::new(
        vec!["Year".to_string()],
        10,
        vec!["Sales DESC".to_string()],
    );

    let params = TopFeaturesParams::builder()
        .top_filter(filter)
        .time("1609459200000,1640995200000")
        .build()
        .expect("Valid params");

    assert_eq!(params.time, Some("1609459200000,1640995200000".to_string()));

    Ok(())
}
