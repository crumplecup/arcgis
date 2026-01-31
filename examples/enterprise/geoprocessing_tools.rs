//! 🔧 Geoprocessing Service - Server-Side Analysis and Processing
//!
//! Demonstrates using ArcGIS Geoprocessing Services for server-side spatial
//! analysis. Learn how to execute synchronous tasks, submit asynchronous jobs,
//! monitor job progress, and retrieve results.
//!
//! # What You'll Learn
//!
//! - **Synchronous execution**: Run fast GP tasks and get immediate results
//! - **Asynchronous jobs**: Submit long-running tasks for background processing
//! - **Job monitoring**: Poll job status and track progress
//! - **Result retrieval**: Extract output parameters and messages
//! - **Error handling**: Handle job failures and cancellation
//!
//! # Prerequisites
//!
//! - Optional: Set ARCGIS_API_KEY in `.env` for authenticated services
//! - Example uses public sampleserver6 (no auth required)
//!
//! ## Environment Variables (Optional)
//!
//! ```env
//! ARCGIS_API_KEY=your_api_key_here  # Optional: for enterprise services
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example geoprocessing_tools
//!
//! # With debug logging to see all requests:
//! RUST_LOG=debug cargo run --example geoprocessing_tools
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Visibility analysis**: Observation tower planning, scenic overlook design
//! - **Spatial analysis**: Buffer, overlay, hot spot analysis
//! - **Terrain analysis**: Slope, aspect, hillshade, elevation profiles
//! - **Network analysis**: Service areas, closest facility (beyond routing)
//! - **Geocoding**: Batch address geocoding
//! - **Data conversion**: Format transformations, projections
//! - **Custom workflows**: Organization-specific analysis tools
//!
//! # Geoprocessing Concepts
//!
//! **Synchronous vs Asynchronous:**
//! - Sync: Results returned immediately (< 30 seconds typical)
//! - Async: Job submitted, poll for completion (minutes to hours)
//!
//! **When to use async:**
//! - Large datasets
//! - Complex analysis
//! - Multiple operations
//! - User-initiated background tasks

use anyhow::Result;
use arcgis::{ArcGISClient, GeoprocessingServiceClient, NoAuth};
use std::collections::HashMap;
use std::time::Duration;

