//! Tests for Feature Service statistics queries.

mod common;

use arcgis::{
    ApiKeyAuth, ArcGISClient, FeatureServiceClient, LayerId, StatisticDefinition, StatisticType,
};

#[test]
fn test_statistic_type_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_statistic_type_serialization: Starting");

    tracing::info!("test_statistic_type_serialization: Testing Count serialization");
    let count = StatisticType::Count;
    let serialized = serde_json::to_string(&count)?;
    assert_eq!(serialized, "\"count\"");

    tracing::info!("test_statistic_type_serialization: Testing Sum serialization");
    let sum = StatisticType::Sum;
    let serialized = serde_json::to_string(&sum)?;
    assert_eq!(serialized, "\"sum\"");

    tracing::info!("test_statistic_type_serialization: Testing Avg serialization");
    let avg = StatisticType::Avg;
    let serialized = serde_json::to_string(&avg)?;
    assert_eq!(serialized, "\"avg\"");

    tracing::info!("test_statistic_type_serialization: Testing Min serialization");
    let min = StatisticType::Min;
    let serialized = serde_json::to_string(&min)?;
    assert_eq!(serialized, "\"min\"");

    tracing::info!("test_statistic_type_serialization: Testing Max serialization");
    let max = StatisticType::Max;
    let serialized = serde_json::to_string(&max)?;
    assert_eq!(serialized, "\"max\"");

    tracing::info!("test_statistic_type_serialization: Testing Stddev serialization");
    let stddev = StatisticType::Stddev;
    let serialized = serde_json::to_string(&stddev)?;
    assert_eq!(serialized, "\"stddev\"");

    tracing::info!("test_statistic_type_serialization: Testing Var serialization");
    let var = StatisticType::Var;
    let serialized = serde_json::to_string(&var)?;
    assert_eq!(serialized, "\"var\"");

    tracing::info!("test_statistic_type_serialization: Testing PercentileCont serialization");
    let percentile_cont = StatisticType::PercentileCont;
    let serialized = serde_json::to_string(&percentile_cont)?;
    assert_eq!(serialized, "\"PERCENTILE_CONT\"");

    tracing::info!("test_statistic_type_serialization: Testing PercentileDisc serialization");
    let percentile_disc = StatisticType::PercentileDisc;
    let serialized = serde_json::to_string(&percentile_disc)?;
    assert_eq!(serialized, "\"PERCENTILE_DISC\"");

    tracing::info!("test_statistic_type_serialization: Completed");
    Ok(())
}

#[test]
fn test_statistic_definition_creation() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_statistic_definition_creation: Starting");

    tracing::info!("test_statistic_definition_creation: Creating StatisticDefinition");
    let stat = StatisticDefinition::new(
        StatisticType::Avg,
        "POPULATION".to_string(),
        "avg_population".to_string(),
    );

    tracing::info!(
        statistic_type = ?stat.statistic_type(),
        on_field = %stat.on_statistic_field(),
        out_field = %stat.out_statistic_field_name(),
        "test_statistic_definition_creation: Verifying definition"
    );
    assert_eq!(*stat.statistic_type(), StatisticType::Avg);
    assert_eq!(*stat.on_statistic_field(), "POPULATION");
    assert_eq!(*stat.out_statistic_field_name(), "avg_population");

    tracing::info!("test_statistic_definition_creation: Completed");
    Ok(())
}

#[test]
fn test_statistic_definition_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_statistic_definition_serialization: Starting");

    tracing::info!("test_statistic_definition_serialization: Creating StatisticDefinition");
    let stat = StatisticDefinition::new(
        StatisticType::Sum,
        "AREA".to_string(),
        "total_area".to_string(),
    );

    tracing::info!("test_statistic_definition_serialization: Serializing to JSON");
    let json = serde_json::to_string(&stat)?;

    tracing::info!(
        json_len = json.len(),
        "test_statistic_definition_serialization: Verifying JSON content"
    );
    assert!(json.contains("\"statisticType\":\"sum\""));
    assert!(json.contains("\"onStatisticField\":\"AREA\""));
    assert!(json.contains("\"outStatisticFieldName\":\"total_area\""));

    tracing::info!("test_statistic_definition_serialization: Completed");
    Ok(())
}

#[test]
fn test_statistic_definition_deserialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_statistic_definition_deserialization: Starting");

    let json = r#"{
        "statisticType": "count",
        "onStatisticField": "OBJECTID",
        "outStatisticFieldName": "total_count"
    }"#;

    tracing::info!("test_statistic_definition_deserialization: Deserializing from JSON");
    let stat: StatisticDefinition = serde_json::from_str(json)?;

    tracing::info!(
        statistic_type = ?stat.statistic_type(),
        on_field = %stat.on_statistic_field(),
        out_field = %stat.out_statistic_field_name(),
        "test_statistic_definition_deserialization: Verifying deserialized data"
    );
    assert_eq!(*stat.statistic_type(), StatisticType::Count);
    assert_eq!(*stat.on_statistic_field(), "OBJECTID");
    assert_eq!(*stat.out_statistic_field_name(), "total_count");

    tracing::info!("test_statistic_definition_deserialization: Completed");
    Ok(())
}

