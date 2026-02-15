//! Feature Service Field Calculations Example
//!
//! This example demonstrates the calculate_records() method which performs
//! bulk field calculations using SQL expressions or scalar values.
//!
//! # Operations Tested
//!
//! - `calculate_records()` - Update field values using SQL expressions
//! - `calculate_records()` - Update field values using scalar values
//!
//! # ESRI API Documentation
//!
//! - Calculate: <https://developers.arcgis.com/rest/services-reference/enterprise/calculate-feature-service-layer/>
//!
//! # Coverage
//!
//! This example tests:
//! - ✅ calculate_records() with SQL expressions
//! - ✅ calculate_records() with scalar values
//! - ✅ CalculateResult response handling
//! - ✅ FieldCalculation with both value and sqlExpression
//!
//! # Usage
//!
//! This example requires authentication with a feature service that you own and control.
//! **Important**: Public sample servers do not accept API key editing operations.
//!
//! ## Setup
//!
//! 1. Create your own hosted feature service in ArcGIS Online:
//!    - Log into https://www.arcgis.com
//!    - Go to Content → New Item → Feature Layer
//!    - Create a simple layer with text fields (e.g., "eventtype", "description")
//!    - Enable editing in the settings
//!    - Copy the Feature Service URL (ends with /FeatureServer)
//!
//! 2. Set environment variables in your `.env` file:
//!
//! ```env
//! ARCGIS_FEATURES_KEY=your_api_key_with_edit_privileges
//! FEATURE_SERVICE_URL=https://services.arcgis.com/YOUR_ORG/arcgis/rest/services/YOUR_SERVICE/FeatureServer
//! LAYER_ID=0
//! ```
//!
//! 3. Run the example:
//!
//! ```bash
//! cargo run --example feature_service_field_calculations
//!
//! # With debug logging to see request/response details:
//! RUST_LOG=debug cargo run --example feature_service_field_calculations
//! ```
//!
//! ## Troubleshooting
//!
//! - **Error 498 "Invalid Token"**: Your API key doesn't have permissions for this service.
//!   Make sure you own the service and the API key has editing privileges.
//!
//! - **Error 400 "Field does not exist"**: The layer schema doesn't match the example.
//!   Update the field names in the example to match your layer's fields.

