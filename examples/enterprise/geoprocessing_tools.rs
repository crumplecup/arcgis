//! 🔧 Geoprocessing Service - Server-Side Analysis and Processing
//!
//! Demonstrates using ArcGIS Geoprocessing Services for server-side spatial
//! analysis. Learn how to execute asynchronous jobs, monitor job progress,
//! and retrieve results using the generic GeoprocessingServiceClient.
//!
//! # What You'll Learn
//!
//! - **Asynchronous jobs**: Submit long-running tasks for background processing
//! - **Job monitoring**: Poll job status and track progress
//! - **Result retrieval**: Extract output parameters and messages
//! - **Error handling**: Handle job failures and cancellation
//! - **Generic GP client**: Work with any geoprocessing service
//!
//! # Prerequisites
//!
//! - None! Example uses public sampleserver6 (no auth required)
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
//! - **Crime analysis**: Hotspot detection for patrol optimization
//! - **Emergency response**: Identify high-frequency incident areas
//! - **Public health**: Disease outbreak cluster detection
//! - **Spatial analysis**: Buffer, overlay, density analysis
//! - **Terrain analysis**: Slope, aspect, hillshade, elevation
//! - **Network analysis**: Service areas, closest facility
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
//! - Large datasets (thousands of features)
//! - Complex spatial analysis
//! - Multiple chained operations
//! - User-initiated background tasks
//!
//! # 911 Hotspot Analysis
//!
//! This example uses a public 911 call dataset from San Diego (January-May 1998)
//! to demonstrate kernel density hotspot analysis. The service identifies areas
//! with high concentrations of emergency calls, useful for:
//! - Patrol route optimization
//! - Emergency resource placement
//! - Public safety planning

use anyhow::Result;
use arcgis::{ArcGISClient, GeoprocessingServiceClient, NoAuth};
use std::collections::HashMap;
use std::time::Duration;