#[test]
fn test_query_builder_statistics() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_query_builder_statistics: Starting");

    tracing::info!("test_query_builder_statistics: Creating test client");
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);

    // Test that the builder pattern compiles
    tracing::info!("test_query_builder_statistics: Building query with statistics");
    let builder = service
        .query(LayerId::new(0))
        .statistics(vec![
            StatisticDefinition::new(
                StatisticType::Count,
                "OBJECTID".to_string(),
                "total".to_string(),
            ),
            StatisticDefinition::new(
                StatisticType::Sum,
                "POPULATION".to_string(),
                "total_pop".to_string(),
            ),
        ])
        .group_by(&["STATE", "COUNTY"])
        .having("total > 100");

    drop(builder);

    tracing::info!("test_query_builder_statistics: Completed");
    Ok(())
}

#[test]
fn test_query_builder_statistics_with_order_by() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_query_builder_statistics_with_order_by: Starting");

    tracing::info!("test_query_builder_statistics_with_order_by: Creating test client");
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);

    // Test statistics with ordering
    tracing::info!(
        "test_query_builder_statistics_with_order_by: Building query with statistics and ordering"
    );
    let builder = service
        .query(LayerId::new(0))
        .statistics(vec![StatisticDefinition::new(
            StatisticType::Avg,
            "AREA".to_string(),
            "avg_area".to_string(),
        )])
        .group_by(&["REGION"])
        .order_by(&["avg_area DESC"]);

    drop(builder);

    tracing::info!("test_query_builder_statistics_with_order_by: Completed");
    Ok(())
}

#[test]
fn test_multiple_statistics_types() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_multiple_statistics_types: Starting");

    tracing::info!("test_multiple_statistics_types: Creating multiple statistic definitions");
    let stats = [
        StatisticDefinition::new(
            StatisticType::Count,
            "OBJECTID".to_string(),
            "count".to_string(),
        ),
        StatisticDefinition::new(
            StatisticType::Sum,
            "REVENUE".to_string(),
            "total_revenue".to_string(),
        ),
        StatisticDefinition::new(
            StatisticType::Avg,
            "REVENUE".to_string(),
            "avg_revenue".to_string(),
        ),
        StatisticDefinition::new(
            StatisticType::Min,
            "REVENUE".to_string(),
            "min_revenue".to_string(),
        ),
        StatisticDefinition::new(
            StatisticType::Max,
            "REVENUE".to_string(),
            "max_revenue".to_string(),
        ),
        StatisticDefinition::new(
            StatisticType::Stddev,
            "REVENUE".to_string(),
            "stddev_revenue".to_string(),
        ),
    ];

    tracing::info!(
        stats_count = stats.len(),
        "test_multiple_statistics_types: Verifying multiple statistics"
    );
    assert_eq!(stats.len(), 6);
    assert_eq!(*stats[0].statistic_type(), StatisticType::Count);
    assert_eq!(*stats[1].statistic_type(), StatisticType::Sum);
    assert_eq!(*stats[2].statistic_type(), StatisticType::Avg);
    assert_eq!(*stats[3].statistic_type(), StatisticType::Min);
    assert_eq!(*stats[4].statistic_type(), StatisticType::Max);
    assert_eq!(*stats[5].statistic_type(), StatisticType::Stddev);

    tracing::info!("test_multiple_statistics_types: Completed");
    Ok(())
}

#[test]
fn test_having_clause_compilation() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_having_clause_compilation: Starting");

    tracing::info!("test_having_clause_compilation: Creating test client");
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);

    // Test various HAVING clause formats
    tracing::info!("test_having_clause_compilation: Testing HAVING clause format 1 (cnt > 10)");
    let _builder1 = service
        .query(LayerId::new(0))
        .statistics(vec![StatisticDefinition::new(
            StatisticType::Count,
            "ID".to_string(),
            "cnt".to_string(),
        )])
        .group_by(&["CATEGORY"])
        .having("cnt > 10");

    tracing::info!(
        "test_having_clause_compilation: Testing HAVING clause format 2 (total >= 1000)"
    );
    let _builder2 = service
        .query(LayerId::new(0))
        .statistics(vec![StatisticDefinition::new(
            StatisticType::Sum,
            "AMOUNT".to_string(),
            "total".to_string(),
        )])
        .group_by(&["REGION"])
        .having("total >= 1000");

    tracing::info!("test_having_clause_compilation: Testing HAVING clause format 3 (BETWEEN)");
    let _builder3 = service
        .query(LayerId::new(0))
        .statistics(vec![StatisticDefinition::new(
            StatisticType::Avg,
            "SCORE".to_string(),
            "avg_score".to_string(),
        )])
        .group_by(&["DIVISION"])
        .having("avg_score BETWEEN 70 AND 90");

    tracing::info!("test_having_clause_compilation: Completed");
    Ok(())
}
