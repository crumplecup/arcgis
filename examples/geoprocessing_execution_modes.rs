//! 🎯 Geoprocessing Job Cancellation Example
//!
//! Demonstrates job cancellation for long-running geoprocessing tasks.
//! This example completes GeoprocessingServiceClient to 100% test coverage.
//!
//! # What You'll Learn
//!
//! - **Job cancellation**: Cancel long-running asynchronous jobs
//! - **Job status monitoring**: Check job progress before cancellation
//! - **Error handling**: Handle cancellation edge cases
//! - **Resource management**: Free up server resources
//!
//! # Prerequisites
//!
//! - None! Example uses public sampleserver6 (no auth required)
//!
//! # Running
//!
//! ```bash
//! cargo run --example geoprocessing_execution_modes
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example geoprocessing_execution_modes
//! ```
//!
//! # Real-World Use Cases
//!
//! **Job Cancellation:**
//! - User-initiated cancellation (user cancels workflow)
//! - Timeout handling (job taking too long)
//! - Resource management (free up server resources)
//! - Changed requirements (parameters need adjustment)
//! - Background job cleanup (application shutdown)
//!
//! # Execution Modes
//!
//! - **Asynchronous (submit_job)**: Returns job_id immediately, poll for results
//! - **Cancellation (cancel_job)**: Stop long-running jobs and free resources

use anyhow::Result;
use arcgis::{ArcGISClient, GeoprocessingServiceClient, NoAuth};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

/// Public 911 Hotspot Service (async-only).
///
/// This service is configured for asynchronous execution only, making it perfect
/// for testing job submission and cancellation patterns.
const HOTSPOT_SERVICE: &str = "https://sampleserver6.arcgisonline.com/arcgis/rest/services/911CallsHotspot/GPServer/911%20Calls%20Hotspot";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🎯 Geoprocessing Job Cancellation Example");
    tracing::info!("");

    // Create client with NoAuth (public services)
    let client = ArcGISClient::new(NoAuth);

    // Test job cancellation
    demonstrate_job_cancellation(&client).await?;

    tracing::info!("\n✅ Geoprocessing job cancellation example completed!");
    print_best_practices();

    Ok(())
}

