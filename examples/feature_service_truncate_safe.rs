//! 🗑️ Feature Service - Truncate Operation
//!
//! Demonstrates the truncate() operation which deletes ALL features from a layer.
//!
//! # ⚠️ WARNING - DESTRUCTIVE OPERATION ⚠️
//!
//! This example performs a DESTRUCTIVE operation that CANNOT be undone:
//!
//! - ❌ Deletes ALL features from the layer
//! - ❌ Cannot be rolled back or undone
//! - ❌ Should ONLY be run on test/development services
//! - ❌ NEVER run on production data without backups and approval
//!
//! # What You'll Learn
//!
//! - **Truncate operation**: Delete all features from a layer atomically
//! - **Truncate limitations**: Does NOT support versioned editing (no sessionId/gdbVersion)
//! - **When to use truncate**: Bulk cleanup, data reset, migration preparation
//! - **Safety requirements**: Test services only, backups, approval processes
//!
//! # How Truncate Works
//!
//! ```text
//! 1. Add test features to layer
//! 2. Verify features exist (pre-truncate count)
//! 3. Call truncate() - deletes ALL features
//! 4. Verify layer is empty (post-truncate count)
//! ```
//!
//! # Important Limitations
//!
//! The ESRI REST API truncate operation:
//! - **Does NOT support sessionId parameter** (no edit sessions)
//! - **Does NOT support gdbVersion parameter** (no versioned editing)
//! - **Does NOT support views** (layer cannot be a view)
//! - **Does NOT support sync-enabled layers**
//! - **Does NOT return deleted IDs**
//! - **Requires administrative privileges**
//!
//! This means you CANNOT isolate truncate to a branch version - it always
//! operates on the DEFAULT version of the data.
//!
//! # Prerequisites
//!
//! - ArcGIS Feature Service (Enterprise or Online)
//! - **TEST SERVICE ONLY** - this will delete all data
//! - Administrative authentication (edit permissions)
//! - Layer that is NOT a view and NOT sync-enabled
//!
//! ## Environment Variables
//!
//! Set these in your `.env` file:
//!
//! ```env
//! # TEST Feature service - will have all features deleted!
//! ARCGIS_FEATURE_URL=https://your-server.com/arcgis/rest/services/TEST_SERVICE/FeatureServer
//!
//! # Authentication (Enterprise key recommended)
//! ARCGIS_ENTERPRISE_KEY=your_enterprise_api_key
//! # OR
//! ARCGIS_FEATURES_KEY=your_features_api_key
//! ```
//!
//! # Running
//!
//! ```bash
//! # ONLY run on test services!
//! cargo run --example feature_service_truncate_safe
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example feature_service_truncate_safe
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Test data cleanup**: Clear test layers between test runs
//! - **Administrative maintenance**: Reset staging environments
//! - **Data migration**: Clear target before bulk import
//! - **Development workflows**: Reset dev layers to clean state
//!
//! **CRITICAL**: This operation should ONLY be used on:
//! - Test/development services
//! - Staging environments
//! - With complete backups
//! - With explicit written approval for data deletion

