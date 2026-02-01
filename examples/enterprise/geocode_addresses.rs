//! 🗺️ Geocoding Services - Address and Coordinate Conversion
//!
//! Demonstrates common geocoding operations using the ArcGIS World Geocoding Service.
//! Learn how to convert addresses to coordinates and vice versa, with autocomplete
//! suggestions and quality filtering.
//!
//! # What You'll Learn
//!
//! - **Forward geocoding**: Convert addresses to geographic coordinates
//! - **Reverse geocoding**: Convert coordinates to addresses
//! - **Autocomplete suggestions**: Get address suggestions as user types
//! - **Batch processing**: Geocode multiple addresses efficiently
//! - **Quality filtering**: Filter results by match score
//! - **Best practices**: Rate limiting and score thresholds
//!
//! # Prerequisites
//!
//! - ArcGIS API key (required for geocoding services)
//! - Geocoding API credits (included with most ArcGIS subscriptions)
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
//! cargo run --example geocode_addresses
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example geocode_addresses
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Delivery apps**: Convert customer addresses to map coordinates
//! - **Real estate**: Display property locations from street addresses
//! - **Emergency services**: Locate incidents from reported addresses
//! - **Address validation**: Verify and standardize user-entered addresses
//! - **Search interfaces**: Provide autocomplete for location search

