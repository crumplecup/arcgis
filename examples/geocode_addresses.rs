//! Comprehensive Geocoding Service example.
//!
//! This example demonstrates common geocoding operations:
//! - Forward geocoding (address → coordinates)
//! - Reverse geocoding (coordinates → address)
//! - Autocomplete suggestions
//! - Batch geocoding (multiple addresses)
//!
//! # Prerequisites
//!
//! - ArcGIS API key (required for geocoding services)
//! - Geocoding API credits (included with most ArcGIS subscriptions)
//!
//! # Environment Variables
//!
//! Create a `.env` file with:
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
//! ```

use arcgis::{ApiKeyAuth, ArcGISClient, ArcGISPoint, GeocodeServiceClient};

/// ArcGIS World Geocoding Service URL
const WORLD_GEOCODE_SERVICE: &str =
    "https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🗺️  Geocoding Service Examples");

    // Load API key from environment (.env file automatically loaded)
    let auth = ApiKeyAuth::from_env()?;
    let client = ArcGISClient::new(auth);
    let geocoder = GeocodeServiceClient::new(WORLD_GEOCODE_SERVICE, &client);

    // Example 1: Forward Geocoding (address to coordinates)
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
                x = candidate.location().x,
                y = candidate.location().y,
                score = candidate.score(),
                "Geocoded successfully"
            );
        } else {
            tracing::warn!(address = %address, "No candidates found");
        }
    }

    // Example 2: Reverse Geocoding (coordinates to address)
    tracing::info!("\n=== Example 2: Reverse Geocoding ===");
    tracing::info!("Convert coordinates to addresses");

    let locations = vec![
        ("Redlands, CA", -117.1825, 34.0555),
        ("White House", -77.0365, 38.8977),
        ("Eiffel Tower", 2.2945, 48.8584),
    ];

    for (name, lon, lat) in &locations {
        let point = ArcGISPoint {
            x: *lon,
            y: *lat,
            z: None,
            m: None,
            spatial_reference: None,
        };

        tracing::debug!(
            name = %name,
            lon = lon,
            lat = lat,
            "Reverse geocoding location"
        );

        let response = geocoder.reverse_geocode(&point).await?;

        let long_label = response.address().long_label().as_deref().unwrap_or("N/A");
        let city = response.address().city().as_deref().unwrap_or("N/A");
        let country = response.address().country_code().as_deref().unwrap_or("N/A");

        tracing::info!(
            name = %name,
            address = %long_label,
            city = %city,
            country = %country,
            "Reverse geocoded successfully"
        );
    }

    // Example 3: Autocomplete Suggestions
    tracing::info!("\n=== Example 3: Autocomplete Suggestions ===");
    tracing::info!("Get address suggestions as user types");

    let partial_texts = vec!["380 New Y", "White Hou", "Eiffel"];

    for text in &partial_texts {
        tracing::debug!(partial_text = %text, "Getting suggestions");

        let response = geocoder.suggest(*text).await?;

        tracing::info!(
            partial_text = %text,
            suggestion_count = response.suggestions().len(),
            "Got suggestions"
        );

        for (i, suggestion) in response.suggestions().iter().take(3).enumerate() {
            tracing::debug!(
                index = i,
                suggestion = %suggestion.text(),
                is_collection = suggestion.is_collection(),
                "Suggestion"
            );
        }
    }

    // Example 4: Processing Multiple Addresses
    tracing::info!("\n=== Example 4: Processing Multiple Addresses ===");
    tracing::info!("Geocode multiple addresses efficiently");

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
        tracing::debug!(index = i, address = %address, "Geocoding address");

        let response = geocoder.find_address_candidates(*address).await?;

        if let Some(candidate) = response.candidates().first() {
            tracing::info!(
                index = i,
                address = %address,
                matched_address = %candidate.address(),
                x = candidate.location().x,
                y = candidate.location().y,
                score = candidate.score(),
                "Successfully geocoded"
            );
        }

        // Be respectful to the API - small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    // Example 5: High-precision geocoding
    tracing::info!("\n=== Example 5: Filtering by Match Score ===");
    tracing::info!("Only accept high-quality matches");

    let test_address = "380 New York St, Redlands";
    let min_score = 90.0; // Only accept matches with 90%+ confidence

    let response = geocoder
        .find_address_candidates(test_address)
        .await?;

    let high_quality_matches: Vec<_> = response
        .candidates()
        .iter()
        .filter(|c| *c.score() >= min_score)
        .collect();

    tracing::info!(
        total_candidates = response.candidates().len(),
        high_quality_count = high_quality_matches.len(),
        min_score = min_score,
        "Filtered candidates"
    );

    for candidate in high_quality_matches.iter().take(3) {
        tracing::info!(
            address = %candidate.address(),
            score = candidate.score(),
            x = candidate.location().x,
            y = candidate.location().y,
            "High-quality match"
        );
    }

    tracing::info!("\n✅ All geocoding examples completed successfully!");
    tracing::info!("💡 Tips:");
    tracing::info!("   - Filter results by score for higher quality matches (90+ is excellent)");
    tracing::info!("   - Use suggest() for autocomplete in user interfaces");
    tracing::info!("   - Reverse geocoding is great for 'Where am I?' features");
    tracing::info!("   - Add delays between requests to respect API rate limits");

    Ok(())
}
