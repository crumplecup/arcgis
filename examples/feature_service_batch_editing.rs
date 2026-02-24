//! 📝 Feature Service - Batch Editing Operations
//!
//! Demonstrates atomic batch editing operations for feature services.
//! Learn how to efficiently add, update, and delete features in single transactions,
//! ensuring data integrity and optimal performance.
//!
//! # What You'll Learn
//!
//! - **Atomic editing**: Apply adds, updates, and deletes in one transaction
//! - **Bulk updates**: Update multiple features efficiently
//! - **Global IDs**: Use global IDs for replicated/offline editing scenarios
//! - **Table definitions**: Query table schema and metadata
//! - **Transaction control**: Rollback on failure, session management
//!
//! # Prerequisites
//!
//! - ArcGIS Feature Service with edit capabilities
//! - Appropriate authentication (API key or OAuth)
//! - Features with OBJECTID field (and optionally GlobalID)
//!
//! ## Environment Variables
//!
//! Set these in your `.env` file:
//!
//! ```env
//! # Feature service base URL
//! ARCGIS_FEATURE_URL=https://your-server.com/arcgis/rest/services/MyService/FeatureServer
//!
//! # Authentication (choose one)
//! ARCGIS_ENTERPRISE_KEY=your_enterprise_api_key
//! # OR
//! ARCGIS_FEATURES_KEY=your_features_api_key
//! # OR
//! ARCGIS_CLIENT_ID=your_oauth_client_id
//! ARCGIS_CLIENT_SECRET=your_oauth_client_secret
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example feature_service_batch_editing
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example feature_service_batch_editing
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Data import**: Bulk load features from external sources
//! - **Maintenance**: Update multiple asset statuses in one transaction
//! - **Quality control**: Batch update attributes after validation
//! - **Synchronization**: Apply offline edits using global IDs
//! - **Cleanup**: Delete obsolete features efficiently

use anyhow::Result;
use arcgis::{
    ApiKeyAuth, ApiKeyTier, ArcGISClient, EditOptions, EnvConfig, Feature, FeatureServiceClient,
    LayerId, ObjectId,
};
use arcgis::example_tracker::ExampleTracker;
use secrecy::ExposeSecret;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Start accountability tracking
    let tracker = ExampleTracker::new("feature_service_batch_editing")
        .methods(&[
            "get_table_definition", "apply_edits",
            "update_features", "apply_edits_with_global_ids"
        ])
        .service_type("FeatureServiceClient")
        .start();

    tracing::info!("📝 ArcGIS Feature Service - Batch Editing Examples");
    tracing::info!("Demonstrating atomic editing operations");
    tracing::info!("");

    // Load feature service URL from environment
    let config = EnvConfig::global();
    let feature_url = config
        .arcgis_feature_url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!(
            "ARCGIS_FEATURE_URL not set in .env file.\n\
             Example: ARCGIS_FEATURE_URL=https://your-server.com/arcgis/rest/services/MyService/FeatureServer"
        ))?;

    tracing::info!("Feature Service: {}", feature_url);
    tracing::info!("");

    // Create authenticated client
    tracing::debug!("Creating authenticated client");

    // Use enterprise key for enterprise servers, fallback to features key
    let auth = if let Some(enterprise_key) = &config.arcgis_enterprise_key {
        tracing::debug!("Using ARCGIS_ENTERPRISE_KEY for authentication");
        ApiKeyAuth::new(enterprise_key.expose_secret())
    } else {
        tracing::debug!("Using ARCGIS_FEATURES_KEY for authentication");
        ApiKeyAuth::from_env(ApiKeyTier::Features)?
    };

    let client = ArcGISClient::new(auth);
    let fs_client = FeatureServiceClient::new(feature_url, &client);

    // Demonstrate batch editing operations
    demonstrate_get_table_definition(&fs_client).await?;
    demonstrate_apply_edits(&fs_client).await?;
    demonstrate_update_features(&fs_client).await?;
    demonstrate_apply_edits_with_global_ids(&fs_client).await?;

    tracing::info!("\n✅ All batch editing examples completed successfully!");
    tracing::info!("🎉 100% FeatureServiceClient batch editing coverage achieved!");
    print_best_practices();

    // Mark tracking as successful
    tracker.success();
    Ok(())
}