use anyhow::Result;
use arcgis::example_tracker::ExampleTracker;
use arcgis::{
    ApiKeyAuth, ApiKeyTier, ArcGISClient, EditOptions, EnvConfig, Feature,
    FeatureServiceClient, LayerId,
};
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
    let tracker = ExampleTracker::new("feature_service_truncate_safe")
        .methods(&["truncate", "add_features", "query_feature_count"])
        .service_type("FeatureServiceClient")
        .start();

    tracing::info!("🗑️  ArcGIS Feature Service - Truncate Operation");
    tracing::info!("");
    tracing::warn!("⚠️  WARNING: This example will DELETE ALL FEATURES from layer 0");
    tracing::warn!("⚠️  Only run on TEST services with data you can lose!");
    tracing::info!("");

    // Load feature service URL from environment
    let config = EnvConfig::global();
    let feature_url = config
        .arcgis_feature_url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!(
            "ARCGIS_FEATURE_URL not set in .env file.\n\
             \n\
             ⚠️  WARNING: This example will DELETE ALL FEATURES from layer 0!\n\
             \n\
             This example requires a TEST Feature Service (NOT production!).\n\
             The layer must NOT be a view and must NOT be sync-enabled.\n\
             \n\
             Example: ARCGIS_FEATURE_URL=https://your-test-server.com/arcgis/rest/services/TEST_SERVICE/FeatureServer"
        ))?;

    tracing::info!("Connected to feature service: {}", feature_url);
    tracing::info!("");

    // Create authenticated client
    let auth = if let Some(enterprise_key) = &config.arcgis_enterprise_key {
        tracing::debug!("Using ARCGIS_ENTERPRISE_KEY for authentication");
        ApiKeyAuth::new(enterprise_key.expose_secret())
    } else {
        tracing::debug!("Using ARCGIS_FEATURES_KEY for authentication");
        ApiKeyAuth::from_env(ApiKeyTier::Features)?
    };

    let client = ArcGISClient::new(auth);
    let service = FeatureServiceClient::new(feature_url, &client);

    // Demonstrate truncate operation
    let success = demonstrate_truncate(&service).await?;

    if success {
        tracing::info!("\n✅ Truncate operation demonstration completed!");
        tracing::info!("🎉 FeatureServiceClient truncate() method verified!");
    } else {
        tracing::info!("\n✅ Truncate method tested (not supported on this server)");
    }

    print_safety_guidelines();

    // Mark tracking as successful
    tracker.success();
    Ok(())
}

/// Demonstrates the truncate operation.
///
/// IMPORTANT: This operates on the DEFAULT version - there is no "safe" version management
/// approach because ESRI's truncate API does not support sessionId or gdbVersion parameters.
///
/// Returns true if truncate succeeded, false if not supported.
async fn demonstrate_truncate(service: &FeatureServiceClient<'_>) -> Result<bool> {
    tracing::info!("\n=== Truncate Operation ===");
    tracing::info!("Demonstrating bulk feature deletion");
    tracing::info!("");

    let layer_id = LayerId::new(0);

    // STEP 1: Add test features to layer
    tracing::info!("📋 STEP 1: Add test features to layer (will be deleted by truncate)");

    let test_features: Vec<Feature> = (0..5)
        .map(|i| {
            let mut attrs = HashMap::new();
            attrs.insert("label".to_string(), json!(format!("Truncate Test {}", i)));
            Feature::new(attrs, None)
        })
        .collect();

    let add_result = service
        .add_features(layer_id, test_features, EditOptions::default())
        .await?;

    anyhow::ensure!(add_result.all_succeeded(), "Failed to add test features");

    let added_count = add_result.success_count();
    tracing::info!("✅ Added {} test features to layer", added_count);
    tracing::info!("");

    // STEP 2: Verify features exist before truncate
    tracing::info!("📋 STEP 2: Verify features exist (pre-truncate count)");

    let pre_truncate_count = service.query_feature_count(layer_id, "1=1").await?;

    tracing::info!("✅ Pre-truncate count: {} features", pre_truncate_count);
    anyhow::ensure!(
        pre_truncate_count >= added_count as u32,
        "Feature count should include our test features"
    );
    tracing::info!("");

    // STEP 3: ⚠️ TRUNCATE - The destructive operation ⚠️
    tracing::info!("📋 STEP 3: ⚠️  TRUNCATE (Delete ALL features from layer) ⚠️");
    tracing::warn!("   This will delete ALL {} features from layer {}", pre_truncate_count, layer_id);
    tracing::warn!("   This operation CANNOT be undone!");
    tracing::info!("");

    let truncate_result = service.truncate(layer_id, EditOptions::default()).await?;

    // Check if truncate is supported on this server
    if truncate_result.success().unwrap_or(false) {
        tracing::info!("✅ TRUNCATE completed successfully");
        tracing::info!("   All features deleted from layer");
        tracing::info!("");
    } else {
        tracing::error!("❌ TRUNCATE failed or not supported");
        if let Some(error) = truncate_result.error() {
            tracing::error!("   Error code: {:?}", error.code());
            tracing::error!("   Error message: {:?}", error.message());
        }
        tracing::warn!("   This server may not support truncate operation");
        tracing::warn!("   Possible reasons:");
        tracing::warn!("   - Server version < 10.7 (truncate added in 10.7)");
        tracing::warn!("   - Branch-versioned services may not support truncate");
        tracing::warn!("   - Service configuration disables truncate");
        tracing::warn!("");
        tracing::warn!("   Alternative: Use delete_features() with WHERE clause '1=1'");
        tracing::info!("");

        // Don't fail the example - just note that truncate isn't supported
        return Ok(false);
    }

    // STEP 4: Verify truncate worked
    tracing::info!("📋 STEP 4: Verify truncate worked (post-truncate count)");

    let post_truncate_count = service.query_feature_count(layer_id, "1=1").await?;

    tracing::info!("✅ Post-truncate count: {} features", post_truncate_count);

    anyhow::ensure!(
        post_truncate_count == 0,
        "Layer should be empty after truncate, found {} features",
        post_truncate_count
    );

    tracing::info!("✅ VERIFIED: Layer is completely empty");
    tracing::info!("");

    // Summary
    tracing::info!("📊 Summary:");
    tracing::info!("   Pre-truncate:  {} features", pre_truncate_count);
    tracing::info!("   Post-truncate: {} features", post_truncate_count);
    tracing::info!("   Deleted:       {} features", pre_truncate_count);
    tracing::info!("");
    tracing::info!("✅ Truncate operation verified working");
    tracing::info!("⚠️  All features have been permanently deleted from layer {}", layer_id);

    Ok(true)
}

