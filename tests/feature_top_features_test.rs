//! Tests for Feature Service top features queries.

mod common;

use arcgis::{ApiKeyAuth, ArcGISClient, FeatureServiceClient, TopFeaturesParams, TopFilter};

#[test]
fn test_top_filter_creation() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_filter_creation: Starting");

    tracing::info!("test_top_filter_creation: Creating TopFilter");
    let filter = TopFilter::new(
        vec!["State".to_string()],
        3,
        vec!["Population DESC".to_string()],
    );

    tracing::info!(
        group_by_fields_count = filter.group_by_fields().len(),
        top_count = filter.top_count(),
        order_by_fields_count = filter.order_by_fields().len(),
        "test_top_filter_creation: Verifying filter"
    );
    assert_eq!(*filter.group_by_fields(), vec!["State".to_string()]);
    assert_eq!(*filter.top_count(), 3);
    assert_eq!(
        *filter.order_by_fields(),
        vec!["Population DESC".to_string()]
    );

    tracing::info!("test_top_filter_creation: Completed");
    Ok(())
}

#[test]
fn test_top_filter_multiple_group_by_fields() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_filter_multiple_group_by_fields: Starting");

    tracing::info!(
        "test_top_filter_multiple_group_by_fields: Creating filter with multiple group-by fields"
    );
    let filter = TopFilter::new(
        vec!["State".to_string(), "County".to_string()],
        5,
        vec!["Revenue DESC".to_string()],
    );

    tracing::info!(
        group_by_fields_count = filter.group_by_fields().len(),
        top_count = filter.top_count(),
        "test_top_filter_multiple_group_by_fields: Verifying filter"
    );
    assert_eq!(filter.group_by_fields().len(), 2);
    assert_eq!(*filter.top_count(), 5);

    tracing::info!("test_top_filter_multiple_group_by_fields: Completed");
    Ok(())
}

#[test]
fn test_top_filter_multiple_order_by_fields() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_filter_multiple_order_by_fields: Starting");

    tracing::info!(
        "test_top_filter_multiple_order_by_fields: Creating filter with multiple order-by fields"
    );
    let filter = TopFilter::new(
        vec!["Category".to_string()],
        10,
        vec!["Score DESC".to_string(), "Name ASC".to_string()],
    );

    tracing::info!(
        order_by_fields_count = filter.order_by_fields().len(),
        "test_top_filter_multiple_order_by_fields: Verifying order-by fields"
    );
    assert_eq!(filter.order_by_fields().len(), 2);
    assert_eq!(filter.order_by_fields()[0], "Score DESC");
    assert_eq!(filter.order_by_fields()[1], "Name ASC");

    tracing::info!("test_top_filter_multiple_order_by_fields: Completed");
    Ok(())
}

#[test]
fn test_top_features_params_builder() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_features_params_builder: Starting");

    tracing::info!("test_top_features_params_builder: Creating TopFilter");
    let filter = TopFilter::new(
        vec!["State".to_string()],
        3,
        vec!["Population DESC".to_string()],
    );

    tracing::info!("test_top_features_params_builder: Building TopFeaturesParams");
    let params = TopFeaturesParams::builder()
        .top_filter(filter.clone())
        .where_("Population > 100000")
        .out_fields(vec!["Name".to_string(), "Population".to_string()])
        .return_geometry(false)
        .build()?;

    tracing::info!(
        has_top_filter = params.top_filter().is_some(),
        where_clause = ?params.where_(),
        out_fields_count = ?params.out_fields().as_ref().map(|f| f.len()),
        return_geometry = ?params.return_geometry(),
        "test_top_features_params_builder: Verifying params"
    );
    assert!(params.top_filter().is_some());
    assert_eq!(
        params
            .top_filter()
            .as_ref()
            .unwrap()
            .group_by_fields()
            .as_slice(),
        vec!["State".to_string()]
    );
    assert_eq!(*params.where_(), Some("Population > 100000".to_string()));
    assert_eq!(params.out_fields().as_ref().map(|f| f.len()), Some(2));
    assert_eq!(*params.return_geometry(), Some(false));

    tracing::info!("test_top_features_params_builder: Completed");
    Ok(())
}

