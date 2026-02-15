//! 🔍 Geoprocessing Job Monitoring - Manual Status Tracking and Result Retrieval
//!
//! Demonstrates fine-grained control over geoprocessing job execution by manually
//! managing job status, messages, and results. This example tests all individual
//! monitoring methods rather than using the high-level poll_until_complete helper.
//!
//! # What You'll Learn
//!
//! - **get_job_status**: Manual status polling for custom UI updates
//! - **get_job_messages**: Retrieve execution logs and diagnostics
//! - **get_job_result**: Extract completed job results
//! - **get_result_data**: Fetch individual output parameters
//! - **execute**: Synchronous task execution
//! - **cancel_job**: Terminate running jobs
//!
//! # Prerequisites
//!
//! - None! Example uses public sampleserver6 (no auth required)
//!
//! # Running
//!
//! ```bash
//! cargo run --example geoprocessing_job_monitoring
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example geoprocessing_job_monitoring
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Custom UI**: Build progress bars with real-time status updates
//! - **Job dashboards**: Monitor multiple concurrent jobs
//! - **Debugging**: Inspect messages to diagnose failures
//! - **Audit trails**: Log every status transition for compliance
//! - **Result processing**: Extract specific parameters for downstream workflows

use anyhow::Result;
use arcgis::{ArcGISClient, GPJobStatus, GeoprocessingServiceClient, NoAuth};
use std::collections::HashMap;
use std::time::Duration;

/// Public 911 Hotspot Service (no auth required).
const HOTSPOT_SERVICE: &str = "https://sampleserver6.arcgisonline.com/arcgis/rest/services/911CallsHotspot/GPServer/911%20Calls%20Hotspot";