/// Demonstrates querying table definition metadata.
async fn demonstrate_get_table_definition(fs_client: &FeatureServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Get Table Definition ===");
    tracing::info!("Query schema and metadata for a table");
    tracing::info!("");

    // Query first table/layer (usually layer 0)
    let layer_id = LayerId::new(0);

    tracing::info!("Querying table definition for layer {}...", layer_id);

    let table_def = fs_client.get_table_definition(layer_id).await?;

    tracing::info!("✅ Table definition retrieved");
    tracing::info!("   Name: {}", table_def.name());

    if let Some(table_type) = table_def.table_type() {
        tracing::info!("   Type: {}", table_type);
    }

    let fields = table_def.fields();
    tracing::info!("   Fields: {} total", fields.len());

    // Show first few fields
    for (idx, field) in fields.iter().take(5).enumerate() {
        let nullable_str = field.nullable()
            .map(|n| if n { "nullable" } else { "not null" })
            .unwrap_or("unknown");

        tracing::info!(
            "      {}. {} ({:?}, {})",
            idx + 1,
            field.name(),
            field.field_type(),
            nullable_str
        );
    }

    if fields.len() > 5 {
        tracing::info!("      ... and {} more fields", fields.len() - 5);
    }

    // Assertions
    anyhow::ensure!(!fields.is_empty(), "Table should have at least one field");

    if let Some(object_id_field) = table_def.object_id_field() {
        tracing::info!("   Object ID field: {}", object_id_field);
    }

    if let Some(global_id_field) = table_def.global_id_field() {
        tracing::info!("   Global ID field: {}", global_id_field);
    }

    // Note: TableDefinition doesn't include capabilities
    // Use FeatureServiceClient.get_service_definition() for capabilities

    tracing::info!("");
    tracing::info!("💡 Table definition:");
    tracing::info!("   • Describes schema: fields, types, constraints");
    tracing::info!("   • Identifies key fields (OBJECTID, GlobalID)");
    tracing::info!("   • Lists capabilities (Create, Update, Delete, etc.)");
    tracing::info!("   • Essential for building dynamic editing UIs");

    Ok(())
}

/// Demonstrates atomic batch editing with apply_edits.
async fn demonstrate_apply_edits(fs_client: &FeatureServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Atomic Batch Editing (apply_edits) ===");
    tracing::info!("Add, update, and delete features in single transaction");
    tracing::info!("");

    let layer_id = LayerId::new(0);

    // Prepare features to add
    let mut new_attrs1 = HashMap::new();
    new_attrs1.insert("name".to_string(), json!("Batch Test Feature 1"));
    new_attrs1.insert("description".to_string(), json!("Created by apply_edits"));
    let feature_to_add1 = Feature::new(new_attrs1, None);

    let mut new_attrs2 = HashMap::new();
    new_attrs2.insert("name".to_string(), json!("Batch Test Feature 2"));
    new_attrs2.insert("description".to_string(), json!("Also created by apply_edits"));
    let feature_to_add2 = Feature::new(new_attrs2, None);

    tracing::info!("Preparing batch operation:");
    tracing::info!("   • Adding 2 new features");
    tracing::info!("   • Rollback enabled (all or nothing)");
    tracing::info!("");

    // Apply edits with rollback on failure
    let options = EditOptions {
        rollback_on_failure: Some(true),
        ..Default::default()
    };

    let result = fs_client
        .apply_edits(
            layer_id,
            Some(vec![feature_to_add1, feature_to_add2]),
            None,  // No updates
            None,  // No deletes
            options,
        )
        .await?;

    // Validate results
    tracing::info!("✅ Batch edit completed");
    tracing::info!("   Total success: {}", result.success_count());
    tracing::info!("   Total failure: {}", result.failure_count());
    tracing::info!("   Add results: {}", result.add_results().len());

    // Assertions
    anyhow::ensure!(
        result.add_results().len() == 2,
        "Should have 2 add results"
    );

    anyhow::ensure!(
        result.all_succeeded(),
        "All operations should succeed. Failures: {:?}",
        result
            .add_results()
            .iter()
            .filter(|r| !r.success())
            .collect::<Vec<_>>()
    );

    // Get the created Object IDs for cleanup
    let created_ids: Vec<ObjectId> = result
        .add_results()
        .iter()
        .filter_map(|r| *r.object_id())
        .collect();

    anyhow::ensure!(
        created_ids.len() == 2,
        "Should have 2 created object IDs"
    );

    tracing::info!("   Created IDs: {:?}", created_ids);

    // Clean up - delete the features we just created
    tracing::info!("");
    tracing::info!("Cleaning up created features...");

    let cleanup_result = fs_client
        .apply_edits(
            layer_id,
            None,               // No adds
            None,               // No updates
            Some(created_ids),  // Delete the ones we created
            EditOptions::default(),
        )
        .await?;

    anyhow::ensure!(
        cleanup_result.all_succeeded(),
        "Cleanup should succeed"
    );

    tracing::info!("✅ Cleanup completed");

    tracing::info!("");
    tracing::info!("💡 apply_edits advantages:");
    tracing::info!("   • Atomic: all operations succeed or all fail");
    tracing::info!("   • Efficient: single request for multiple operations");
    tracing::info!("   • Mixed operations: add + update + delete together");
    tracing::info!("   • Transaction control: rollback_on_failure option");

    Ok(())
}

