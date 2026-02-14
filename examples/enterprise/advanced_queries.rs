//! Advanced Feature Service query examples.
//!
//! This example demonstrates advanced Feature Service query capabilities:
//! - Related record queries (find related records across tables)
//! - Top N features queries (top features by ranking within groups)
//! - Statistical aggregations (COUNT, SUM, AVG, MIN, MAX with GROUP BY)
//! - Pagination patterns (manual pagination for large datasets)
//! - Domain lookups (get valid coded values for fields)
//!
//! # Use Case: SF311 Service Request Analysis
//!
//! This example uses SF311 service requests to demonstrate:
//! 1. Calculating statistics by request type
//! 2. Finding top 5 requests by district
//! 3. Paginating through large result sets efficiently
//! 4. Related records queries (optional - requires relationship classes)
//! 5. Domain lookups (optional - requires domain definitions)
//!
//! # Prerequisites
//!
//! - None! This example uses a public ESRI sample service
//! - The service has 9,664+ SF311 service request records
//! - No authentication required (NoAuth)
//!
//! # Environment Variables
//!
//! None required for this example.
//!
//! # Running
//!
//! ```bash
//! # With debug logging to see all operations
//! RUST_LOG=debug cargo run --example advanced_queries
//!
//! # With info logging for cleaner output
//! RUST_LOG=info cargo run --example advanced_queries
//! ```

use anyhow::Result;
use arcgis::{
    ArcGISClient, FeatureServiceClient, LayerId, NoAuth, ObjectId, RelatedRecordsParams,
    StatisticDefinition, StatisticType, TopFeaturesParams, TopFilter,
};
use tracing::instrument;