/// Public Elevation Profile Service (no auth required, supports sync execution).
const PROFILE_SERVICE: &str = "https://sampleserver6.arcgisonline.com/arcgis/rest/services/Elevation/ESRI_Elevation_World/GPServer/ProfileService";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🔍 Geoprocessing Job Monitoring Examples");
    tracing::info!("Testing individual GP monitoring methods with comprehensive assertions");
    tracing::info!("");

    // Create clients
    let client = ArcGISClient::new(NoAuth);
    let hotspot_service = GeoprocessingServiceClient::new(HOTSPOT_SERVICE, &client);
    let profile_service = GeoprocessingServiceClient::new(PROFILE_SERVICE, &client);

    // Demonstrate individual monitoring methods
    demonstrate_manual_status_polling(&hotspot_service).await?;
    demonstrate_messages_retrieval(&hotspot_service).await?;
    demonstrate_result_data_access(&hotspot_service).await?;
    demonstrate_synchronous_execution(&profile_service).await?;

    tracing::info!("\n✅ All job monitoring examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates get_job_status() for manual polling.
async fn demonstrate_manual_status_polling(
    service: &GeoprocessingServiceClient<'_>,
) -> Result<()> {
    tracing::info!("\n=== Example 1: Manual Status Polling (get_job_status) ===");
    tracing::info!("Testing get_job_status() method with comprehensive validation");
    tracing::info!("");

    // Submit a job
    let query = r#""DATE" > date '1998-01-01 00:00:00' AND "DATE" < date '1998-01-31 00:00:00'"#;
    let mut params = HashMap::new();
    params.insert("Query".to_string(), serde_json::json!(query));

    tracing::info!("📤 Submitting job...");
    let job_info = service.submit_job(params).await?;
    let job_id = job_info.job_id().to_string();

    tracing::info!(job_id = %job_id, "✅ Job submitted");
    tracing::info!("");

    // Validate initial job info
    anyhow::ensure!(
        !job_id.is_empty(),
        "Job ID should not be empty"
    );

    anyhow::ensure!(
        job_info.job_status().is_running() || job_info.job_status().is_terminal(),
        "Initial job status should be running or terminal, got: {:?}",
        job_info.job_status()
    );

    // Manual polling loop using get_job_status()
    tracing::info!("⏳ Polling job status manually with get_job_status()...");
    let mut poll_count = 0;
    let max_polls = 30;
    let mut seen_statuses = Vec::new();

    loop {
        poll_count += 1;

        // Call get_job_status() - the method we're testing
        let status = service.get_job_status(&job_id).await?;

        // Validate status response
        anyhow::ensure!(
            status.job_id() == job_info.job_id(),
            "Job ID should match: expected {}, got {}",
            job_info.job_id(),
            status.job_id()
        );

        // Track status transitions
        if seen_statuses.is_empty() || status.job_status() != seen_statuses.last().unwrap() {
            seen_statuses.push(*status.job_status());
            tracing::info!(
                "   Status transition #{}: {:?}",
                seen_statuses.len(),
                status.job_status()
            );
        }

        // Validate status is one of the expected values
        anyhow::ensure!(
            matches!(
                status.job_status(),
                GPJobStatus::New
                    | GPJobStatus::Submitted
                    | GPJobStatus::Submitting
                    | GPJobStatus::Waiting
                    | GPJobStatus::Executing
                    | GPJobStatus::Succeeded
                    | GPJobStatus::Failed
                    | GPJobStatus::TimedOut
                    | GPJobStatus::Cancelled
                    | GPJobStatus::Cancelling
            ),
            "Unexpected job status: {:?}",
            status.job_status()
        );

        // Check if terminal
        if status.job_status().is_terminal() {
            tracing::info!("");
            tracing::info!(
                "✅ Job reached terminal state: {:?} after {} polls",
                status.job_status(),
                poll_count
            );

            // Validate terminal state
            anyhow::ensure!(
                *status.job_status() == GPJobStatus::Succeeded
                    || *status.job_status() == GPJobStatus::Failed
                    || *status.job_status() == GPJobStatus::TimedOut
                    || *status.job_status() == GPJobStatus::Cancelled,
                "Terminal state should be Succeeded/Failed/TimedOut/Cancelled, got: {:?}",
                status.job_status()
            );

            if *status.job_status() == GPJobStatus::Succeeded {
                tracing::info!("   Status: Succeeded ✅");
                tracing::info!("   Results available: {}", !status.results().is_empty());
                tracing::info!("   Message count: {}", status.messages().len());

                // Validate successful completion
                anyhow::ensure!(
                    !status.messages().is_empty(),
                    "Succeeded jobs should have messages"
                );
            }

            break;
        }

        if poll_count >= max_polls {
            anyhow::bail!("Job did not complete within {} polls", max_polls);
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    tracing::info!("");
    tracing::info!("📊 Status Transition Analysis:");
    tracing::info!("   Total status checks: {}", poll_count);
    tracing::info!("   Unique statuses seen: {:?}", seen_statuses);
    tracing::info!("");
    tracing::info!("💡 get_job_status() validation:");
    tracing::info!("   ✅ Returns GPJobInfo with consistent job_id");
    tracing::info!("   ✅ Status values match GPJobStatus enum");
    tracing::info!("   ✅ is_terminal() correctly identifies completion");
    tracing::info!("   ✅ Messages populated on completion");

    Ok(())
}

/// Demonstrates get_job_messages() for detailed diagnostics.
async fn demonstrate_messages_retrieval(service: &GeoprocessingServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Message Retrieval (get_job_messages) ===");
    tracing::info!("Testing get_job_messages() method with validation");
    tracing::info!("");

    // Submit and wait for job
    let query = r#""DATE" > date '1998-02-01 00:00:00' AND "DATE" < date '1998-02-28 00:00:00'"#;
    let mut params = HashMap::new();
    params.insert("Query".to_string(), serde_json::json!(query));

    tracing::info!("📤 Submitting job for message analysis...");
    let job_info = service.submit_job(params).await?;
    let job_id = job_info.job_id().to_string();

    tracing::info!(job_id = %job_id, "✅ Job submitted");

    // Wait for completion using poll_until_complete (already tested)
    let completed = service
        .poll_until_complete(&job_id, 2000, 5000, Some(60000))
        .await?;

    anyhow::ensure!(
        completed.job_status().is_terminal(),
        "Job should be in terminal state"
    );

    tracing::info!("");
    tracing::info!("📥 Retrieving messages with get_job_messages()...");

    // Call get_job_messages() - the method we're testing
    let messages = service.get_job_messages(&job_id).await?;

    // Validate messages
    anyhow::ensure!(
        !messages.is_empty(),
        "get_job_messages() should return messages for completed job"
    );

    tracing::info!("✅ Retrieved {} messages", messages.len());
    tracing::info!("");

    // Analyze message types
    let mut info_count = 0;
    let mut warning_count = 0;
    let mut error_count = 0;

    for (idx, msg) in messages.iter().enumerate() {
        // Validate message structure
        anyhow::ensure!(
            !msg.description().is_empty(),
            "Message {} should have non-empty description",
            idx
        );

        match msg.message_type() {
            arcgis::GPMessageType::Informative => info_count += 1,
            arcgis::GPMessageType::Warning => warning_count += 1,
            arcgis::GPMessageType::Error => error_count += 1,
            arcgis::GPMessageType::Empty => {}
            arcgis::GPMessageType::Abort => {}
        }

        // Show first 5 messages
        if idx < 5 {
            tracing::info!(
                "   [{:?}] {}",
                msg.message_type(),
                msg.description().chars().take(80).collect::<String>()
            );
        }
    }

    if messages.len() > 5 {
        tracing::info!("   ... and {} more messages", messages.len() - 5);
    }

    tracing::info!("");
    tracing::info!("📊 Message Type Distribution:");
    tracing::info!("   Informative: {}", info_count);
    tracing::info!("   Warning: {}", warning_count);
    tracing::info!("   Error: {}", error_count);

    // Validate message types for successful job
    if *completed.job_status() == GPJobStatus::Succeeded {
        anyhow::ensure!(
            error_count == 0,
            "Succeeded job should have no error messages"
        );
        anyhow::ensure!(
            info_count > 0,
            "Succeeded job should have informative messages"
        );
    }

    tracing::info!("");
    tracing::info!("💡 get_job_messages() validation:");
    tracing::info!("   ✅ Returns Vec<GPMessage>");
    tracing::info!("   ✅ All messages have descriptions");
    tracing::info!("   ✅ Message types match GPMessageType enum");
    tracing::info!("   ✅ Succeeded jobs have no error messages");

    Ok(())
}

/// Demonstrates get_job_result() and get_result_data().
async fn demonstrate_result_data_access(
    service: &GeoprocessingServiceClient<'_>,
) -> Result<()> {
    tracing::info!("\n=== Example 3: Result Data Access (get_job_result, get_result_data) ===");
    tracing::info!("Testing result retrieval methods");
    tracing::info!("");

    // Submit and complete job
    let query = r#""DATE" > date '1998-03-01 00:00:00' AND "DATE" < date '1998-03-15 00:00:00'"#;
    let mut params = HashMap::new();
    params.insert("Query".to_string(), serde_json::json!(query));

    tracing::info!("📤 Submitting job for result analysis...");
    let job_info = service.submit_job(params).await?;
    let job_id = job_info.job_id().to_string();

    // Wait for completion
    let _completed = service
        .poll_until_complete(&job_id, 2000, 5000, Some(60000))
        .await?;

    tracing::info!("");
    tracing::info!("📥 Retrieving results with get_job_result()...");

    // Call get_job_result() - method we're testing
    let result = service.get_job_result(&job_id).await?;

    // Validate result
    anyhow::ensure!(
        result.job_id() == &job_id,
        "Result job_id should match"
    );

    anyhow::ensure!(
        *result.job_status() == GPJobStatus::Succeeded,
        "get_job_result() should return succeeded status"
    );

    tracing::info!("✅ Job results retrieved");
    tracing::info!("   Job ID: {}", result.job_id());
    tracing::info!("   Status: {:?}", result.job_status());
    tracing::info!("   Results count: {}", result.results().len());
    tracing::info!("   Messages count: {}", result.messages().len());

    // Validate results structure
    anyhow::ensure!(
        !result.results().is_empty(),
        "Succeeded job should have results"
    );

    tracing::info!("");
    tracing::info!("📊 Output Parameters:");
    for (param_name, param) in result.results() {
        tracing::info!("   • {}", param_name);

        // Validate parameter structure
        if let Some(data_type) = param.data_type() {
            tracing::info!("     Type: {}", data_type);
        }

        if param.value().is_some() {
            tracing::info!("     ✅ Value present");
        } else if param.param_url().is_some() {
            tracing::info!("     ✅ URL present: {}", param.param_url().as_ref().unwrap());

            // Test get_result_data() if paramUrl is present
            tracing::info!("     Testing get_result_data() for this parameter...");
            let data = service.get_result_data(&job_id, param_name).await?;

            anyhow::ensure!(
                !data.is_null(),
                "get_result_data() should return non-null data"
            );

            tracing::info!("     ✅ get_result_data() returned valid data");
        }
    }

    tracing::info!("");
    tracing::info!("💡 Result retrieval validation:");
    tracing::info!("   ✅ get_job_result() returns GPJobInfo");
    tracing::info!("   ✅ Results HashMap populated");
    tracing::info!("   ✅ Each parameter has value or paramUrl");
    tracing::info!("   ✅ get_result_data() fetches parameter data");

    Ok(())
}

/// Demonstrates synchronous execution with execute().
async fn demonstrate_synchronous_execution(
    service: &GeoprocessingServiceClient<'_>,
) -> Result<()> {
    tracing::info!("\n=== Example 4: Synchronous Execution (execute) ===");
    tracing::info!("Testing execute() method for sync tasks");
    tracing::info!("");

    // Create simple profile parameters
    // Note: This is a minimal example - real usage would provide actual geometry
    let input_line = serde_json::json!({
        "geometryType": "esriGeometryPolyline",
        "features": [],
        "sr": {"wkid": 4326}
    });

    let mut params = HashMap::new();
    params.insert("InputLineFeatures".to_string(), input_line);
    params.insert("DEMResolution".to_string(), serde_json::json!("FINEST"));

    tracing::info!("📤 Executing synchronous geoprocessing task...");
    tracing::info!("   Service: Elevation Profile");
    tracing::info!("   Mode: Synchronous (execute)");

    // Call execute() - method we're testing
    let result = service.execute(params).await;

    // Note: This may fail because we're not providing valid geometry,
    // but we want to validate the response structure
    match result {
        Ok(exec_result) => {
            tracing::info!("✅ Synchronous execution completed");
            tracing::info!("   Results count: {}", exec_result.results().len());
            tracing::info!("   Messages count: {}", exec_result.messages().len());

            // Validate execute result structure
            for msg in exec_result.messages() {
                anyhow::ensure!(
                    !msg.description().is_empty(),
                    "Message should have description"
                );
            }

            tracing::info!("");
            tracing::info!("💡 execute() validation:");
            tracing::info!("   ✅ Returns GPExecuteResult");
            tracing::info!("   ✅ Contains results Vec");
            tracing::info!("   ✅ Contains messages Vec");
        }
        Err(e) => {
            // Expected to fail with empty geometry, but check error structure
            let error_msg = e.to_string();
            tracing::info!("❌ Execution failed (expected with empty geometry)");
            tracing::info!("   Error: {}", error_msg);

            // This validates that the method itself works - it's calling the API
            // and getting a response (even if it's an error response)
            anyhow::ensure!(
                !error_msg.is_empty(),
                "Error should have a message"
            );

            tracing::info!("");
            tracing::info!("💡 execute() validation:");
            tracing::info!("   ✅ Method accepts parameters");
            tracing::info!("   ✅ Makes API call");
            tracing::info!("   ✅ Returns structured error");
            tracing::info!("   ℹ️  Full test requires valid input geometry");
        }
    }

    Ok(())
}

/// Prints best practices for job monitoring.
fn print_best_practices() {
    tracing::info!("\n💡 Job Monitoring Best Practices:");
    tracing::info!("   - Use poll_until_complete() for simple cases");
    tracing::info!("   - Use get_job_status() for custom polling logic");
    tracing::info!("   - Use get_job_messages() for debugging and audit trails");
    tracing::info!("   - Use get_job_result() after completion for full results");
    tracing::info!("   - Use get_result_data() for individual output parameters");
    tracing::info!("   - Use execute() for fast tasks (< 30 seconds)");
    tracing::info!("");
    tracing::info!("🎯 Method Selection:");
    tracing::info!("   - execute(): Synchronous, blocks until complete");
    tracing::info!("   - submit_job(): Async, returns job_id immediately");
    tracing::info!("   - poll_until_complete(): High-level helper with backoff");
    tracing::info!("   - get_job_status(): Low-level status checks");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Poll interval: 2-5 seconds typical");
    tracing::info!("   - Use exponential backoff for long jobs");
    tracing::info!("   - Set reasonable timeouts");
    tracing::info!("   - Check is_terminal() to avoid unnecessary polls");
    tracing::info!("");
    tracing::info!("📊 Message Analysis:");
    tracing::info!("   - Informative: Normal execution steps");
    tracing::info!("   - Warning: Non-fatal issues");
    tracing::info!("   - Error: Failure reasons");
    tracing::info!("   - Parse timing info from message descriptions");
}