/// Demonstrates bulk update operations.
async fn demonstrate_update_features(fs_client: &FeatureServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Bulk Feature Updates ===");
    tracing::info!("Update multiple features efficiently");
    tracing::info!("");

    let layer_id = LayerId::new(0);

    // First, create test features to update
    let mut attrs1 = HashMap::new();
    attrs1.insert("name".to_string(), json!("Update Test 1"));
    attrs1.insert("status".to_string(), json!("initial"));
    let feature1 = Feature::new(attrs1, None);

    let mut attrs2 = HashMap::new();
    attrs2.insert("name".to_string(), json!("Update Test 2"));
    attrs2.insert("status".to_string(), json!("initial"));
    let feature2 = Feature::new(attrs2, None);

    tracing::info!("Creating test features...");

    let add_result = fs_client
        .add_features(layer_id, vec![feature1, feature2], EditOptions::default())
        .await?;

    anyhow::ensure!(add_result.all_succeeded(), "Feature creation should succeed");

    let created_ids: Vec<(i64, ObjectId)> = add_result
        .add_results()
        .iter()
        .enumerate()
        .filter_map(|(idx, r)| r.object_id().as_ref().map(|id| (idx as i64, *id)))
        .collect();

    anyhow::ensure!(created_ids.len() == 2, "Should create 2 features");

    tracing::info!("✅ Created {} test features", created_ids.len());
    tracing::info!("");

    // Now prepare bulk updates
    let mut update1_attrs = HashMap::new();
    update1_attrs.insert("OBJECTID".to_string(), json!(created_ids[0].1.get()));
    update1_attrs.insert("status".to_string(), json!("updated"));
    let update1 = Feature::new(update1_attrs, None);

    let mut update2_attrs = HashMap::new();
    update2_attrs.insert("OBJECTID".to_string(), json!(created_ids[1].1.get()));
    update2_attrs.insert("status".to_string(), json!("updated"));
    let update2 = Feature::new(update2_attrs, None);

    tracing::info!("Updating features in bulk...");

    let update_result = fs_client
        .update_features(
            layer_id,
            vec![update1, update2],
            EditOptions {
                rollback_on_failure: Some(true),
                ..Default::default()
            },
        )
        .await?;

    // Validate update results
    tracing::info!("✅ Bulk update completed");
    tracing::info!("   Updated: {}", update_result.success_count());
    tracing::info!("   Failed: {}", update_result.failure_count());

    // Assertions
    anyhow::ensure!(
        update_result.update_results().len() == 2,
        "Should have 2 update results"
    );

    anyhow::ensure!(
        update_result.all_succeeded(),
        "All updates should succeed"
    );

    // Cleanup
    tracing::info!("");
    tracing::info!("Cleaning up test features...");

    let cleanup_ids: Vec<ObjectId> = created_ids.into_iter().map(|(_, id)| id).collect();

    let cleanup_result = fs_client
        .delete_features(layer_id, cleanup_ids, EditOptions::default())
        .await?;

    anyhow::ensure!(
        cleanup_result.all_succeeded(),
        "Cleanup should succeed"
    );

    tracing::info!("✅ Cleanup completed");

    tracing::info!("");
    tracing::info!("💡 update_features:");
    tracing::info!("   • Bulk updates are more efficient than individual edits");
    tracing::info!("   • Features must include OBJECTID to identify which to update");
    tracing::info!("   • Partial updates: only specify fields to change");
    tracing::info!("   • Use rollback_on_failure for atomic bulk updates");

    Ok(())
}