use anyhow::Result;
use arcgis::{
    ApiKeyAuth, ApiKeyTier, ArcGISClient, ArcGISPoint, GeocodeServiceClient,
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

    tracing::info!("🗺️  ArcGIS Geocoding Service Examples");
    tracing::info!("Demonstrating address and coordinate conversion");

    // Create geocoding service client (automatically loads .env)
    tracing::debug!("Creating geocoding service client");
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let geocoder = GeocodeServiceClient::new(WORLD_GEOCODE_SERVICE, &client);

    // Demonstrate geocoding operations
    demonstrate_forward_geocoding(&geocoder).await?;
    demonstrate_reverse_geocoding(&geocoder).await?;
    demonstrate_autocomplete(&geocoder).await?;
    demonstrate_batch_processing(&geocoder).await?;
    demonstrate_high_precision(&geocoder).await?;

    tracing::info!("\n✅ All geocoding examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates forward geocoding (address → coordinates).
async fn demonstrate_forward_geocoding(geocoder: &GeocodeServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Forward Geocoding ===");
    tracing::info!("Convert addresses to geographic coordinates");

    let addresses = vec![
        "380 New York St, Redlands, CA 92373",
        "1600 Pennsylvania Ave NW, Washington, DC",
        "Eiffel Tower, Paris, France",
    ];

    for address in &addresses {
        tracing::debug!(address = %address, "Geocoding address");

        let response = geocoder.find_address_candidates(*address).await?;

        if let Some(candidate) = response.candidates().first() {
            tracing::info!(
                address = %address,
                matched_address = %candidate.address(),
                x = *candidate.location().x(),
                y = *candidate.location().y(),
                score = candidate.score(),
                "✅ Geocoded successfully"
            );
        } else {
            tracing::warn!(address = %address, "No candidates found");
        }
    }

    Ok(())
}

/// Demonstrates reverse geocoding (coordinates → address).
async fn demonstrate_reverse_geocoding(geocoder: &GeocodeServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Reverse Geocoding ===");
    tracing::info!("Convert coordinates to addresses");

    let locations = vec![
        ("Redlands, CA", -117.1825, 34.0555),
        ("White House", -77.0365, 38.8977),
        ("Eiffel Tower", 2.2945, 48.8584),
    ];

    for (name, lon, lat) in &locations {
        let point = ArcGISPoint::new(*lon, *lat);

        tracing::debug!(
            name = %name,
            lon = lon,
            lat = lat,
            "Reverse geocoding location"
        );

        let response = geocoder.reverse_geocode(&point).await?;

        let long_label = response.address().long_label().as_deref().unwrap_or("N/A");
        let city = response.address().city().as_deref().unwrap_or("N/A");
        let country = response
            .address()
            .country_code()
            .as_deref()
            .unwrap_or("N/A");

        tracing::info!(
            name = %name,
            address = %long_label,
            city = %city,
            country = %country,
            "✅ Reverse geocoded successfully"
        );
    }

    Ok(())
}

/// Demonstrates autocomplete suggestions for user input.
async fn demonstrate_autocomplete(geocoder: &GeocodeServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Autocomplete Suggestions ===");
    tracing::info!("Get address suggestions as user types");

    let partial_texts = vec!["380 New Y", "White Hou", "Eiffel"];

    for text in &partial_texts {
        tracing::debug!(partial_text = %text, "Getting suggestions");

        let response = geocoder.suggest(*text).await?;

        tracing::info!(
            partial_text = %text,
            suggestion_count = response.suggestions().len(),
            "✅ Got suggestions"
        );

        // Show top 3 suggestions
        for (i, suggestion) in response.suggestions().iter().take(3).enumerate() {
            tracing::debug!(
                index = i + 1,
                suggestion = %suggestion.text(),
                is_collection = suggestion.is_collection(),
                "Suggestion"
            );
        }
    }

    Ok(())
}

/// Demonstrates batch processing of multiple addresses with rate limiting.
async fn demonstrate_batch_processing(geocoder: &GeocodeServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 4: Batch Processing ===");
    tracing::info!("Geocode multiple addresses efficiently with rate limiting");

    let tech_companies = [
        "1 Apple Park Way, Cupertino, CA",
        "1 Microsoft Way, Redmond, WA",
        "1600 Amphitheatre Parkway, Mountain View, CA",
    ];

    tracing::info!(
        address_count = tech_companies.len(),
        "Processing multiple addresses"
    );

    for (i, address) in tech_companies.iter().enumerate() {
        tracing::debug!(index = i + 1, address = %address, "Geocoding address");

        let response = geocoder.find_address_candidates(*address).await?;

        if let Some(candidate) = response.candidates().first() {
            tracing::info!(
                index = i + 1,
                address = %address,
                matched_address = %candidate.address(),
                x = *candidate.location().x(),
                y = *candidate.location().y(),
                score = candidate.score(),
                "✅ Successfully geocoded"
            );
        }

        // Be respectful to the API - small delay between requests
        if i < tech_companies.len() - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    }

    Ok(())
}

/// Demonstrates filtering results by match score for high-precision geocoding.
async fn demonstrate_high_precision(geocoder: &GeocodeServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 5: High-Precision Filtering ===");
    tracing::info!("Only accept high-quality matches");

    let test_address = "380 New York St, Redlands";
    let min_score = 90.0; // Only accept matches with 90%+ confidence

    tracing::debug!(
        address = %test_address,
        min_score = min_score,
        "Geocoding with quality filter"
    );

    let response = geocoder.find_address_candidates(test_address).await?;

    // Filter by score
    let high_quality_matches: Vec<_> = response
        .candidates()
        .iter()
        .filter(|c| *c.score() >= min_score)
        .collect();

    tracing::info!(
        total_candidates = response.candidates().len(),
        high_quality_count = high_quality_matches.len(),
        min_score = min_score,
        "✅ Filtered candidates"
    );

    // Show high-quality matches
    for candidate in high_quality_matches.iter().take(3) {
        tracing::info!(
            address = %candidate.address(),
            score = candidate.score(),
            x = *candidate.location().x(),
            y = *candidate.location().y(),
            "High-quality match"
        );
    }

    Ok(())
}

/// Prints best practices for geocoding operations.
fn print_best_practices() {
    tracing::info!("\n💡 Geocoding Best Practices:");
    tracing::info!(
        "   - Filter results by score for higher quality (90+ is excellent, 80+ is good)"
    );
    tracing::info!("   - Use suggest() for autocomplete in search interfaces");
    tracing::info!("   - Reverse geocoding is ideal for 'Where am I?' features");
    tracing::info!("   - Add delays between requests to respect API rate limits");
    tracing::info!("   - Cache geocoding results to reduce API calls");
    tracing::info!("   - Validate addresses before storage to ensure consistency");
    tracing::info!("");
    tracing::info!("📊 Understanding Match Scores:");
    tracing::info!("   - 100: Perfect match (exact address)");
    tracing::info!("   - 90-99: Excellent match (minor differences)");
    tracing::info!("   - 80-89: Good match (may need verification)");
    tracing::info!("   - 70-79: Fair match (likely needs user confirmation)");
    tracing::info!("   - Below 70: Poor match (recommend rejecting)");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Batch requests when possible (use geocode_addresses for multiple)");
    tracing::info!("   - Use suggest() before find_address_candidates() to improve accuracy");
    tracing::info!("   - Implement exponential backoff for rate limit errors");
    tracing::info!("   - Store coordinates to avoid repeated geocoding");
    tracing::info!("");
    tracing::info!("💰 Credit Conservation:");
    tracing::info!("   - Forward geocoding: ~0.004 credits per address");
    tracing::info!("   - Reverse geocoding: ~0.004 credits per coordinate");
    tracing::info!("   - Autocomplete: Free (no credits consumed)");
    tracing::info!("   - Cache results aggressively to minimize costs");
}
