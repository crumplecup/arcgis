//! Tests for Feature Service statistics queries.

use arcgis::{
    ApiKeyAuth, ArcGISClient, FeatureServiceClient, LayerId, Result, StatisticDefinition,
    StatisticType,
};

#[test]
fn test_statistic_type_serialization() -> Result<()> {
    let count = StatisticType::Count;
    let serialized = serde_json::to_string(&count)?;
    assert_eq!(serialized, "\"count\"");

    let sum = StatisticType::Sum;
    let serialized = serde_json::to_string(&sum)?;
    assert_eq!(serialized, "\"sum\"");

    let avg = StatisticType::Avg;
    let serialized = serde_json::to_string(&avg)?;
    assert_eq!(serialized, "\"avg\"");

    let min = StatisticType::Min;
    let serialized = serde_json::to_string(&min)?;
    assert_eq!(serialized, "\"min\"");

    let max = StatisticType::Max;
    let serialized = serde_json::to_string(&max)?;
    assert_eq!(serialized, "\"max\"");

    let stddev = StatisticType::Stddev;
    let serialized = serde_json::to_string(&stddev)?;
    assert_eq!(serialized, "\"stddev\"");

    let var = StatisticType::Var;
    let serialized = serde_json::to_string(&var)?;
    assert_eq!(serialized, "\"var\"");

    let percentile_cont = StatisticType::PercentileCont;
    let serialized = serde_json::to_string(&percentile_cont)?;
    assert_eq!(serialized, "\"PERCENTILE_CONT\"");

    let percentile_disc = StatisticType::PercentileDisc;
    let serialized = serde_json::to_string(&percentile_disc)?;
    assert_eq!(serialized, "\"PERCENTILE_DISC\"");

    Ok(())
}

#[test]
fn test_statistic_definition_creation() {
    let stat = StatisticDefinition::new(
        StatisticType::Avg,
        "POPULATION".to_string(),
        "avg_population".to_string(),
    );

    assert_eq!(*stat.statistic_type(), StatisticType::Avg);
    assert_eq!(*stat.on_statistic_field(), "POPULATION");
    assert_eq!(*stat.out_statistic_field_name(), "avg_population");
}

#[test]
fn test_statistic_definition_serialization() -> Result<()> {
    let stat = StatisticDefinition::new(
        StatisticType::Sum,
        "AREA".to_string(),
        "total_area".to_string(),
    );

    let json = serde_json::to_string(&stat)?;
    assert!(json.contains("\"statisticType\":\"sum\""));
    assert!(json.contains("\"onStatisticField\":\"AREA\""));
    assert!(json.contains("\"outStatisticFieldName\":\"total_area\""));

    Ok(())
}

#[test]
fn test_statistic_definition_deserialization() -> Result<()> {
    let json = r#"{
        "statisticType": "count",
        "onStatisticField": "OBJECTID",
        "outStatisticFieldName": "total_count"
    }"#;

    let stat: StatisticDefinition = serde_json::from_str(json)?;
    assert_eq!(*stat.statistic_type(), StatisticType::Count);
    assert_eq!(*stat.on_statistic_field(), "OBJECTID");
    assert_eq!(*stat.out_statistic_field_name(), "total_count");

    Ok(())
}

#[test]
fn test_query_builder_statistics() {
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);

    // Test that the builder pattern compiles
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
}

#[test]
fn test_query_builder_statistics_with_order_by() {
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);

    // Test statistics with ordering
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
}

#[test]
fn test_multiple_statistics_types() {
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

    assert_eq!(stats.len(), 6);
    assert_eq!(*stats[0].statistic_type(), StatisticType::Count);
    assert_eq!(*stats[1].statistic_type(), StatisticType::Sum);
    assert_eq!(*stats[2].statistic_type(), StatisticType::Avg);
    assert_eq!(*stats[3].statistic_type(), StatisticType::Min);
    assert_eq!(*stats[4].statistic_type(), StatisticType::Max);
    assert_eq!(*stats[5].statistic_type(), StatisticType::Stddev);
}

#[test]
fn test_having_clause_compilation() {
    let auth = ApiKeyAuth::new("test_key");
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);

    // Test various HAVING clause formats
    let _builder1 = service
        .query(LayerId::new(0))
        .statistics(vec![StatisticDefinition::new(
            StatisticType::Count,
            "ID".to_string(),
            "cnt".to_string(),
        )])
        .group_by(&["CATEGORY"])
        .having("cnt > 10");

    let _builder2 = service
        .query(LayerId::new(0))
        .statistics(vec![StatisticDefinition::new(
            StatisticType::Sum,
            "AMOUNT".to_string(),
            "total".to_string(),
        )])
        .group_by(&["REGION"])
        .having("total >= 1000");

    let _builder3 = service
        .query(LayerId::new(0))
        .statistics(vec![StatisticDefinition::new(
            StatisticType::Avg,
            "SCORE".to_string(),
            "avg_score".to_string(),
        )])
        .group_by(&["DIVISION"])
        .having("avg_score BETWEEN 70 AND 90");
}