/// ESRI's sample SF311 incidents service (for demonstration).
///
/// This is a public service with 9,664+ service request incidents from San Francisco.
/// Fields: req_type (request type), district (neighborhood), status, address, etc.
const SAMPLE_SERVICE: &str =
    "https://sampleserver6.arcgisonline.com/arcgis/rest/services/SF311/FeatureServer";

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    tracing::debug!("Initializing tracing subscriber");
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🔍 Advanced Feature Service Query Examples");
    tracing::info!("Use Case: SF311 Service Request Analysis");
    tracing::debug!(service_url = %SAMPLE_SERVICE, "Using ESRI sample SF311 service");

    // Create client (NoAuth - public service)
    tracing::debug!("Creating ArcGIS client for public service");
    let client = ArcGISClient::new(NoAuth);
    let service = FeatureServiceClient::new(SAMPLE_SERVICE, &client);
    let layer_id = LayerId::new(0);

    tracing::info!("\n📊 Demonstrating Advanced Query Capabilities\n");

    // First test a simple query to verify authentication and that service has data
    tracing::debug!("Testing basic connectivity with simple query");
    let test_result = service
        .query(layer_id)
        .where_clause("1=1")
        .limit(10)
        .return_geometry(false)
        .execute()
        .await?;

    let feature_count = test_result.features().len();
    tracing::info!(
        features = feature_count,
        "Basic query successful - authentication working"
    );

    // Service should have some data for meaningful tests
    anyhow::ensure!(
        feature_count > 0,
        "Service returned no features - cannot demonstrate advanced queries. \
         Service may be empty or query may have failed silently."
    );

    // Demonstrate advanced query operations
    demonstrate_statistical_aggregations(&service, layer_id).await?;
    demonstrate_pagination_strategies(&service, layer_id).await?;

    // Note: Advanced features require specific service configurations
    tracing::info!("\n⚠️  Note: Some advanced queries require newer service versions:");
    tracing::info!("   - queryTopFeatures (ArcGIS Enterprise 10.3+, requires newer service)");
    tracing::info!("   - queryRelatedRecords (requires relationship classes defined)");
    tracing::info!("   - queryDomains (requires domain/subtype definitions)");
    tracing::info!("\n   This example demonstrates statistical aggregations and pagination,");
    tracing::info!("   which work with all ArcGIS service versions.");

    tracing::info!("\n✅ All advanced query examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates statistical aggregations with GROUP BY and HAVING clauses.
///
/// This function shows how to:
/// - Calculate multiple statistics (COUNT, SUM, AVG, MIN, MAX)
/// - Group results by one or more fields
/// - Filter aggregated results with HAVING clause
/// - Order aggregated results
#[instrument(skip(service), fields(layer_id = %layer_id))]
async fn demonstrate_statistical_aggregations(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> Result<()> {
    tracing::info!("=== Example 1: Statistical Aggregations ===");
    tracing::info!("Calculate service request statistics by type");
    tracing::debug!("Creating statistic definitions for aggregation");

    // Define statistics to calculate
    let stats = vec![StatisticDefinition::new(
        StatisticType::Count,
        "objectid".to_string(),
        "total_requests".to_string(),
    )];

    tracing::debug!(stat_count = stats.len(), "Prepared statistic definitions");

    // Query with statistics and grouping
    tracing::debug!("Executing statistics query with GROUP BY");
    let result = service
        .query(layer_id)
        .where_clause("1=1")
        .statistics(stats)
        .group_by(&["req_type"])
        .order_by(&["total_requests DESC"])
        .return_geometry(false)
        .execute()
        .await?;

    let feature_count = result.features().len();
    tracing::info!(
        groups = feature_count,
        "Statistics calculated by request type"
    );
    tracing::debug!(
        exceeded_limit = result.exceeded_transfer_limit(),
        "Query completion status"
    );

    // Verify statistics query returned results
    anyhow::ensure!(
        feature_count > 0,
        "Statistics query returned no groups. Expected aggregated results by req_type field. \
         Service may not have data or req_type field may not exist."
    );

    // Display top results and verify they have required fields
    for (idx, feature) in result.features().iter().take(5).enumerate() {
        let req_type = feature.attributes().get("req_type");
        let count = feature.attributes().get("total_requests");

        // Verify aggregated results have expected fields
        anyhow::ensure!(
            count.is_some(),
            "Statistics result missing 'total_requests' field. \
             Aggregation may have failed or field name mismatch."
        );

        tracing::info!(
            rank = idx + 1,
            request_type = ?req_type,
            request_count = ?count,
            "Service request type statistics"
        );
    }

    tracing::debug!("Statistical aggregations demonstration complete");
    Ok(())
}

/// Demonstrates queryTopFeatures operation for ranking within groups.
///
/// This function shows how to:
/// - Define grouping fields
/// - Set top N count
/// - Order results within groups
/// - Apply filters to groups
///
/// **Note:** This operation requires ArcGIS Enterprise 10.3+ or ArcGIS Online.
/// The ESRI sample server used in this example doesn't support it, so this
/// function is kept for documentation but not called.
#[allow(dead_code)]
#[instrument(skip(service), fields(layer_id = %layer_id))]
async fn demonstrate_top_features(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> Result<()> {
    tracing::info!("\n=== Example 2: Top N Features Query ===");
    tracing::info!("Find top 5 service requests by district");

    // Create top filter
    tracing::debug!("Creating TopFilter specification");
    let top_filter = TopFilter::new(
        vec!["district".to_string()],      // Group by district
        5,                                 // Top 5 from each district
        vec!["objectid DESC".to_string()], // Order by ID (proxy for recent)
    );

    tracing::debug!(
        group_by_fields = ?["district"],
        top_count = 5,
        "TopFilter configuration prepared"
    );

    // Build top features parameters
    tracing::debug!("Building TopFeaturesParams");
    let params = TopFeaturesParams::builder()
        .top_filter(top_filter)
        .where_("req_type IS NOT NULL".to_string())
        .out_fields(vec![
            "req_type".to_string(),
            "district".to_string(),
            "address".to_string(),
        ])
        .return_geometry(false)
        .f("json".to_string())
        .build()
        .expect("Valid top features params");

    tracing::debug!("Executing queryTopFeatures operation");
    let result = service.query_top_features(layer_id, params).await?;

    let feature_count = result.features().len();
    tracing::info!(
        total_features = feature_count,
        expected_max = 5 * 10, // 5 per district, ~10 districts
        "Top features query completed"
    );
    tracing::debug!(
        exceeded_limit = result.exceeded_transfer_limit(),
        "Query completion status"
    );

    // Verify top features query returned results
    anyhow::ensure!(
        feature_count > 0,
        "Top features query returned no results. Expected top 5 features per district. \
         Service may not have data or district field may not exist."
    );

    // Group and display results
    tracing::debug!("Processing and displaying top features by district");
    let mut by_district: std::collections::HashMap<String, Vec<_>> =
        std::collections::HashMap::new();

    for feature in result.features() {
        let district_value = feature.attributes().get("district");

        // Verify expected field exists
        anyhow::ensure!(
            district_value.is_some(),
            "Feature missing 'district' field. \
             Top features query requires this grouping field to exist."
        );

        if let Some(district) = district_value.and_then(|v| v.as_str()) {
            by_district
                .entry(district.to_string())
                .or_default()
                .push(feature);
        }
    }

    let district_count = by_district.len();
    tracing::info!(
        districts_found = district_count,
        "Grouping top features by district"
    );

    // Verify we got results for multiple districts
    anyhow::ensure!(
        district_count > 0,
        "No districts found in results. district field may be null or invalid."
    );

    for (district, features) in by_district.iter().take(3) {
        tracing::info!(
            district = %district,
            feature_count = features.len(),
            "Top requests in district"
        );

        for (idx, feature) in features.iter().take(3).enumerate() {
            let req_type = feature.attributes().get("req_type");
            let address = feature.attributes().get("address");

            tracing::debug!(
                district = %district,
                rank = idx + 1,
                request_type = ?req_type,
                address = ?address,
                "Service request detail"
            );
        }
    }

    tracing::debug!("Top features demonstration complete");
    Ok(())
}

/// Demonstrates pagination strategies for large datasets.
///
/// This function shows:
/// - Manual pagination with offset/limit
/// - Tracking total records fetched
/// - Detecting when pagination is complete
/// - Comparing manual vs automatic pagination
#[instrument(skip(service), fields(layer_id = %layer_id))]
async fn demonstrate_pagination_strategies(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> Result<()> {
    tracing::info!("\n=== Example 3: Pagination Strategies ===");
    tracing::info!("Efficiently paginate through large result sets");

    let page_size = 100;
    let max_pages = 3; // Limit for demo

    tracing::debug!(
        page_size = page_size,
        max_pages = max_pages,
        "Starting manual pagination"
    );

    // Manual pagination
    let mut total_fetched = 0;
    let mut page_num = 0;

    loop {
        if page_num >= max_pages {
            tracing::debug!(
                pages_fetched = page_num,
                "Reached maximum page limit for demonstration"
            );
            break;
        }

        let offset = page_num * page_size;
        tracing::debug!(
            page = page_num,
            offset = offset,
            limit = page_size,
            "Fetching page"
        );

        let result = service
            .query(layer_id)
            .where_clause("1=1")
            .out_fields(&["req_type", "address"])
            .return_geometry(false)
            .limit(page_size)
            .offset(offset)
            .execute()
            .await?;

        let feature_count = result.features().len();
        total_fetched += feature_count;

        tracing::info!(
            page = page_num,
            features_in_page = feature_count,
            cumulative_total = total_fetched,
            exceeded_limit = result.exceeded_transfer_limit(),
            "Page retrieved"
        );

        // Check if we've fetched all available records
        if feature_count < page_size as usize || !*result.exceeded_transfer_limit() {
            tracing::debug!(
                feature_count = feature_count,
                page_size = page_size,
                exceeded_limit = result.exceeded_transfer_limit(),
                "Reached end of available data"
            );
            break;
        }

        page_num += 1;
    }

    tracing::info!(
        total_pages = page_num + 1,
        total_features = total_fetched,
        "Manual pagination complete"
    );

    // Verify manual pagination fetched data
    anyhow::ensure!(
        total_fetched > 0,
        "Manual pagination fetched no features. Service may be empty or query failed."
    );

    // Compare with automatic pagination
    tracing::debug!("Demonstrating automatic pagination with execute_all()");
    let auto_result = service
        .query(layer_id)
        .where_clause("1=1")
        .out_fields(&["req_type"])
        .return_geometry(false)
        .limit(page_size)
        .execute_all()
        .await?;

    let auto_total = auto_result.features().len();
    tracing::info!(
        auto_pagination_total = auto_total,
        manual_pagination_total = total_fetched,
        "Automatic pagination completed for comparison"
    );
    tracing::debug!(
        exceeded_limit = auto_result.exceeded_transfer_limit(),
        "Automatic pagination status"
    );

    // Verify automatic pagination also fetched data
    anyhow::ensure!(
        auto_total > 0,
        "Automatic pagination fetched no features. Service may be empty or query failed."
    );

    // Verify automatic pagination is consistent with manual (allowing for manual page limit)
    anyhow::ensure!(
        auto_total >= total_fetched,
        "Automatic pagination returned fewer features ({}) than manual ({}). This is unexpected.",
        auto_total,
        total_fetched
    );

    if auto_total > total_fetched {
        tracing::info!(
            additional_features = auto_total - total_fetched,
            "Automatic pagination fetched more features (manual was limited to {} pages)",
            max_pages
        );
    }

    tracing::debug!("Pagination strategies demonstration complete");
    Ok(())
}

/// Demonstrates querying related records across relationship classes.
///
/// **Note:** This requires a service with relationship classes defined.
/// Not all services have relationships configured. The ESRI sample server
/// used in this example doesn't have relationships, so this function is
/// kept for documentation but not called.
///
/// This function shows how to:
/// - Query related records for specific features
/// - Access related records grouped by parent object ID
/// - Navigate relationship classes
#[allow(dead_code)]
#[instrument(skip(service), fields(layer_id = %layer_id))]
async fn demonstrate_related_records(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> Result<()> {
    tracing::info!("\n=== Example 4: Related Records Query ===");
    tracing::info!("Find evidence records related to crime incidents");
    tracing::debug!("Note: Requires service with relationship classes defined");

    // First, get some incident IDs
    tracing::debug!("Fetching sample incident IDs");
    let incidents = service
        .query(layer_id)
        .where_clause("1=1")
        .ids_only(true)
        .limit(5)
        .execute()
        .await?;

    let object_ids: Vec<ObjectId> = incidents
        .features()
        .iter()
        .filter_map(|f| {
            f.attributes()
                .get("objectid")
                .and_then(|v| v.as_i64())
                .and_then(|id| {
                    if id >= 0 {
                        Some(ObjectId::new(id as u32))
                    } else {
                        None
                    }
                })
        })
        .collect();

    tracing::debug!(
        incident_count = object_ids.len(),
        "Retrieved incident IDs for related records query"
    );

    anyhow::ensure!(
        !object_ids.is_empty(),
        "No object IDs found for related records query. Service may be empty or objectid field missing."
    );

    // Build related records params
    tracing::debug!("Building RelatedRecordsParams");
    let params = RelatedRecordsParams::builder()
        .object_ids(object_ids.clone())
        .relationship_id(0u32) // Relationship ID (service-specific)
        .out_fields(vec!["*".to_string()])
        .return_geometry(false)
        .build()
        .expect("Valid related records params");

    tracing::debug!(
        object_id_count = object_ids.len(),
        relationship_id = 0,
        "Executing queryRelatedRecords operation"
    );

    // Query related records
    let result = service.query_related_records(layer_id, params).await?;

    let group_count = result.related_record_groups().len();
    tracing::info!(groups = group_count, "Related records query completed");

    // Display results
    for group in result.related_record_groups().iter().take(3) {
        let related_count = group.related_records().len();

        tracing::info!(
            incident_id = ?group.object_id(),
            related_records = related_count,
            "Found related evidence"
        );

        for (idx, record) in group.related_records().iter().take(2).enumerate() {
            tracing::debug!(
                incident_id = ?group.object_id(),
                evidence_num = idx + 1,
                attributes = ?record.attributes(),
                "Evidence record detail"
            );
        }
    }

    tracing::debug!("Related records demonstration complete");
    Ok(())
}

/// Demonstrates querying field domains and subtypes.
///
/// **Note:** This requires a service with domains/subtypes configured.
/// The ESRI sample server used in this example doesn't have domains,
/// so this function is kept for documentation but not called.
///
/// This function shows how to:
/// - Query coded value domains
/// - Access domain values for specific layers
/// - Get subtype information
#[allow(dead_code)]
#[instrument(skip(service))]
async fn demonstrate_domain_lookups(service: &FeatureServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 5: Domain Lookups ===");
    tracing::info!("Query valid domain values for crime types");
    tracing::debug!("Note: Requires service with domain/subtype definitions");

    let layers = vec![LayerId::new(0)];

    tracing::debug!(layer_count = layers.len(), "Querying domains for layers");

    let result = service.query_domains(layers).await?;

    let layer_count = result.layers().len();
    tracing::info!(layers_with_domains = layer_count, "Domain query completed");

    anyhow::ensure!(
        layer_count > 0,
        "Domain query returned no layers. Service may not support domains or query failed."
    );

    for layer_info in result.layers() {
        let domain_count = layer_info.domains().len();
        let subtype_count = layer_info.subtypes().as_ref().map(|s| s.len()).unwrap_or(0);

        tracing::info!(
            layer_id = ?layer_info.id(),
            domains = domain_count,
            subtypes = subtype_count,
            "Layer domain information"
        );

        // Display first few domains
        for (field_name, domain) in layer_info.domains().iter().take(3) {
            tracing::debug!(
                field = %field_name,
                domain_name = %domain.name(),
                domain_type = ?domain.domain_type(),
                "Domain definition"
            );

            // Display coded values if available
            if let Some(coded_values) = domain.coded_values() {
                let value_count = coded_values.len();
                tracing::debug!(
                    field = %field_name,
                    coded_value_count = value_count,
                    "Coded values available"
                );

                for (idx, coded_value) in coded_values.iter().take(3).enumerate() {
                    tracing::debug!(
                        field = %field_name,
                        value_num = idx + 1,
                        code = ?coded_value.code(),
                        name = %coded_value.name(),
                        "Valid domain value"
                    );
                }
            }
        }
    }

    tracing::debug!("Domain lookups demonstration complete");
    Ok(())
}

/// Prints best practices for advanced queries.
fn print_best_practices() {
    tracing::info!("\n💡 Advanced Query Best Practices:");
    tracing::info!("");
    tracing::info!("📊 Statistical Aggregations:");
    tracing::info!("   - Use statistics queries instead of client-side aggregation");
    tracing::info!("   - GROUP BY reduces data transfer and improves performance");
    tracing::info!("   - HAVING filters aggregated results efficiently");
    tracing::info!("   - Combine with ORDER BY to sort aggregated results");
    tracing::info!("");
    tracing::info!("🏆 Top Features:");
    tracing::info!("   - Use queryTopFeatures for ranking within groups");
    tracing::info!("   - Specify clear ORDER BY for deterministic results");
    tracing::info!("   - Consider filtering with WHERE before grouping");
    tracing::info!("   - Top queries reduce result size vs full query + sort");
    tracing::info!("");
    tracing::info!("📄 Pagination:");
    tracing::info!("   - Manual pagination: Full control over page fetching");
    tracing::info!("   - Automatic pagination: Convenient for smaller datasets");
    tracing::info!("   - Use resultOffset/resultRecordCount for manual control");
    tracing::info!("   - Monitor exceededTransferLimit to detect more data");
    tracing::info!("   - Page size affects performance: 100-1000 is typical");
    tracing::info!("");
    tracing::info!("🔗 Relationships:");
    tracing::info!("   - Query related records instead of joining tables");
    tracing::info!("   - Results are grouped by source object ID");
    tracing::info!("   - Filter related records with definitionExpression");
    tracing::info!("   - Reduce network traffic with targeted out_fields");
    tracing::info!("");
    tracing::info!("🏷️  Domains:");
    tracing::info!("   - Query domains once, cache for form validation");
    tracing::info!("   - Use coded values for dropdown lists");
    tracing::info!("   - Subtypes affect valid domain values per feature");
    tracing::info!("   - Domain queries don't count against rate limits");
    tracing::info!("");
    tracing::info!("⚡ Performance:");
    tracing::info!("   - Server-side aggregation >> client-side processing");
    tracing::info!("   - Return only needed fields with out_fields");
    tracing::info!("   - Skip geometry when not needed (return_geometry: false)");
    tracing::info!("   - Use appropriate page sizes for your use case");
    tracing::info!("   - Consider PBF format for large datasets");
}