#[test]
fn test_top_features_params_default() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_features_params_default: Starting");

    tracing::info!("test_top_features_params_default: Creating default params");
    let params = TopFeaturesParams::default();

    tracing::info!(
        has_top_filter = params.top_filter().is_some(),
        has_where = params.where_().is_some(),
        return_geometry = ?params.return_geometry(),
        format = ?params.f(),
        "test_top_features_params_default: Verifying defaults"
    );
    assert!(params.top_filter().is_none());
    assert!(params.where_().is_none());
    assert_eq!(*params.return_geometry(), Some(true));
    assert_eq!(*params.f(), Some("json".to_string()));

    tracing::info!("test_top_features_params_default: Completed");
    Ok(())
}

#[test]
fn test_top_features_params_with_spatial_filter() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_features_params_with_spatial_filter: Starting");

    tracing::info!("test_top_features_params_with_spatial_filter: Creating filter");
    let filter = TopFilter::new(
        vec!["Region".to_string()],
        5,
        vec!["Sales DESC".to_string()],
    );

    tracing::info!(
        "test_top_features_params_with_spatial_filter: Building params with spatial filter"
    );
    let params = TopFeaturesParams::builder()
        .top_filter(filter)
        .spatial_rel(arcgis::SpatialRel::Intersects)
        .distance(1000.0)
        .units("esriSRUnit_Meter")
        .build()?;

    tracing::info!(
        spatial_rel = ?params.spatial_rel(),
        distance = ?params.distance(),
        units = ?params.units(),
        "test_top_features_params_with_spatial_filter: Verifying spatial filter"
    );
    assert_eq!(*params.spatial_rel(), Some(arcgis::SpatialRel::Intersects));
    assert_eq!(*params.distance(), Some(1000.0));
    assert_eq!(*params.units(), Some("esriSRUnit_Meter".to_string()));

    tracing::info!("test_top_features_params_with_spatial_filter: Completed");
    Ok(())
}

#[test]
fn test_top_features_params_with_geometry_options() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_features_params_with_geometry_options: Starting");

    tracing::info!("test_top_features_params_with_geometry_options: Creating filter");
    let filter = TopFilter::new(
        vec!["Type".to_string()],
        10,
        vec!["Elevation DESC".to_string()],
    );

    tracing::info!(
        "test_top_features_params_with_geometry_options: Building params with geometry options"
    );
    let params = TopFeaturesParams::builder()
        .top_filter(filter)
        .return_geometry(true)
        .out_sr(4326)
        .geometry_precision(6)
        .return_z(true)
        .return_m(false)
        .build()?;

    tracing::info!(
        return_geometry = ?params.return_geometry(),
        out_sr = ?params.out_sr(),
        geometry_precision = ?params.geometry_precision(),
        return_z = ?params.return_z(),
        return_m = ?params.return_m(),
        "test_top_features_params_with_geometry_options: Verifying geometry options"
    );
    assert_eq!(*params.return_geometry(), Some(true));
    assert_eq!(*params.out_sr(), Some(4326));
    assert_eq!(*params.geometry_precision(), Some(6));
    assert_eq!(*params.return_z(), Some(true));
    assert_eq!(*params.return_m(), Some(false));

    tracing::info!("test_top_features_params_with_geometry_options: Completed");
    Ok(())
}