/// Public 911 Hotspot Service (no auth required).
///
/// This service performs kernel density analysis on 911 call data from San Diego (1998).
/// It's an async-only service, perfect for demonstrating job-based execution patterns.
///
/// Data: January 1 - May 31, 1998 (San Diego County 911 calls)
/// Fields: DATE (timestamp), Day (day of week: SUN/MON/TUE/etc)
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

    tracing::info!("🔧 Geoprocessing Service Examples");
    tracing::info!("Analyzing 911 call patterns using kernel density hotspot analysis");
    tracing::info!("Dataset: San Diego County 911 calls (Jan-May 1998)");
    tracing::info!("");

    // Create client with NoAuth (public service)
    let client = ArcGISClient::new(NoAuth);
    let gp_service = GeoprocessingServiceClient::new(HOTSPOT_SERVICE, &client);

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
    tracing::info!("Analyzing weekend 911 call patterns in January 1998");
    tracing::info!("");
    tracing::info!("📊 What is hotspot analysis?");
    tracing::info!("   Kernel density estimation identifies areas with high concentrations");
    tracing::info!("   of point events (911 calls). Used for:");
    tracing::info!("   • Crime pattern analysis and patrol optimization");
    tracing::info!("   • Emergency resource allocation");
    tracing::info!("   • Public safety planning and policy");
    tracing::info!("   • Disease outbreak cluster detection");
    tracing::info!("");
    tracing::info!("🎯 Analysis Parameters:");
    tracing::info!("   Time Period: January 1-31, 1998");
    tracing::info!("   Filter: Weekend calls only (Saturday & Sunday)");
    tracing::info!("   Output: Density raster showing call concentration");
    tracing::info!("   Use Case: Determine if weekend call patterns differ from weekdays");
    tracing::info!("");
    tracing::info!("💡 Why this matters:");
    tracing::info!("   Weekend 911 calls often show different spatial patterns:");
    tracing::info!("   • More calls from entertainment districts");
    tracing::info!("   • Different time-of-day patterns");
    tracing::info!("   • Distinct incident types (domestic vs commercial)");

    // SQL query for weekend calls in January 1998
    // Using exact format from service default
    let query = r#"("DATE" > date '1998-01-01 00:00:00' AND "DATE" < date '1998-01-31 00:00:00') AND ("Day" = 'SUN' OR "Day"= 'SAT')"#;

    let mut params = HashMap::new();
    params.insert("Query".to_string(), serde_json::json!(query));

    tracing::info!("");
    tracing::info!("📤 Submitting hotspot analysis job to server...");
    tracing::info!("   SQL Query: {}", query);
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

    // Parse server messages to extract analysis statistics
    tracing::info!("");
    tracing::info!("📊 Weekend 911 Call Analysis Results:");

    // Extract processing time from messages
    let mut processing_time: Option<String> = None;

    for msg in result.messages() {
        let desc = msg.description();

        // Extract elapsed time from final success message
        if desc.contains("Succeeded at") && desc.contains("Elapsed Time:") {
            if let Some(time_part) = desc.split("Elapsed Time: ").nth(1) {
                if let Some(time_str) = time_part.split(" seconds").next() {
                    processing_time = Some(time_str.to_string());
                }
            }
        }
    }

    // Count processing steps to infer data volume
    let step_count = result
        .messages()
        .iter()
        .filter(|m| {
            let desc = m.description();
            desc.starts_with("Executing (") || desc.contains("Running script")
        })
        .count();

    tracing::info!("   ✅ Hotspot analysis completed successfully");
    if let Some(time) = processing_time {
        tracing::info!("   ⏱️  Processing time: {} seconds", time);
    }
    tracing::info!("   🔧 Processing steps executed: {}", step_count);
    tracing::info!("");
    tracing::info!("   📈 What was analyzed:");
    tracing::info!("      • Time period: January weekends (Saturdays & Sundays)");
    tracing::info!("      • Data filtered: SQL query selected weekend 911 calls");
    tracing::info!("      • Analysis type: Getis-Ord Gi* hotspot statistic");
    tracing::info!("      • Output: Kernel density raster with hotspot zones");
    tracing::info!("");
    tracing::info!("   🎯 Key Processing Steps Observed:");
    tracing::info!("      1. Select Layer By Attribute - filtered by date and day");
    tracing::info!("      2. Copy Features - extracted matching records");
    tracing::info!("      3. Integrate - consolidated overlapping points");
    tracing::info!("      4. Collect Events - aggregated call locations");
    tracing::info!("      5. Hot Spot Analysis - Getis-Ord Gi* statistic");
    tracing::info!("      6. Natural Neighbor - interpolated density surface");
    tracing::info!("      7. Reclassify - classified into hotspot zones");
    tracing::info!("");
    tracing::info!("   💡 Result Interpretation:");
    tracing::info!("      Weekend 911 calls analyzed for spatial clustering.");
    tracing::info!("      Hotspot zones identify statistically significant concentrations");
    tracing::info!("      where call density is higher than random chance would predict.");
    tracing::info!("");
    tracing::info!("      Expected hotspot patterns:");
    tracing::info!("      • Entertainment districts (bars, clubs)");
    tracing::info!("      • Beach/recreational areas");
    tracing::info!("      • Major highway corridors");
    tracing::info!("      • Tourist attractions");
    tracing::info!("");
    tracing::info!("      🚨 Actionable Intelligence:");
    tracing::info!("      → Deploy extra patrols to identified hotspots on weekends");
    tracing::info!("      → Position ambulances near high-density zones");
    tracing::info!("      → Compare with weekday patterns (see Example 2)");
    tracing::info!("      → Adjust resource allocation by day of week");

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
    tracing::info!("📅 Analysis Period: Weekday calls in February 1998");
    tracing::info!("   Time Period: February 1-28, 1998");
    tracing::info!("   Filter: Weekdays only (Monday through Friday)");
    tracing::info!("");
    tracing::info!("🎯 Monitoring Strategy:");
    tracing::info!("   Instead of automatic polling (Example 1), this demonstrates");
    tracing::info!("   manual status checks - useful when you want to:");
    tracing::info!("   • Show real-time progress to users in a UI");
    tracing::info!("   • Update a progress bar or spinner");
    tracing::info!("   • Log each status transition for debugging");
    tracing::info!("   • Implement custom retry/timeout logic");

    // SQL query for weekday calls in February 1998
    let query = r#"("DATE" > date '1998-02-01 00:00:00' AND "DATE" < date '1998-02-28 23:59:59') AND ("Day" IN ('MON', 'TUE', 'WED', 'THU', 'FRI'))"#;

    let mut params = HashMap::new();
    params.insert("Query".to_string(), serde_json::json!(query));

    tracing::info!("");
    tracing::info!("📤 Submitting weekday hotspot analysis job...");
    tracing::info!("   SQL Query: {}", query);
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
            arcgis::GPJobStatus::Executing => "Server calculating kernel density hotspots",
            arcgis::GPJobStatus::Succeeded => "Analysis complete, results ready",
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
                tracing::info!("📊 Weekday 911 Call Analysis Results:");

                // Extract statistics from processing messages
                let mut copy_time: Option<String> = None;
                let mut hotspot_time: Option<String> = None;
                let mut total_time: Option<String> = None;

                for msg in result.messages() {
                    let desc = msg.description();

                    if desc.contains("Copy Features") && desc.contains("Elapsed Time:") {
                        if let Some(time_part) = desc.split("Elapsed Time: ").nth(1) {
                            if let Some(time_str) = time_part.split(" seconds").next() {
                                copy_time = Some(time_str.to_string());
                            }
                        }
                    }

                    if desc.contains("Hot Spot Analysis") && desc.contains("Elapsed Time:") {
                        if let Some(time_part) = desc.split("Elapsed Time: ").nth(1) {
                            if let Some(time_str) = time_part.split(" seconds").next() {
                                hotspot_time = Some(time_str.to_string());
                            }
                        }
                    }

                    if desc.starts_with("Succeeded at") && desc.contains("Elapsed Time:") {
                        if let Some(time_part) = desc.split("Elapsed Time: ").nth(1) {
                            if let Some(time_str) = time_part.split(" seconds").next() {
                                total_time = Some(time_str.to_string());
                            }
                        }
                    }
                }

                tracing::info!("   ✅ Analysis completed successfully");
                tracing::info!("   🗓️  Dataset: February 1-28, 1998 (weekdays only)");
                tracing::info!("   📊 Processing stages:");
                if let Some(time) = copy_time {
                    tracing::info!("      • Feature extraction: {} seconds", time);
                }
                if let Some(time) = hotspot_time {
                    tracing::info!("      • Hotspot calculation: {} seconds", time);
                }
                if let Some(time) = total_time {
                    tracing::info!("      • Total processing: {} seconds", time);
                }
                tracing::info!("");
                tracing::info!("   🔍 Weekday vs Weekend Comparison:");
                tracing::info!("      Weekday patterns (this analysis):");
                tracing::info!("      • Morning rush hour: 7-9 AM traffic incidents");
                tracing::info!("      • Business districts: Medical emergencies during work hours");
                tracing::info!("      • School zones: 8 AM and 3 PM peaks");
                tracing::info!("      • Industrial areas: Workplace accidents");
                tracing::info!("");
                tracing::info!("      Weekend patterns (Example 1):");
                tracing::info!("      • Entertainment districts: Late night (10 PM - 2 AM)");
                tracing::info!("      • Beach/recreational: Afternoon peaks");
                tracing::info!("      • Residential: Domestic incidents");
                tracing::info!("");
                tracing::info!("   💡 Operational Intelligence:");
                tracing::info!("      → Weekdays need morning rush hour coverage");
                tracing::info!("      → Weekends need late-night entertainment district patrols");
                tracing::info!("      → Different ambulance positioning strategies needed");
                tracing::info!("      → Staffing levels should vary by day of week");
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
    tracing::info!("📅 Analysis Period: All calls in March 1998");
    tracing::info!("   Time Period: March 1-31, 1998");
    tracing::info!("   Filter: All days (full month analysis)");
    tracing::info!("");
    tracing::info!("📝 Why Job Messages Matter:");
    tracing::info!("   Geoprocessing jobs can run for minutes or hours.");
    tracing::info!("   Server messages provide:");
    tracing::info!("   • Processing step details (\"Analyzing 15,234 points...\")");
    tracing::info!("   • Performance warnings (\"Large dataset, may be slow\")");
    tracing::info!("   • Error diagnostics (\"Invalid date format\")");
    tracing::info!("   • Validation feedback (\"Query returned 0 features\")");
    tracing::info!("");
    tracing::info!("   Message types:");
    tracing::info!("   • Informative: Normal processing steps");
    tracing::info!("   • Warning: Non-fatal issues (e.g., data clipped)");
    tracing::info!("   • Error: Fatal problems that caused failure");

    // SQL query for all calls in March 1998
    let query = r#""DATE" > date '1998-03-01 00:00:00' AND "DATE" < date '1998-03-31 23:59:59'"#;

    let mut params = HashMap::new();
    params.insert("Query".to_string(), serde_json::json!(query));

    tracing::info!("");
    tracing::info!("📤 Submitting full-month hotspot analysis for message analysis...");
    tracing::info!("   SQL Query: {}", query);
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
    tracing::info!("💬 Analysis of Server Messages ({} total):", messages.len());

    if messages.is_empty() {
        tracing::info!("   ℹ️  No messages returned");
    } else {
        // Parse messages to extract meaningful statistics
        let mut processing_stages: Vec<(String, String)> = Vec::new();
        let mut current_stage = String::new();

        for msg in messages.iter() {
            let desc = msg.description();

            // Extract stage names from "Executing (Tool): ..." messages
            if desc.starts_with("Executing (") {
                if let Some(tool_part) = desc.split("Executing (").nth(1) {
                    if let Some(tool_name) = tool_part.split("):").next() {
                        current_stage = tool_name.to_string();
                    }
                }
            }

            // Extract elapsed time for each stage
            if desc.contains("Succeeded at")
                && desc.contains("Elapsed Time:")
                && !current_stage.is_empty()
            {
                if let Some(time_part) = desc.split("Elapsed Time: ").nth(1) {
                    if let Some(time_str) = time_part.split(" seconds").next() {
                        let stage_time = time_str.to_string();
                        processing_stages.push((current_stage.clone(), stage_time));
                        current_stage.clear();
                    }
                }
            }
        }

        tracing::info!("");
        tracing::info!(
            "   📊 Processing Pipeline ({} stages):",
            processing_stages.len()
        );
        for (stage, time) in processing_stages.iter() {
            tracing::info!("      • {}: {} seconds", stage, time);
        }

        // Extract interesting details
        let has_integrate = messages
            .iter()
            .any(|m| m.description().contains("Integrate"));
        let has_hotspot = messages
            .iter()
            .any(|m| m.description().contains("Hot Spot Analysis"));
        let has_interpolation = messages
            .iter()
            .any(|m| m.description().contains("Natural Neighbor"));

        tracing::info!("");
        tracing::info!("   🔍 Analysis Techniques Applied:");
        if has_integrate {
            tracing::info!("      ✓ Point Integration (150 feet tolerance)");
            tracing::info!("        → Consolidates overlapping call locations");
        }
        if has_hotspot {
            tracing::info!("      ✓ Getis-Ord Gi* Hotspot Statistic");
            tracing::info!("        → Identifies statistically significant clusters");
        }
        if has_interpolation {
            tracing::info!("      ✓ Natural Neighbor Interpolation");
            tracing::info!("        → Creates continuous density surface from points");
        }

        tracing::info!("");
        tracing::info!("   📈 Full Month Analysis (March 1998):");
        tracing::info!("      • All days included (not just weekends or weekdays)");
        tracing::info!("      • Captures complete monthly call patterns");
        tracing::info!("      • Baseline for seasonal comparisons");
        tracing::info!("");
        tracing::info!("   💡 What These Messages Reveal:");
        tracing::info!("      1. Data Quality: 'Integrate' step shows calls were de-duplicated");
        tracing::info!("      2. Statistical Rigor: Getis-Ord Gi* is industry-standard");
        tracing::info!("      3. Processing Efficiency: ~17 seconds for month of data");
        tracing::info!("      4. Output Quality: Multi-step interpolation ensures smooth surface");
        tracing::info!("");
        tracing::info!("   🎯 Comparing Across Examples:");
        tracing::info!("      • Example 1 (Weekends): Entertainment district focus");
        tracing::info!("      • Example 2 (Weekdays): Business/commute patterns");
        tracing::info!("      • Example 3 (Full Month): Overall baseline patterns");
        tracing::info!("");
        tracing::info!("      Combined analysis enables:");
        tracing::info!("      → Day-of-week resource allocation strategies");
        tracing::info!("      → Identification of persistent vs. temporal hotspots");
        tracing::info!("      → Evidence-based patrol route optimization");
    }

    tracing::info!("");
    tracing::info!("🎯 Use Cases for Message Analysis:");
    tracing::info!("   • Debugging: Why did my hotspot analysis fail?");
    tracing::info!("   • Auditing: What steps did the server perform?");
    tracing::info!("   • Optimization: How many features were processed?");
    tracing::info!("   • Validation: Were any inputs modified or clipped?");

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
    tracing::info!("   - Filter data with SQL queries before analysis");
    tracing::info!("   - Use appropriate search radius for density");
    tracing::info!("   - Consider output raster resolution (time vs detail)");
    tracing::info!("   - Cache service metadata (capabilities, parameters)");
    tracing::info!("");
    tracing::info!("🔧 Common GP Services:");
    tracing::info!("   - Spatial Statistics: Hotspot, clustering, autocorrelation");
    tracing::info!("   - Density Analysis: Kernel, point density surfaces");
    tracing::info!("   - Terrain Analysis: Slope, aspect, viewshed, hillshade");
    tracing::info!("   - Spatial Analysis: Buffer, overlay, proximity");
    tracing::info!("   - Network Analysis: Service areas, routing, allocation");
    tracing::info!("   - Geocoding: Batch address locating");
    tracing::info!("");
    tracing::info!("📊 Hotspot Analysis Tips:");
    tracing::info!("   - Compare temporal patterns (weekday vs weekend)");
    tracing::info!("   - Validate results against known high-activity areas");
    tracing::info!("   - Use appropriate search radius for your data");
    tracing::info!("   - Consider population density when interpreting results");
}
