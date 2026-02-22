//! 🔧 Portal Service Management Example
//!
//! Demonstrates advanced service management operations for hosted feature services.
//! This example shows how to publish, monitor, update, and overwrite services.
//!
//! # What You'll Learn
//!
//! - **Publish status monitoring**: Track asynchronous publish jobs
//! - **Service definition updates**: Modify service capabilities and settings
//! - **Service overwrite**: Replace service data while preserving URL/ID
//! - **Job polling**: Monitor long-running operations
//! - **Service lifecycle**: Complete publish → update → overwrite workflow
//!
//! # Prerequisites
//!
//! - Required: API key with content creation privileges in `.env`
//! - Permissions: Create items, publish services, manage content
//!
//! ## Environment Variables
//!
//! ```env
//! ARCGIS_CONTENT_KEY=your_api_key_with_content_privileges
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example portal_service_management
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example portal_service_management
//! ```
//!
//! # Real-World Use Cases
//!
//! - **CI/CD pipelines**: Automated service deployment and updates
//! - **Data refresh workflows**: Overwrite services with new data regularly
//! - **Service configuration**: Programmatic capability management
//! - **Performance tuning**: Adjust max record counts and caching
//! - **Blue-green deployments**: Maintain stable URLs while updating data
//!
//! # Operations Demonstrated
//!
//! - **get_publish_status**: Monitor publish job progress
//! - **update_service_definition**: Modify service settings
//! - **overwrite_service**: Replace service data (preserves item ID/URL)

use anyhow::Result;
use arcgis::{
    AddItemParams, ApiKeyAuth, ApiKeyTier, ArcGISClient, OverwriteParameters, PortalClient,
    PublishParameters, UpdateServiceDefinitionParams,
};
use std::time::Duration;