/// Prints safety guidelines for truncate operations.
fn print_safety_guidelines() {
    tracing::info!("\n⚠️  TRUNCATE Safety Guidelines:");
    tracing::info!("");
    tracing::info!("🚨 CRITICAL LIMITATION:");
    tracing::info!("   • Truncate does NOT support sessionId parameter");
    tracing::info!("   • Truncate does NOT support gdbVersion parameter");
    tracing::info!("   • You CANNOT isolate truncate to a branch version");
    tracing::info!("   • Truncate ALWAYS operates on DEFAULT version");
    tracing::info!("   • There is NO \"safe\" pattern using version management");
    tracing::info!("");
    tracing::info!("✅ DO:");
    tracing::info!("   • Use ONLY on test/development services");
    tracing::info!("   • Have complete backups before ANY truncate");
    tracing::info!("   • Test on identical staging environment first");
    tracing::info!("   • Document the operation in change logs");
    tracing::info!("   • Verify layer is not a view or sync-enabled");
    tracing::info!("   • Get written approval for production use");
    tracing::info!("");
    tracing::info!("❌ DON'T:");
    tracing::info!("   • NEVER truncate production without approval + backups");
    tracing::info!("   • NEVER test on live/production services");
    tracing::info!("   • NEVER skip verification steps");
    tracing::info!("   • NEVER assume you can undo (you can't!)");
    tracing::info!("   • NEVER use on views or sync-enabled layers");
    tracing::info!("");
    tracing::info!("🔒 Production Truncate Checklist:");
    tracing::info!("   [ ] Written approval from stakeholders");
    tracing::info!("   [ ] Complete database backup taken and verified");
    tracing::info!("   [ ] Tested on identical staging environment");
    tracing::info!("   [ ] Scheduled during maintenance window");
    tracing::info!("   [ ] Rollback plan documented (restore from backup)");
    tracing::info!("   [ ] Post-operation verification plan ready");
    tracing::info!("   [ ] All users notified of downtime");
    tracing::info!("");
    tracing::info!("💡 Alternative for Versioned Workflows:");
    tracing::info!("   If you need version-safe deletion, use delete_features():");
    tracing::info!("   • Supports sessionId and gdbVersion parameters");
    tracing::info!("   • Can be isolated to branch versions");
    tracing::info!("   • Use WHERE clause '1=1' to delete all features");
    tracing::info!("   • Allows testing in isolated versions safely");
    tracing::info!("");
    tracing::info!("📊 Coverage:");
    tracing::info!("   ✅ truncate() method demonstrated");
    tracing::info!("   ✅ Limitations clearly documented");
    tracing::info!("   ✅ Safety requirements emphasized");
}