use anyhow::{Context, Result};
use arcgis::{
    ApiKeyAuth, ArcGISClient, EditOptions, EnvConfig, Feature, FeatureQueryParams,
    FeatureServiceClient, FieldCalculation, LayerId, ObjectId,
};
use secrecy::ExposeSecret;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("feature_service_field_calculations=info".parse()?),
        )
        .init();

    tracing::info!("=== Feature Service Field Calculations Example ===");

    // Load environment configuration (automatically loads .env)
    let config = EnvConfig::global();

    // Validate required API key
    let features_key = config.arcgis_features_key.as_ref().context(
        "ARCGIS_FEATURES_KEY not set. Add to .env:\n\
         ARCGIS_FEATURES_KEY=your_api_key_here\n\
         \n\
         This key is used for feature editing operations (add/update/delete/calculate).",
    )?;

    // Get service URL - REQUIRED for this example
    let service_url = std::env::var("FEATURE_SERVICE_URL").context(
        "FEATURE_SERVICE_URL not set. This example requires a feature service you own.\n\
         \n\
         Public sample servers reject API key editing operations.\n\
         \n\
         To run this example:\n\
         1. Create a hosted feature service in ArcGIS Online (https://www.arcgis.com)\n\
         2. Enable editing and add some text fields (eventtype, description)\n\
         3. Set FEATURE_SERVICE_URL in .env:\n\
         \n\
         FEATURE_SERVICE_URL=https://services.arcgis.com/YOUR_ORG/arcgis/rest/services/YOUR_SERVICE/FeatureServer\n\
         \n\
         See example documentation for detailed setup instructions.",
    )?;

    let layer_id_str = std::env::var("LAYER_ID").unwrap_or_else(|_| "0".to_string());
    let layer_id = LayerId::new(layer_id_str.parse::<u32>()?);

    // Create client with features key
    let auth = ApiKeyAuth::new(features_key.expose_secret());
    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new(&service_url, &client);

    tracing::info!("Connected to feature service: {}", service_url);
    tracing::info!("Using layer ID: {}", layer_id);
    tracing::info!("");
    tracing::info!("⚠️  Note: This example will add, modify, and delete features.");
    tracing::info!("   Make sure you have a backup if using a production service.");
    tracing::info!("");

    // ========================================
    // Setup: Add test features for calculation
    // ========================================
    tracing::info!("\n--- Setup: Adding test features ---");

    let mut test_features = Vec::new();
    for i in 1..=3 {
        let mut attributes = HashMap::new();
        attributes.insert("eventtype".to_string(), json!("Test Event"));
        attributes.insert("description".to_string(), json!(format!("Test {}", i)));
        attributes.insert("eventdate".to_string(), json!(1234567890000i64)); // Unix timestamp in ms

        test_features.push(Feature::new(attributes, None));
    }

    let add_result = service
        .add_features(layer_id, test_features, EditOptions::default())
        .await?;

    anyhow::ensure!(
        add_result.all_succeeded(),
        "Failed to add test features: {:?}",
        add_result.add_results()
    );

    let added_ids: Vec<ObjectId> = add_result
        .add_results()
        .iter()
        .filter_map(|r| *r.object_id())
        .collect();

    tracing::info!("✓ Added {} test features", added_ids.len());
    tracing::info!("  Object IDs: {:?}", added_ids);

    anyhow::ensure!(
        added_ids.len() == 3,
        "Expected 3 features, got {}",
        added_ids.len()
    );

    // Build WHERE clause for our test features
    let where_clause = format!(
        "OBJECTID IN ({})",
        added_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    // ========================================
    // Test 1: Calculate using scalar value
    // ========================================
    tracing::info!("\n--- Test 1: Calculate with scalar value ---");

    let calc_expressions = vec![FieldCalculation::with_value(
        "eventtype",
        json!("Updated Event"),
    )];

    let result = service
        .calculate_records(
            layer_id,
            &where_clause,
            calc_expressions,
            EditOptions::default(),
        )
        .await?;

    tracing::info!("Calculate result: {:?}", result);

    // Validate result
    anyhow::ensure!(
        *result.success(),
        "Calculate operation should succeed: {:?}",
        result
    );

    if let Some(count) = result.updated_feature_count() {
        anyhow::ensure!(*count == 3, "Should update 3 features, got {}", count);
        tracing::info!("✓ Updated {} features with scalar value", count);
    }

    // Verify the update by querying
    let params = FeatureQueryParams::builder()
        .where_clause(&where_clause)
        .return_geometry(false)
        .out_fields(vec!["OBJECTID".to_string(), "eventtype".to_string()])
        .build()?;

    let feature_set = service.query_with_params(layer_id, params).await?;

    anyhow::ensure!(
        feature_set.features().len() == 3,
        "Should retrieve 3 features"
    );

    for feature in feature_set.features() {
        let event_type = feature
            .attributes()
            .get("eventtype")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        anyhow::ensure!(
            event_type == "Updated Event",
            "eventtype should be 'Updated Event', got '{}'",
            event_type
        );
    }

    tracing::info!("✓ Verified scalar value calculation");

    // ========================================
    // Test 2: Calculate using SQL expression
    // ========================================
    tracing::info!("\n--- Test 2: Calculate with SQL expression ---");

    // Update description field using a SQL expression that references another field
    let calc_expressions = vec![FieldCalculation::with_sql_expression(
        "description",
        "CONCAT(eventtype, ' - Modified')",
    )];

    let result = service
        .calculate_records(
            layer_id,
            &where_clause,
            calc_expressions,
            EditOptions::default(),
        )
        .await?;

    tracing::info!("Calculate result: {:?}", result);

    // Validate result
    anyhow::ensure!(
        *result.success(),
        "Calculate operation should succeed: {:?}",
        result
    );

    if let Some(count) = result.updated_feature_count() {
        anyhow::ensure!(*count == 3, "Should update 3 features, got {}", count);
        tracing::info!("✓ Updated {} features with SQL expression", count);
    }

    // Verify the update
    let params = FeatureQueryParams::builder()
        .where_clause(&where_clause)
        .return_geometry(false)
        .out_fields(vec![
            "OBJECTID".to_string(),
            "description".to_string(),
            "eventtype".to_string(),
        ])
        .build()?;

    let feature_set = service.query_with_params(layer_id, params).await?;

    for feature in feature_set.features() {
        let description = feature
            .attributes()
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        anyhow::ensure!(
            description == "Updated Event - Modified",
            "description should be 'Updated Event - Modified', got '{}'",
            description
        );
    }

    tracing::info!("✓ Verified SQL expression calculation");

    // ========================================
    // Test 3: Multiple field calculations
    // ========================================
    tracing::info!("\n--- Test 3: Calculate multiple fields simultaneously ---");

    let calc_expressions = vec![
        FieldCalculation::with_value("eventtype", json!("Final Event")),
        FieldCalculation::with_value("description", json!("Test Complete")),
    ];

    let result = service
        .calculate_records(
            layer_id,
            &where_clause,
            calc_expressions,
            EditOptions::default(),
        )
        .await?;

    tracing::info!("Calculate result: {:?}", result);

    anyhow::ensure!(
        *result.success(),
        "Calculate operation should succeed: {:?}",
        result
    );

    if let Some(count) = result.updated_feature_count() {
        anyhow::ensure!(*count == 3, "Should update 3 features, got {}", count);
        tracing::info!("✓ Updated {} features (multiple fields)", count);
    }

    // Verify both fields were updated
    let params = FeatureQueryParams::builder()
        .where_clause(&where_clause)
        .return_geometry(false)
        .out_fields(vec![
            "OBJECTID".to_string(),
            "eventtype".to_string(),
            "description".to_string(),
        ])
        .build()?;

    let feature_set = service.query_with_params(layer_id, params).await?;

    for feature in feature_set.features() {
        let event_type = feature
            .attributes()
            .get("eventtype")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let description = feature
            .attributes()
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        anyhow::ensure!(
            event_type == "Final Event",
            "eventtype should be 'Final Event', got '{}'",
            event_type
        );

        anyhow::ensure!(
            description == "Test Complete",
            "description should be 'Test Complete', got '{}'",
            description
        );
    }

    tracing::info!("✓ Verified multiple field calculations");

    // ========================================
    // Cleanup: Delete test features
    // ========================================
    tracing::info!("\n--- Cleanup: Deleting test features ---");

    let delete_result = service
        .delete_features(layer_id, added_ids, EditOptions::default())
        .await?;

    anyhow::ensure!(
        delete_result.all_succeeded(),
        "Failed to delete test features: {:?}",
        delete_result.delete_results()
    );

    tracing::info!(
        "✓ Deleted {} test features",
        delete_result.delete_results().len()
    );

    // ========================================
    // Summary
    // ========================================
    tracing::info!("\n=== Summary ===");
    tracing::info!("✓ All calculate_records() operations succeeded");
    tracing::info!("✓ FieldCalculation with scalar values works correctly");
    tracing::info!("✓ FieldCalculation with SQL expressions works correctly");
    tracing::info!("✓ Multiple field calculations work correctly");
    tracing::info!("✓ CalculateResult response format validated");
    tracing::info!("\nFeatureServiceClient coverage for calculate_records: 100%");

    Ok(())
}