/// Portal base URL for ArcGIS Online
const PORTAL_URL: &str = "https://www.arcgis.com/sharing/rest";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🔧 Portal Service Management Example");
    tracing::info!("");

    // Load API key from environment
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Content)
        .expect("ARCGIS_CONTENT_KEY environment variable required");

    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new(PORTAL_URL, &client);

    tracing::info!("✅ Authenticated with API key");
    tracing::info!("");

    // Run the service management workflow
    run_service_management_workflow(&portal).await?;

    tracing::info!("✅ All service management operations completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates the complete service management workflow.
async fn run_service_management_workflow(portal: &PortalClient<'_>) -> Result<()> {
    // ========================================================================
    // STEP 1: Upload initial GeoJSON data
    // ========================================================================
    tracing::info!("=== STEP 1: Uploading Initial GeoJSON Data ===");
    tracing::info!("Creating initial dataset for publishing");
    tracing::info!("");

    let initial_geojson = r#"{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [-122.4194, 37.7749]
      },
      "properties": {
        "name": "San Francisco",
        "population": 883305,
        "version": "1.0"
      }
    },
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [-118.2437, 34.0522]
      },
      "properties": {
        "name": "Los Angeles",
        "population": 3979576,
        "version": "1.0"
      }
    }
  ]
}"#;

    let timestamp = chrono::Utc::now().timestamp();
    let service_name = format!("ServiceMgmt_{}", timestamp);

    let item_params = AddItemParams::new(format!("{} Source", service_name), "GeoJSON")
        .with_description("Source data for service management demo")
        .with_tags(vec!["demo".to_string(), "service-mgmt".to_string()])
        .with_text(initial_geojson);

    let add_result = portal.add_item(item_params).await?;
    let source_item_id = add_result.id().to_string();

    tracing::info!("✅ Uploaded source data");
    tracing::info!("   Item ID: {}", source_item_id);
    tracing::info!("   Features: 2 cities (version 1.0)");
    tracing::info!("");

    // ========================================================================
    // STEP 2: Publish as hosted feature service (asynchronous)
    // ========================================================================
    tracing::info!("=== STEP 2: Publishing Hosted Feature Service ===");
    tracing::info!("Publishing GeoJSON as hosted feature layer");
    tracing::info!("");
    tracing::info!("   Method: publish()");
    tracing::info!("   Note: Returns job_id for asynchronous monitoring");
    tracing::info!("");

    let publish_params = PublishParameters::new(&service_name)
        .with_description("Demo service for testing management operations")
        .with_capabilities("Query,Create,Update,Delete")
        .with_max_record_count(1000);

    let publish_result = portal.publish(&source_item_id, publish_params).await?;

    assert!(
        *publish_result.success(),
        "Publish failed: {:?}",
        publish_result
    );

    tracing::info!("✅ Publish request submitted");

    // ========================================================================
    // STEP 3: Monitor publish status
    // ========================================================================
    if let Some(job_id) = publish_result.job_id() {
        tracing::info!("");
        tracing::info!("=== STEP 3: Monitoring Publish Status ===");
        tracing::info!("Tracking asynchronous publish job");
        tracing::info!("");
        tracing::info!("   Job ID: {}", job_id);
        tracing::info!("   Method: get_publish_status()");
        tracing::info!("");

        // Poll until complete
        let mut attempts = 0;
        let max_attempts = 30; // 30 attempts * 2 seconds = 60 seconds max

        loop {
            attempts += 1;

            let status = portal.get_publish_status(job_id).await?;

            if let Some(job_status) = status.job_status() {
                let progress = status.progress().unwrap_or(0);

                tracing::info!(
                    "   Poll {}/{}: {} ({}%)",
                    attempts,
                    max_attempts,
                    job_status,
                    progress
                );

                // Check if job is complete
                if job_status.contains("Succeeded") {
                    tracing::info!("✅ Publish completed successfully");
                    if !status.messages().is_empty() {
                        tracing::info!("   Messages:");
                        for msg in status.messages() {
                            tracing::info!("     - {}", msg);
                        }
                    }
                    break;
                } else if job_status.contains("Failed") {
                    anyhow::bail!("Publish job failed: {:?}", status);
                }
            }

            if attempts >= max_attempts {
                anyhow::bail!("Publish job timed out after {} seconds", max_attempts * 2);
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    } else {
        tracing::info!("");
        tracing::info!("=== STEP 3: Publish Status ===");
        tracing::info!("✅ Publish completed synchronously (no job_id)");
    }

    let service_item_id = publish_result
        .service_item_id()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No service item ID in publish result"))?
        .to_string();

    tracing::info!("");
    tracing::info!("   Service Item ID: {}", service_item_id);
    if let Some(url) = publish_result.service_url() {
        tracing::info!("   Service URL: {}", url);
    }
    tracing::info!("");

    // ========================================================================
    // STEP 4: Update service definition
    // ========================================================================
    tracing::info!("=== STEP 4: Updating Service Definition ===");
    tracing::info!("Modifying service capabilities and settings");
    tracing::info!("");
    tracing::info!("   Method: update_service_definition()");
    tracing::info!("   Changes:");
    tracing::info!("     - Capabilities: Query only (disable editing)");
    tracing::info!("     - Max records: 2000 → 5000");
    tracing::info!("     - Description: Updated");
    tracing::info!("");

    let update_params = UpdateServiceDefinitionParams::new()
        .with_capabilities("Query")
        .with_max_record_count(5000)
        .with_description("Updated service - read-only with higher query limits");

    let update_result = portal
        .update_service_definition(&service_item_id, update_params)
        .await?;

    assert!(
        *update_result.success(),
        "Service definition update failed: {:?}",
        update_result
    );

    tracing::info!("✅ Service definition updated");
    tracing::info!("   New capabilities: Query only");
    tracing::info!("   New max records: 5000");
    tracing::info!("");

    // ========================================================================
    // STEP 5: Create updated dataset
    // ========================================================================
    tracing::info!("=== STEP 5: Creating Updated Dataset ===");
    tracing::info!("Preparing new data to overwrite the service");
    tracing::info!("");

    let updated_geojson = r#"{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [-122.4194, 37.7749]
      },
      "properties": {
        "name": "San Francisco",
        "population": 873965,
        "version": "2.0"
      }
    },
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [-118.2437, 34.0522]
      },
      "properties": {
        "name": "Los Angeles",
        "population": 3898747,
        "version": "2.0"
      }
    },
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [-121.8863, 37.3382]
      },
      "properties": {
        "name": "San Jose",
        "population": 1026908,
        "version": "2.0"
      }
    }
  ]
}"#;

    let update_item_params =
        AddItemParams::new(format!("{} Updated", service_name), "GeoJSON")
            .with_description("Updated data for overwrite operation")
            .with_tags(vec!["demo".to_string(), "service-mgmt".to_string()])
            .with_text(updated_geojson);

    let update_add_result = portal.add_item(update_item_params).await?;
    let update_source_id = update_add_result.id().to_string();

    tracing::info!("✅ Uploaded updated data");
    tracing::info!("   Item ID: {}", update_source_id);
    tracing::info!("   Features: 3 cities (version 2.0)");
    tracing::info!("   Change: Added San Jose");
    tracing::info!("");

    // ========================================================================
    // STEP 6: Overwrite service with new data
    // ========================================================================
    tracing::info!("=== STEP 6: Overwriting Service ===");
    tracing::info!("Replacing service data while preserving URL and item ID");
    tracing::info!("");
    tracing::info!("   Method: overwrite_service()");
    tracing::info!("   Source: {} (3 features)", update_source_id);
    tracing::info!("   Target: {} (2 features → 3 features)", service_item_id);
    tracing::info!("   Benefit: Service URL and item ID remain the same");
    tracing::info!("");

    let overwrite_params = OverwriteParameters::new(&update_source_id, &service_item_id);

    let overwrite_result = portal.overwrite_service(overwrite_params).await?;

    assert!(
        *overwrite_result.success(),
        "Service overwrite failed: {:?}",
        overwrite_result
    );

    tracing::info!("✅ Service overwritten successfully");
    tracing::info!("   Item ID: {} (unchanged)", service_item_id);
    tracing::info!("   Data: Updated to version 2.0 with 3 features");
    tracing::info!("");

    // ========================================================================
    // STEP 7: Cleanup
    // ========================================================================
    tracing::info!("=== STEP 7: Cleaning Up ===");
    tracing::info!("Deleting test items and service");
    tracing::info!("");

    // Delete service
    let delete_service = portal.delete_item(&service_item_id).await?;
    assert!(*delete_service.success(), "Failed to delete service");
    tracing::info!("✅ Deleted service: {}", service_item_id);

    // Delete source items
    let delete_source = portal.delete_item(&source_item_id).await?;
    assert!(*delete_source.success(), "Failed to delete source item");
    tracing::info!("✅ Deleted source item: {}", source_item_id);

    let delete_update = portal.delete_item(&update_source_id).await?;
    assert!(*delete_update.success(), "Failed to delete update item");
    tracing::info!("✅ Deleted update item: {}", update_source_id);
    tracing::info!("");

    // ========================================================================
    // Summary
    // ========================================================================
    tracing::info!("📊 Service Management Workflow Summary:");
    tracing::info!("   ✓ Published service from GeoJSON data");
    tracing::info!("   ✓ Monitored publish job with get_publish_status()");
    tracing::info!("   ✓ Updated service definition (capabilities, max records)");
    tracing::info!("   ✓ Overwrote service data while preserving URL/ID");
    tracing::info!("   ✓ Cleaned up all test resources");

    Ok(())
}