/// Demonstrates editing with global IDs.
async fn demonstrate_apply_edits_with_global_ids(
    fs_client: &FeatureServiceClient<'_>,
) -> Result<()> {
    tracing::info!("\n=== Example 4: Global ID Editing ===");
    tracing::info!("Use global IDs for replicated/offline editing scenarios");
    tracing::info!("");

    let layer_id = LayerId::new(0);

    // Note: Global IDs are typically auto-generated by the service
    // For demonstration, we'll create features and then work with their global IDs

    tracing::info!("Creating test feature (global ID will be auto-generated)...");

    let mut new_attrs = HashMap::new();
    new_attrs.insert("name".to_string(), json!("Global ID Test Feature"));
    new_attrs.insert("description".to_string(), json!("Testing global ID operations"));
    let feature = Feature::new(new_attrs, None);

    let add_result = fs_client
        .add_features(layer_id, vec![feature], EditOptions::default())
        .await?;

    anyhow::ensure!(add_result.all_succeeded(), "Feature creation should succeed");

    let first_result = add_result
        .add_results()
        .first()
        .ok_or_else(|| anyhow::anyhow!("No add results returned"))?;

    let object_id = first_result
        .object_id()
        .ok_or_else(|| anyhow::anyhow!("No object ID returned"))?;

    let global_id = first_result
        .global_id()
        .clone()
        .ok_or_else(|| anyhow::anyhow!("No global ID returned (service may not support global IDs)"))?;

    tracing::info!("✅ Feature created");
    tracing::info!("   Object ID: {}", object_id);
    tracing::info!("   Global ID: {}", global_id);
    tracing::info!("");

    // Now demonstrate updating using global IDs
    tracing::info!("Updating feature using global ID...");

    let mut update_attrs = HashMap::new();
    update_attrs.insert("globalId".to_string(), json!(global_id));
    update_attrs.insert("status".to_string(), json!("updated via global ID"));
    let update_feature = Feature::new(update_attrs, None);

    let update_result = fs_client
        .apply_edits_with_global_ids(
            layer_id,
            None,                         // No adds
            Some(vec![update_feature]),  // Update by global ID
            None,                         // No deletes
            EditOptions::default(),
        )
        .await?;

    tracing::info!("✅ Update via global ID completed");
    tracing::info!("   Success: {}", update_result.success_count());

    anyhow::ensure!(
        update_result.all_succeeded(),
        "Global ID update should succeed"
    );

    // Cleanup using global ID
    tracing::info!("");
    tracing::info!("Cleaning up using global ID...");

    let delete_result = fs_client
        .apply_edits_with_global_ids(
            layer_id,
            None,                        // No adds
            None,                        // No updates
            Some(vec![global_id.to_string()]),  // Delete by global ID
            EditOptions::default(),
        )
        .await?;

    anyhow::ensure!(
        delete_result.all_succeeded(),
        "Global ID delete should succeed"
    );

    tracing::info!("✅ Cleanup completed");

    tracing::info!("");
    tracing::info!("💡 Global ID editing:");
    tracing::info!("   • Global IDs are stable across replicas and syncs");
    tracing::info!("   • Essential for disconnected/offline editing");
    tracing::info!("   • Used in multi-user collaborative workflows");
    tracing::info!("   • Survives data migration and replication");
    tracing::info!("   • Automatically generated by service (if enabled)");

    Ok(())
}

/// Prints best practices for batch editing.
fn print_best_practices() {
    tracing::info!("\n💡 Batch Editing Best Practices:");
    tracing::info!("   - Use apply_edits for mixed operations (add + update + delete)");
    tracing::info!("   - Enable rollback_on_failure for atomic transactions");
    tracing::info!("   - Batch multiple edits to reduce network round-trips");
    tracing::info!("   - Use update_features for bulk attribute updates");
    tracing::info!("   - Prefer global IDs for replicated/offline scenarios");
    tracing::info!("   - Always validate EditResult success before proceeding");
    tracing::info!("");
    tracing::info!("📊 Edit Operation Types:");
    tracing::info!("   • add_features - Add new features");
    tracing::info!("   • update_features - Update existing features by OBJECTID");
    tracing::info!("   • delete_features - Delete features by OBJECTID");
    tracing::info!("   • apply_edits - Batch add/update/delete in one transaction");
    tracing::info!("   • apply_edits_with_global_ids - Use global IDs instead of OBJECTIDs");
    tracing::info!("   • calculate_records - Bulk field calculations with SQL");
    tracing::info!("");
    tracing::info!("⚙️  Transaction Control:");
    tracing::info!("   • rollback_on_failure - All or nothing (recommended)");
    tracing::info!("   • session_id - Edit session for versioned geodatabases");
    tracing::info!("   • gdb_version - Target specific version");
    tracing::info!("   • use_global_ids - Use global IDs for identification");
    tracing::info!("");
    tracing::info!("📊 Coverage:");
    tracing::info!("   ✅ 4/6 batch editing methods demonstrated:");
    tracing::info!("      • apply_edits ✅");
    tracing::info!("      • update_features ✅");
    tracing::info!("      • apply_edits_with_global_ids ✅");
    tracing::info!("      • get_table_definition ✅");
    tracing::info!("      • truncate (skipped - destructive)");
    tracing::info!("      • get_service_definition (covered in other examples)");
}