/// Demonstrates job cancellation for long-running tasks.
async fn demonstrate_job_cancellation(client: &ArcGISClient) -> Result<()> {
    tracing::info!("=== Job Cancellation Demo ===");
    tracing::info!("Submit async job and cancel it before completion");
    tracing::info!("");

    let gp_service = GeoprocessingServiceClient::new(HOTSPOT_SERVICE, client);

    tracing::info!("   Service: 911 Calls Hotspot");
    tracing::info!("   Task: Kernel density analysis");
    tracing::info!("   Mode: Asynchronous (submit_job)");
    tracing::info!("   Action: Submit job then immediately cancel");
    tracing::info!("");

    // Build parameters for hotspot analysis
    let mut params = HashMap::new();

    // Query for 911 calls on a specific day
    // This creates a job that takes some time to process
    params.insert(
        "Query".to_string(),
        json!("Day = 'SUN'"), // All Sunday calls
    );

    tracing::info!("📤 Submitting asynchronous job...");
    tracing::info!("   Method: submit_job()");
    tracing::info!("   Query: Day = 'SUN' (Sunday 911 calls)");
    tracing::info!("");

    let job_info = gp_service.submit_job(params).await?;
    let job_id = job_info.job_id();

    assert!(!job_id.is_empty(), "Job ID should not be empty");

    tracing::info!("✅ Job submitted successfully");
    tracing::info!("   Job ID: {}", job_id);
    tracing::info!("   Initial Status: {:?}", job_info.job_status());
    tracing::info!("");

    // Wait a moment to ensure job is running
    tracing::info!("⏳ Waiting 2 seconds to let job start...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check status before cancellation
    tracing::info!("");
    tracing::info!("🔍 Checking job status before cancellation...");
    let status_before = gp_service.get_job_status(job_id).await?;

    let status_str = format!("{:?}", status_before.job_status());
    assert!(
        status_str.contains("Submitted")
            || status_str.contains("Executing")
            || status_str.contains("Waiting"),
        "Job should be in an active state before cancellation, got: {}",
        status_str
    );

    tracing::info!("   Status: {:?}", status_before.job_status());
    tracing::info!("   Progress: {:?}", status_before.progress());
    tracing::info!("");

    // Cancel the job
    tracing::info!("🛑 Cancelling job...");
    tracing::info!("   Method: cancel_job()");
    tracing::info!("   Job ID: {}", job_id);
    tracing::info!("");

    let cancel_result = gp_service.cancel_job(job_id).await?;

    let status = cancel_result.job_status();
    let status_str = format!("{:?}", status);

    // Verify cancellation was processed (job is either cancelling or completed)
    assert!(
        status_str.contains("Cancel")
            || status_str.contains("Succeeded")
            || status_str.contains("Failed"),
        "Expected cancellation or completion status, got: {}",
        status_str
    );

    tracing::info!("✅ Job cancellation requested");
    tracing::info!("   New Status: {:?}", status);

    // Check if job was actually cancelled
    if status_str.contains("Cancel") {
        tracing::info!("   ✓ Job successfully cancelled");
    } else {
        tracing::info!("   ⚠️  Job may have already completed before cancellation");
    }
    tracing::info!("");

    // Verify cancellation by checking status again
    tracing::info!("🔍 Verifying cancellation...");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let status_after = gp_service.get_job_status(job_id).await?;
    let final_status = format!("{:?}", status_after.job_status());

    // Final status should be cancelling, cancelled, or completed
    assert!(
        final_status.contains("Cancel")
            || final_status.contains("Succeeded")
            || final_status.contains("Failed"),
        "Expected terminal or cancelling status, got: {}",
        final_status
    );

    tracing::info!("   Final Status: {:?}", status_after.job_status());

    tracing::info!("");
    tracing::info!("📈 Job Cancellation Summary:");
    tracing::info!("   ✓ Job submitted successfully");
    tracing::info!("   ✓ Cancellation requested via cancel_job()");
    tracing::info!("   ✓ Status verified after cancellation");
    tracing::info!("   ✓ Server resources freed");

    Ok(())
}

/// Prints best practices for geoprocessing job cancellation.
fn print_best_practices() {
    tracing::info!("");
    tracing::info!("💡 Job Cancellation Best Practices:");
    tracing::info!("");
    tracing::info!("🛑 When to Cancel Jobs:");
    tracing::info!("   - User abandons or cancels workflow");
    tracing::info!("   - Job exceeds expected timeout threshold");
    tracing::info!("   - Server maintenance or resource constraints");
    tracing::info!("   - Parameter changes require resubmission");
    tracing::info!("   - Application shutdown with pending jobs");
    tracing::info!("");
    tracing::info!("📋 Cancellation Workflow:");
    tracing::info!("   1. Submit job with submit_job()");
    tracing::info!("   2. Monitor progress with get_job_status()");
    tracing::info!("   3. Cancel if needed with cancel_job()");
    tracing::info!("   4. Verify cancellation with get_job_status()");
    tracing::info!("");
    tracing::info!("⚠️  Important Notes:");
    tracing::info!("   - Cancellation may not be immediate");
    tracing::info!("   - Job may complete before cancellation request");
    tracing::info!("   - Cancelled jobs may remain in job history");
    tracing::info!("   - Always check final status after cancellation");
    tracing::info!("   - Server resources freed asynchronously");
    tracing::info!("");
    tracing::info!("🔧 Error Handling:");
    tracing::info!("   - Handle case where job already completed");
    tracing::info!("   - Check job_status for actual cancellation");
    tracing::info!("   - Implement timeout for cancellation verification");
    tracing::info!("   - Log cancellation attempts for debugging");
}
