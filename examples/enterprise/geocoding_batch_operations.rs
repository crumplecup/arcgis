//! 🌍 Batch Geocoding Operations - Efficient Bulk Address Processing
//!
//! Demonstrates advanced batch geocoding operations for processing multiple addresses
//! efficiently. Learn how to use batch APIs, advanced options, and optimize for large-scale
//! geocoding workflows.
//!
//! # What You'll Learn
//!
//! - **Batch geocoding**: Process multiple addresses in a single request
//! - **Batch candidates**: Get multiple match candidates for each address
//! - **Advanced options**: Use max_locations and location_type filters
//! - **Performance optimization**: Reduce API calls and improve throughput
//! - **Quality filtering**: Handle batch results with confidence scores
//!
//! # Prerequisites
//!
//! - ArcGIS API key (required for geocoding services)
//! - Geocoding API credits (batch operations consume more credits)
//!
//! ## Environment Variables
//!
//! Set these in your `.env` file:
//!
//! ```env
//! ARCGIS_API_KEY=your_api_key_here
//! ```
//!
//! Get your API key from: https://developers.arcgis.com/
//!
//! # Running
//!
//! ```bash
//! cargo run --example geocoding_batch_operations
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example geocoding_batch_operations
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Data migration**: Geocode large address databases
//! - **Import workflows**: Process CSV/Excel files with addresses
//! - **Address validation**: Batch validate customer addresses
//! - **Location intelligence**: Add coordinates to existing datasets
//! - **Real estate**: Geocode property listings in bulk
//! - **Logistics**: Convert delivery addresses to route waypoints

use anyhow::Result;
use arcgis::{
    ApiKeyAuth, ApiKeyTier, ArcGISClient, GeocodeAddress, GeocodeServiceClient, LocationType,
};