/// Prints best practices for service management operations.
fn print_best_practices() {
    tracing::info!("");
    tracing::info!("💡 Service Management Best Practices:");
    tracing::info!("   - Poll get_publish_status() for long-running publish jobs");
    tracing::info!("   - Use reasonable polling intervals (2-5 seconds)");
    tracing::info!("   - Set timeouts to prevent infinite loops");
    tracing::info!("   - Check job_status for 'Succeeded' or 'Failed'");
    tracing::info!("");
    tracing::info!("🎯 When to Use Each Operation:");
    tracing::info!("   get_publish_status():      Monitor async publish jobs");
    tracing::info!("   update_service_definition(): Change capabilities, limits, settings");
    tracing::info!("   overwrite_service():        Replace data, preserve URL/ID");
    tracing::info!("");
    tracing::info!("⚙️  Service Definition Updates:");
    tracing::info!("   - Capabilities: Query, Create, Update, Delete, Extract");
    tracing::info!("   - Max record count: Balance performance vs completeness");
    tracing::info!("   - Description: Update metadata without republishing");
    tracing::info!("");
    tracing::info!("🔄 Overwrite vs Republish:");
    tracing::info!("   Overwrite:  Preserves item ID and URL (recommended)");
    tracing::info!("   Republish:  Creates new service with new ID/URL");
    tracing::info!("   Use case:   Data refresh workflows, CI/CD pipelines");
    tracing::info!("");
    tracing::info!("⚠️  Important Notes:");
    tracing::info!("   - Publish jobs may take 10-60 seconds depending on data size");
    tracing::info!("   - Service definition changes take effect immediately");
    tracing::info!("   - Overwrite preserves item sharing and metadata");
    tracing::info!("   - Always test overwrite with non-production services first");
}