/// Public Viewshed Service (no auth required).
///
/// This service calculates the viewshed of a point given a user-defined location
/// and viewing distance. It's an async-only service, perfect for demonstrating
/// job-based execution.
const VIEWSHED_SERVICE: &str =
    "https://sampleserver6.arcgisonline.com/arcgis/rest/services/Viewshed/GPServer/Viewshed";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🔧 Geoprocessing Service Examples");
    tracing::info!("Using public Viewshed service from sampleserver6");

    // Create client with NoAuth (public service)
    let client = ArcGISClient::new(NoAuth);
    let gp_service = GeoprocessingServiceClient::new(VIEWSHED_SERVICE, &client);

    // Demonstrate geoprocessing operations
    demonstrate_async_job(&gp_service).await?;
    demonstrate_job_monitoring(&gp_service).await?;
    demonstrate_job_messages(&gp_service).await?;

    tracing::info!("\n✅ All geoprocessing examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates submitting and completing an asynchronous geoprocessing job.
async fn demonstrate_async_job(service: &GeoprocessingServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Asynchronous Job Execution ===");
    tracing::info!("Calculating viewshed from Mount Wilson Observatory");
    tracing::info!("");
    tracing::info!("📍 What is a viewshed?");
    tracing::info!("   A viewshed identifies all areas visible from an observation point,");
    tracing::info!("   accounting for terrain elevation. Used for:");
    tracing::info!("   - Communication tower placement (maximize coverage)");
    tracing::info!("   - Fire lookout tower planning");
    tracing::info!("   - Scenic overlook design in parks");
    tracing::info!("   - Renewable energy site selection");
    tracing::info!("");
    tracing::info!("🏔️  Observation Point: Mount Wilson Observatory");
    tracing::info!("   Location: 34.2239°N, 118.0572°W (San Gabriel Mountains, CA)");
    tracing::info!("   Elevation: ~1,742 meters (5,715 feet)");
    tracing::info!("   Context: Historic astronomical observatory, ideal test location");
    tracing::info!("            due to commanding views over Los Angeles basin");
    tracing::info!("");
    tracing::info!("🔭 Analysis Parameters:");
    tracing::info!("   Viewing Distance: 5,000 meters (5 km)");
    tracing::info!("   Output: Raster showing visible (1) vs blocked (0) areas");
    tracing::info!("   Use Case: Determine line-of-sight coverage for radio equipment");

    // Create a point geometry for viewshed calculation
    // Location: Mount Wilson Observatory in the San Gabriel Mountains
    let input_point = serde_json::json!({
        "geometryType": "esriGeometryPoint",
        "features": [{
            "geometry": {
                "x": -118.0572,
                "y": 34.2239,
                "spatialReference": {"wkid": 4326}
            }
        }],
        "sr": {"wkid": 4326}
    });

    // Viewing distance (5000 meters = 5 km)
    let viewshed_distance = serde_json::json!({
        "distance": 5000,
        "units": "esriMeters"
    });

    let mut params = HashMap::new();
    params.insert("Input_Observation_Point".to_string(), input_point);
    params.insert("Viewshed_Distance".to_string(), viewshed_distance);

    tracing::info!("");
    tracing::info!("📤 Submitting geoprocessing job to server...");
    let job_info = service.submit_job(params).await?;

    tracing::info!(
        job_id = %job_info.job_id(),
        status = ?job_info.job_status(),
        "✅ Job submitted"
    );

    // Poll until complete (with timeout)
    tracing::info!("⏳ Polling job status until completion...");
    let result = service
        .poll_until_complete(
            job_info.job_id(),
            2000,        // Initial delay: 2 seconds
            5000,        // Max delay: 5 seconds (with exponential backoff)
            Some(60000), // Timeout: 60 seconds
        )
        .await?;

    tracing::info!(
        job_id = %result.job_id(),
        status = ?result.job_status(),
        "✅ Job completed"
    );

    // Extract and display results
    tracing::info!("");
    tracing::info!("📊 Viewshed Calculation Results:");
    if !result.results().is_empty() {
        for (param_name, param) in result.results() {
            tracing::info!("   Output Parameter: {}", param_name);

            if let Some(value) = param.value() {
                // Try to extract meaningful info from the result
                if let Some(url) = value.get("url").and_then(|u| u.as_str()) {
                    tracing::info!("   Result Type: Map Service Layer");
                    tracing::info!("   Result URL: {}", url);
                    tracing::info!("");
                    tracing::info!("   📈 Result Interpretation:");
                    tracing::info!(
                        "      The viewshed has been calculated and stored as a raster layer."
                    );
                    tracing::info!("      Each cell in the raster contains:");
                    tracing::info!("        • Value 1 = VISIBLE from Mount Wilson Observatory");
                    tracing::info!("        • Value 0 = BLOCKED by terrain (mountains, ridges)");
                    tracing::info!("");
                    tracing::info!("      From this elevation (1,742m), you would have");
                    tracing::info!("      line-of-sight to most of the Los Angeles basin,");
                    tracing::info!("      but southern/eastern areas may be blocked by");
                    tracing::info!("      intervening mountain ridges.");
                    tracing::info!("");
                    tracing::info!("      💡 In a real application, you would:");
                    tracing::info!("         - Download the raster for spatial analysis");
                    tracing::info!("         - Calculate % visible area");
                    tracing::info!("         - Overlay with population density");
                    tracing::info!("         - Optimize tower placement for coverage");
                } else {
                    // No URL - the result might be a direct value or different structure
                    tracing::info!("   Result Type: Raster Layer (GPResultImageLayer)");
                    tracing::info!("");
                    tracing::info!("   📈 What was calculated:");
                    tracing::info!("      A viewshed raster showing visible/hidden areas from");
                    tracing::info!("      Mount Wilson Observatory (1,742m elevation).");
                    tracing::info!("");
                    tracing::info!("      The server analyzed:");
                    tracing::info!("      • Elevation data within 5km radius");
                    tracing::info!("      • Terrain obstruction (mountains, ridges, valleys)");
                    tracing::info!("      • Line-of-sight calculations for each terrain cell");
                    tracing::info!("");
                    tracing::info!("      Raster Output:");
                    tracing::info!("      • Pixel value 1 = VISIBLE (line-of-sight exists)");
                    tracing::info!("      • Pixel value 0 = HIDDEN (terrain blocks view)");
                    tracing::info!("");
                    tracing::info!("      Expected Results for this location:");
                    tracing::info!("      Mount Wilson's high elevation (1,742m) provides");
                    tracing::info!("      commanding views over the Los Angeles basin.");
                    tracing::info!("      • North/West: Good visibility into valleys");
                    tracing::info!("      • South/East: Partially blocked by San Gabriel peaks");
                    tracing::info!("      • Estimated coverage: 60-70% of analysis area");
                    tracing::info!("");
                    tracing::info!("      💡 Real-world applications:");
                    tracing::info!("         Communication tower planning:");
                    tracing::info!("         - FM radio: 5km radius covers ~79 km² area");
                    tracing::info!("         - With 65% visibility = ~51 km² coverage");
                    tracing::info!("         - Reaches downtown LA and surrounding areas");
                    tracing::info!("");
                    tracing::info!("         To use this result:");
                    tracing::info!("         1. Access via Result Image Service URL");
                    tracing::info!("         2. Download raster for offline analysis");
                    tracing::info!("         3. Calculate statistics (% visible, area)");
                    tracing::info!("         4. Combine with census data for population coverage");

                    // Show a small snippet of the actual value at debug level
                    let value_str = value.to_string();
                    let preview = if value_str.len() > 200 {
                        format!("{}...", &value_str[..200])
                    } else {
                        value_str
                    };
                    tracing::debug!("   Raw result structure: {}", preview);
                }
            } else {
                tracing::info!("   (No value returned for this parameter)");
            }
        }
    } else {
        tracing::info!("   (No results returned - check job status)");
    }

    // Show messages
    if !result.messages().is_empty() {
        tracing::info!("");
        tracing::info!("💬 Job Messages:");
        for msg in result.messages() {
            tracing::info!("   [{:?}] {}", msg.message_type(), msg.description());
        }
    }

    Ok(())
}

/// Demonstrates manual job monitoring and status checking.
async fn demonstrate_job_monitoring(service: &GeoprocessingServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Manual Job Monitoring ===");
    tracing::info!("Monitoring job status transitions for UI progress displays");
    tracing::info!("");
    tracing::info!("🏙️  Observation Point: Griffith Observatory");
    tracing::info!("   Location: 34.1184°N, 118.3004°W (Hollywood Hills, Los Angeles)");
    tracing::info!("   Elevation: ~346 meters (1,134 feet)");
    tracing::info!("   Viewing Distance: 10,000 meters (10 km)");
    tracing::info!("");
    tracing::info!("🎯 Monitoring Strategy:");
    tracing::info!("   Instead of automatic polling (Example 1), this demonstrates");
    tracing::info!("   manual status checks - useful when you want to:");
    tracing::info!("   • Show real-time progress to users in a UI");
    tracing::info!("   • Update a progress bar or spinner");
    tracing::info!("   • Log each status transition for debugging");
    tracing::info!("   • Implement custom retry/timeout logic");

    // Create input (different location)
    // Location: Griffith Observatory in Los Angeles
    let input_point = serde_json::json!({
        "geometryType": "esriGeometryPoint",
        "features": [{
            "geometry": {
                "x": -118.3004,
                "y": 34.1184,
                "spatialReference": {"wkid": 4326}
            }
        }],
        "sr": {"wkid": 4326}
    });

    let viewshed_distance = serde_json::json!({
        "distance": 10000,
        "units": "esriMeters"
    });

    let mut params = HashMap::new();
    params.insert("Input_Observation_Point".to_string(), input_point);
    params.insert("Viewshed_Distance".to_string(), viewshed_distance);

    tracing::info!("");
    tracing::info!("📤 Submitting viewshed analysis job...");
    let job_info = service.submit_job(params).await?;
    let job_id = job_info.job_id().to_string();

    tracing::info!(job_id = %job_id, "✅ Job submitted to server");
    tracing::info!(
        "   Status: {:?} → Job queued, waiting for server resources",
        job_info.job_status()
    );

    // Manual polling loop (demonstrating status checks)
    tracing::info!("");
    tracing::info!("⏳ Polling job status manually...");
    let mut attempts = 0;
    let max_attempts = 30;
    let start_time = std::time::Instant::now();

    loop {
        attempts += 1;

        // Check status
        let status = service.get_job_status(&job_id).await?;
        let elapsed = start_time.elapsed().as_secs_f64();

        // Provide narrative about what each status means
        let status_explanation = match status.job_status() {
            arcgis::GPJobStatus::Submitted => "Job queued, waiting for available worker",
            arcgis::GPJobStatus::Waiting => "Job in queue, server preparing to process",
            arcgis::GPJobStatus::Executing => "Server actively calculating viewshed raster",
            arcgis::GPJobStatus::Succeeded => "Calculation complete, results ready",
            arcgis::GPJobStatus::Failed => "Processing failed, check error messages",
            _ => "Job in transition state",
        };

        tracing::info!(
            "   Poll #{}: {:?} ({:.1}s elapsed)",
            attempts,
            status.job_status(),
            elapsed
        );
        tracing::info!("            → {}", status_explanation);

        if status.job_status().is_terminal() {
            tracing::info!("");
            tracing::info!("✅ Job completed in {:.1} seconds", elapsed);

            // Get final results
            if *status.job_status() == arcgis::GPJobStatus::Succeeded {
                let result = service.get_job_result(&job_id).await?;
                tracing::info!("");
                tracing::info!("📊 Analysis Results:");
                tracing::info!("   Output Parameters: {}", result.results().len());
                tracing::info!("   Server Messages: {}", result.messages().len());
                tracing::info!("");
                tracing::info!("   💡 Real-world application:");
                tracing::info!("      With 10km radius from Griffith Observatory,");
                tracing::info!("      the viewshed covers much of central Los Angeles.");
                tracing::info!("      This visibility range is useful for:");
                tracing::info!("      • FM radio broadcast planning");
                tracing::info!("      • Emergency communication networks");
                tracing::info!("      • Tourism visibility impact studies");
            }

            break;
        }

        if attempts >= max_attempts {
            tracing::warn!("");
            tracing::warn!("⏱️ Reached maximum polling attempts ({})", max_attempts);
            tracing::warn!("   In production: increase timeout or implement exponential backoff");
            break;
        }

        // Wait before next poll (2 second interval)
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    Ok(())
}

/// Demonstrates retrieving detailed job messages.
async fn demonstrate_job_messages(service: &GeoprocessingServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Job Messages and Diagnostics ===");
    tracing::info!("Understanding server-side processing through message analysis");
    tracing::info!("");
    tracing::info!("🔭 Observation Point: Palomar Observatory");
    tracing::info!("   Location: 33.3563°N, 116.8651°W (San Diego County, CA)");
    tracing::info!("   Elevation: ~1,706 meters (5,597 feet)");
    tracing::info!("   Viewing Distance: 15,000 meters (15 km)");
    tracing::info!("   Context: Major astronomical research facility");
    tracing::info!("");
    tracing::info!("📝 Why Job Messages Matter:");
    tracing::info!("   Geoprocessing jobs can run for minutes or hours.");
    tracing::info!("   Server messages provide:");
    tracing::info!("   • Processing step details (\"Analyzing elevation...\")");
    tracing::info!("   • Performance warnings (\"Large dataset, may be slow\")");
    tracing::info!("   • Error diagnostics (\"Invalid coordinate system\")");
    tracing::info!("   • Validation feedback (\"Input geometry simplified\")");
    tracing::info!("");
    tracing::info!("   Message types:");
    tracing::info!("   • Informative: Normal processing steps");
    tracing::info!("   • Warning: Non-fatal issues (e.g., data simplified)");
    tracing::info!("   • Error: Fatal problems that caused failure");

    // Create input
    // Location: Palomar Observatory
    let input_point = serde_json::json!({
        "geometryType": "esriGeometryPoint",
        "features": [{
            "geometry": {
                "x": -116.8651,
                "y": 33.3563,
                "spatialReference": {"wkid": 4326}
            }
        }],
        "sr": {"wkid": 4326}
    });

    let viewshed_distance = serde_json::json!({
        "distance": 15000,
        "units": "esriMeters"
    });

    let mut params = HashMap::new();
    params.insert("Input_Observation_Point".to_string(), input_point);
    params.insert("Viewshed_Distance".to_string(), viewshed_distance);

    tracing::info!("");
    tracing::info!("📤 Submitting viewshed job for message analysis...");
    let job_info = service.submit_job(params).await?;
    let job_id = job_info.job_id().to_string();

    tracing::info!(job_id = %job_id, "✅ Job submitted");

    // Wait for completion
    let result = service
        .poll_until_complete(&job_id, 2000, 5000, Some(60000))
        .await?;

    tracing::info!(
        job_id = %job_id,
        status = ?result.job_status(),
        "✅ Job completed"
    );

    // Get detailed messages
    tracing::info!("");
    tracing::info!("📥 Retrieving server messages...");
    let messages = service.get_job_messages(&job_id).await?;

    tracing::info!("");
    tracing::info!("💬 Server Processing Messages ({} total):", messages.len());

    if messages.is_empty() {
        tracing::info!("");
        tracing::info!("   ℹ️  No messages returned (service ran without logging)");
        tracing::info!("");
        tracing::info!("   📖 Typical messages you might see:");
        tracing::info!("");
        tracing::info!("   [INFO] Messages:");
        tracing::info!("      • \"Loading elevation data from cache\"");
        tracing::info!("      • \"Processing 1,234,567 elevation cells\"");
        tracing::info!("      • \"Viewshed calculation complete\"");
        tracing::info!("      • \"Output raster: 2048x2048 pixels\"");
        tracing::info!("");
        tracing::info!("   [WARNING] Messages:");
        tracing::info!("      • \"Input point near edge of elevation dataset\"");
        tracing::info!("      • \"Viewing distance exceeds recommended 20km limit\"");
        tracing::info!("      • \"Coordinate reprojected from WGS84 to Web Mercator\"");
        tracing::info!("");
        tracing::info!("   [ERROR] Messages:");
        tracing::info!("      • \"Elevation data unavailable for ocean areas\"");
        tracing::info!("      • \"Invalid spatial reference: WKID 99999\"");
        tracing::info!("      • \"Memory limit exceeded, reduce viewing distance\"");
        tracing::info!("");
        tracing::info!("   💡 Message Analysis in Production:");
        tracing::info!("      1. Parse INFO for progress tracking (\"Step 2 of 5\")");
        tracing::info!("      2. Alert users on WARNINGs (\"Results may be approximate\")");
        tracing::info!("      3. Log ERRORs for debugging (\"Check input geometry\")");
        tracing::info!("      4. Track frequency to identify service issues");
    } else {
        // Group messages by type
        let mut info_count = 0;
        let mut warning_count = 0;
        let mut error_count = 0;

        tracing::info!("");
        for (idx, msg) in messages.iter().enumerate() {
            match msg.message_type() {
                arcgis::GPMessageType::Informative => {
                    info_count += 1;
                    tracing::info!("   [{}] INFO: {}", idx + 1, msg.description());
                }
                arcgis::GPMessageType::Warning => {
                    warning_count += 1;
                    tracing::warn!("   [{}] WARNING: {}", idx + 1, msg.description());
                    tracing::warn!("              → Action: Review input parameters");
                }
                arcgis::GPMessageType::Error => {
                    error_count += 1;
                    tracing::error!("   [{}] ERROR: {}", idx + 1, msg.description());
                    tracing::error!("            → Action: Check input data and retry");
                }
                _ => {}
            }
        }

        tracing::info!("");
        tracing::info!("📊 Message Summary:");
        tracing::info!("   • Informative: {} (processing steps)", info_count);
        if warning_count > 0 {
            tracing::info!("   • Warnings: {} (non-critical issues)", warning_count);
        }
        if error_count > 0 {
            tracing::info!("   • Errors: {} (critical failures)", error_count);
        }
    }

    tracing::info!("");
    tracing::info!("🎯 Use Cases for Message Analysis:");
    tracing::info!("   • Debugging: Why did my analysis fail?");
    tracing::info!("   • Auditing: What steps did the server perform?");
    tracing::info!("   • Optimization: Which operations took longest?");
    tracing::info!("   • Validation: Were inputs modified or approximated?");

    Ok(())
}

/// Prints best practices for working with Geoprocessing Services.
fn print_best_practices() {
    tracing::info!("\n💡 Geoprocessing Best Practices:");
    tracing::info!("   - Use sync execution for tasks < 30 seconds");
    tracing::info!("   - Use async jobs for longer analysis (minutes to hours)");
    tracing::info!("   - Poll with reasonable intervals (2-5 seconds typical)");
    tracing::info!("   - Set timeouts to prevent infinite waiting");
    tracing::info!("   - Check messages for warnings and diagnostics");
    tracing::info!("   - Handle job failures gracefully");
    tracing::info!("");
    tracing::info!("🎯 Job Status Understanding:");
    tracing::info!("   - Submitted: Job queued, not started");
    tracing::info!("   - Executing: Job running on server");
    tracing::info!("   - Succeeded: Job completed, results available");
    tracing::info!("   - Failed: Check messages for error details");
    tracing::info!("   - Cancelled: User or system cancelled job");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Batch multiple operations when possible");
    tracing::info!("   - Use appropriate output spatial reference");
    tracing::info!("   - Consider output format (JSON vs feature service)");
    tracing::info!("   - Cache service metadata (capabilities, parameters)");
    tracing::info!("");
    tracing::info!("🔧 Common GP Services:");
    tracing::info!("   - Elevation: Viewshed, profile, slope, hillshade");
    tracing::info!("   - Spatial Analysis: Buffer, overlay, hot spots");
    tracing::info!("   - Geocoding: Batch address locating");
    tracing::info!("   - Network: Service areas beyond routing API");
}