/// ArcGIS World Geocoding Service URL
const WORLD_GEOCODE_SERVICE: &str =
    "https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🌍 Batch Geocoding Operations Examples");
    tracing::info!("Demonstrating efficient bulk address processing");
    tracing::info!("");

    // Create geocoding service client (automatically loads .env)
    tracing::debug!("Creating geocoding service client");
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let geocoder = GeocodeServiceClient::new(WORLD_GEOCODE_SERVICE, &client);

    // Demonstrate batch geocoding operations
    demonstrate_batch_geocode(&geocoder).await?;
    demonstrate_advanced_options(&geocoder).await?;

    tracing::info!("\n✅ All batch geocoding examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates batch geocoding with geocode_addresses().
async fn demonstrate_batch_geocode(geocoder: &GeocodeServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Batch Geocoding ===");
    tracing::info!("Process multiple addresses in a single API request");
    tracing::info!("");

    // Prepare batch addresses
    let addresses = vec![
        GeocodeAddress::new("380 New York St, Redlands, CA 92373"),
        GeocodeAddress::new("1 Microsoft Way, Redmond, WA"),
        GeocodeAddress::new("1600 Amphitheatre Parkway, Mountain View, CA"),
        GeocodeAddress::new("1 Infinite Loop, Cupertino, CA"),
    ];

    tracing::info!(
        address_count = addresses.len(),
        "Geocoding {} addresses in batch",
        addresses.len()
    );

    let response = geocoder.geocode_addresses(addresses).await?;

    // Validate response
    anyhow::ensure!(
        !response.locations().is_empty(),
        "Batch geocode should return results. Got 0 locations."
    );

    anyhow::ensure!(
        response.locations().len() == 4,
        "Expected 4 geocoded locations, got {}",
        response.locations().len()
    );

    tracing::info!("✅ Successfully geocoded {} addresses", response.locations().len());
    tracing::info!("");

    // Display results
    for (idx, location) in response.locations().iter().enumerate() {
        tracing::info!(
            "   {}. {} → ({:.4}, {:.4}) [score: {:.1}]",
            idx + 1,
            location.address(),
            *location.location().x(),
            *location.location().y(),
            *location.score()
        );

        // Validate each result
        anyhow::ensure!(
            !location.address().is_empty(),
            "Location {} should have an address",
            idx
        );

        anyhow::ensure!(
            *location.score() >= 0.0 && *location.score() <= 100.0,
            "Score should be 0-100, got {}",
            location.score()
        );

        // Check for reasonable coordinates (within world bounds)
        anyhow::ensure!(
            *location.location().x() >= -180.0 && *location.location().x() <= 180.0,
            "Longitude should be -180 to 180, got {}",
            location.location().x()
        );

        anyhow::ensure!(
            *location.location().y() >= -90.0 && *location.location().y() <= 90.0,
            "Latitude should be -90 to 90, got {}",
            location.location().y()
        );
    }

    tracing::info!("");
    tracing::info!("💡 Batch geocoding benefits:");
    tracing::info!("   • Single API request for multiple addresses");
    tracing::info!("   • Reduced network overhead");
    tracing::info!("   • More efficient credit usage");
    tracing::info!("   • Ideal for processing CSV/Excel files");

    Ok(())
}

/// Demonstrates advanced options with find_address_candidates_with_options().
async fn demonstrate_advanced_options(geocoder: &GeocodeServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Advanced Geocoding Options ===");
    tracing::info!("Use max_locations and location_type filters for precise control");
    tracing::info!("");

    let test_address = "Main St";

    // Example 1: Limit results with max_locations
    tracing::info!("Testing max_locations parameter:");
    let response_limited = geocoder
        .find_address_candidates_with_options(test_address, Some(3), None)
        .await?;

    anyhow::ensure!(
        !response_limited.candidates().is_empty(),
        "Should find candidates for '{}'",
        test_address
    );

    anyhow::ensure!(
        response_limited.candidates().len() <= 3,
        "max_locations=3 should return ≤3 results, got {}",
        response_limited.candidates().len()
    );

    tracing::info!(
        "   ✅ Requested max 3 locations, got {} candidates",
        response_limited.candidates().len()
    );

    // Example 2: Use location_type filter for rooftop precision
    tracing::info!("");
    tracing::info!("Testing location_type parameter:");
    let precise_address = "380 New York St, Redlands, CA";

    let response_rooftop = geocoder
        .find_address_candidates_with_options(
            precise_address,
            Some(5),
            Some(LocationType::Rooftop),
        )
        .await?;

    anyhow::ensure!(
        !response_rooftop.candidates().is_empty(),
        "Should find rooftop candidates for precise address"
    );

    tracing::info!(
        "   ✅ Found {} rooftop-level candidates",
        response_rooftop.candidates().len()
    );

    // Show top candidates
    tracing::info!("");
    tracing::info!("   Top candidates for '{}':", precise_address);
    for (idx, candidate) in response_rooftop.candidates().iter().take(3).enumerate() {
        tracing::info!(
            "     {}. {} [score: {:.1}]",
            idx + 1,
            candidate.address(),
            *candidate.score()
        );

        // Validate candidate data
        anyhow::ensure!(
            !candidate.address().is_empty(),
            "Candidate {} should have an address",
            idx
        );

        anyhow::ensure!(
            *candidate.score() > 0.0,
            "Candidate {} should have positive score",
            idx
        );
    }

    tracing::info!("");
    tracing::info!("💡 Advanced options:");
    tracing::info!("   • max_locations: Control result count (default: varies)");
    tracing::info!("   • location_type:");
    tracing::info!("     - Rooftop: Precise building-level coordinates");
    tracing::info!("     - Street: Street centerline coordinates");
    tracing::info!("   • Combine both for fine-grained control");

    Ok(())
}

/// Prints best practices for batch geocoding.
fn print_best_practices() {
    tracing::info!("\n💡 Batch Geocoding Best Practices:");
    tracing::info!("   - Use geocode_addresses() for bulk geocoding");
    tracing::info!("   - Use find_address_candidates() in a loop for multiple match options");
    tracing::info!("   - Batch operations are more efficient than individual requests");
    tracing::info!("   - Process in chunks of 100-1000 addresses per request");
    tracing::info!("   - Always validate scores before accepting results");
    tracing::info!("");
    tracing::info!("📊 Credit Usage:");
    tracing::info!("   - geocode_addresses: ~0.004 credits per address");
    tracing::info!("   - find_address_candidates: ~0.004 credits per address");
    tracing::info!("   - Batch operations have no additional overhead");
    tracing::info!("   - Cache results to avoid re-geocoding");
    tracing::info!("");
    tracing::info!("⚡ Performance Optimization:");
    tracing::info!("   - Batch size: 100-1000 addresses optimal");
    tracing::info!("   - Parallel batches: Run multiple batches concurrently");
    tracing::info!("   - Pre-filter: Remove duplicates before geocoding");
    tracing::info!("   - Retry strategy: Implement exponential backoff for failures");
    tracing::info!("");
    tracing::info!("🎯 Quality Control:");
    tracing::info!("   - Accept scores ≥90 automatically");
    tracing::info!("   - Flag scores 70-89 for manual review");
    tracing::info!("   - Reject scores <70");
    tracing::info!("   - Use find_address_candidates for ambiguous addresses");
    tracing::info!("");
    tracing::info!("⚙️  Error Handling:");
    tracing::info!("   - Check each result individually (some may fail)");
    tracing::info!("   - Log failed addresses for manual processing");
    tracing::info!("   - Implement retry logic for network failures");
    tracing::info!("   - Monitor rate limits and implement backoff");
}