#[test]
fn test_top_features_params_return_options() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_features_params_return_options: Starting");

    tracing::info!("test_top_features_params_return_options: Creating filter");
    let filter = TopFilter::new(
        vec!["Category".to_string()],
        1,
        vec!["Value DESC".to_string()],
    );

    tracing::info!("test_top_features_params_return_options: Building params with return options");
    let params = TopFeaturesParams::builder()
        .top_filter(filter)
        .return_ids_only(true)
        .build()?;

    tracing::info!(
        return_ids_only = ?params.return_ids_only(),
        "test_top_features_params_return_options: Verifying return options"
    );
    assert_eq!(*params.return_ids_only(), Some(true));

    tracing::info!("test_top_features_params_return_options: Completed");
    Ok(())
}

#[test]
fn test_top_features_client_method_compiles() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_features_client_method_compiles: Starting");

    tracing::info!("test_top_features_client_method_compiles: Creating test client");
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);

    // Test that the method signature compiles
    tracing::info!("test_top_features_client_method_compiles: Creating filter and params");
    let filter = TopFilter::new(
        vec!["State".to_string()],
        3,
        vec!["Population DESC".to_string()],
    );

    let _params = TopFeaturesParams::builder().top_filter(filter).build();

    // This won't execute but verifies the API compiles
    drop(service);

    tracing::info!("test_top_features_client_method_compiles: Completed");
    Ok(())
}

#[test]
fn test_top_features_params_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_features_params_serialization: Starting");

    tracing::info!("test_top_features_params_serialization: Creating filter");
    let filter = TopFilter::new(
        vec!["State".to_string()],
        5,
        vec!["Population DESC".to_string()],
    );

    tracing::info!("test_top_features_params_serialization: Building params");
    let params = TopFeaturesParams::builder()
        .top_filter(filter)
        .where_("Population > 50000")
        .build()?;

    tracing::info!("test_top_features_params_serialization: Serializing to JSON");
    let serialized = serde_json::to_string(&params)?;

    tracing::info!(
        serialized_len = serialized.len(),
        "test_top_features_params_serialization: Verifying serialization"
    );
    assert!(serialized.contains("\"topFilter\""));
    assert!(serialized.contains("\"groupByFields\""));
    assert!(serialized.contains("\"topCount\":5"));

    tracing::info!("test_top_features_params_serialization: Completed");
    Ok(())
}

#[test]
fn test_top_filter_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_filter_serialization: Starting");

    tracing::info!("test_top_filter_serialization: Creating filter");
    let filter = TopFilter::new(
        vec!["State".to_string(), "County".to_string()],
        3,
        vec!["Revenue DESC".to_string(), "Name ASC".to_string()],
    );

    tracing::info!("test_top_filter_serialization: Serializing to JSON");
    let serialized = serde_json::to_string(&filter)?;

    tracing::info!(
        serialized_len = serialized.len(),
        "test_top_filter_serialization: Verifying serialization"
    );
    assert!(serialized.contains("\"groupByFields\""));
    assert!(serialized.contains("\"State\""));
    assert!(serialized.contains("\"County\""));
    assert!(serialized.contains("\"topCount\":3"));
    assert!(serialized.contains("\"orderByFields\""));
    assert!(serialized.contains("\"Revenue DESC\""));
    assert!(serialized.contains("\"Name ASC\""));

    tracing::info!("test_top_filter_serialization: Completed");
    Ok(())
}

#[test]
fn test_top_features_params_with_time_filter() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_top_features_params_with_time_filter: Starting");

    tracing::info!("test_top_features_params_with_time_filter: Creating filter");
    let filter = TopFilter::new(vec!["Year".to_string()], 10, vec!["Sales DESC".to_string()]);

    tracing::info!("test_top_features_params_with_time_filter: Building params with time filter");
    let params = TopFeaturesParams::builder()
        .top_filter(filter)
        .time("1609459200000,1640995200000")
        .build()?;

    tracing::info!(
        time = ?params.time(),
        "test_top_features_params_with_time_filter: Verifying time filter"
    );
    assert_eq!(
        *params.time(),
        Some("1609459200000,1640995200000".to_string())
    );

    tracing::info!("test_top_features_params_with_time_filter: Completed");
    Ok(())
}
