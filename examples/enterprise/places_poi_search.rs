//! 📍 Places POI Search - Find Nearby Points of Interest
//!
//! Demonstrates POI (point of interest) discovery using the ArcGIS Places Service.
//! Shows how to search for businesses, landmarks, and amenities near a location,
//! filter by category, and retrieve detailed information.
//!
//! # What You'll Learn
//!
//! - **Place search**: Find POI within radius of a point
//! - **Category filtering**: Search specific types (restaurants, gas stations, etc.)
//! - **Distance sorting**: Results ordered by proximity
//! - **Place details**: Get hours, ratings, reviews, contact info
//! - **Category discovery**: List all available POI categories
//! - **Pagination**: Handle large result sets
//!
//! # Prerequisites
//!
//! - API key with location services privileges (Tier 2+)
//! - Places API credits (consumed per search/details request)
//!
//! ## Environment Variables
//!
//! Set in your `.env` file:
//!
//! ```env
//! ARCGIS_LOCATION_KEY=your_api_key_with_location_privileges
//! ```
//!
//! Or use the legacy key:
//!
//! ```env
//! ARCGIS_API_KEY=your_api_key
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example places_poi_search
//!
//! # With debug logging to see all API calls:
//! RUST_LOG=debug cargo run --example places_poi_search
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Mobile apps**: "Find restaurants near me"
//! - **Trip planning**: Locate gas stations along route
//! - **Emergency services**: Find nearest hospitals, police stations
//! - **Real estate**: Analyze nearby amenities for property listings
//! - **Retail**: Competitor analysis (find similar businesses)
//! - **Navigation**: POI search for routing waypoints
//!
//! # Places API
//!
//! The Places Service provides access to millions of POI worldwide:
//! - **Businesses**: Restaurants, shops, services
//! - **Amenities**: Parks, parking, restrooms
//! - **Transportation**: Gas stations, airports, transit
//! - **Landmarks**: Museums, monuments, attractions
//! - **Services**: Banks, hospitals, schools
//!
//! # Credit Usage
//!
//! ⚠️ Places API operations consume credits:
//! - **Search**: ~0.016 credits per request
//! - **Details**: ~0.004 credits per request
//! - **Categories**: Free (no credits)
//!
//! Monitor your ArcGIS Online quota!

use anyhow::Result;
use arcgis::{ApiKeyAuth, ApiKeyTier, ArcGISClient, PlaceSearchParametersBuilder, PlacesClient};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("📍 Places POI Search Examples");
    tracing::info!("Find nearby businesses, landmarks, and amenities");
    tracing::info!("");

    // Create authenticated client (automatically loads .env)
    tracing::debug!("Creating authenticated client");
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let places = PlacesClient::new(&client);

    tracing::info!("✅ Authenticated with API key (ARCGIS_LOCATION_KEY)");
    tracing::info!("");

    // Demonstrate Places API operations
    demonstrate_category_discovery(&places).await?;
    demonstrate_nearby_search(&places).await?;
    demonstrate_category_filtering(&places).await?;
    demonstrate_place_details(&places).await?;
    demonstrate_pagination(&places).await?;

    tracing::info!("\n✅ All Places API examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates discovering available POI categories.
async fn demonstrate_category_discovery(places: &PlacesClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Category Discovery ===");
    tracing::info!("List all available POI categories for filtering");
    tracing::info!("");

    let categories = places.get_categories().await?;

    tracing::info!("✅ Retrieved POI categories");
    tracing::info!("   Total categories: {}", categories.categories().len());
    tracing::info!("");

    // Show top-level categories (no parent)
    let top_level: Vec<_> = categories
        .categories()
        .iter()
        .filter(|c| c.parent_category_id().is_none())
        .collect();

    tracing::info!("   Top-level categories: {}", top_level.len());
    for (i, category) in top_level.iter().take(10).enumerate() {
        tracing::info!(
            "     {}. {} (id: {})",
            i + 1,
            category.name(),
            category.category_id()
        );
    }

    tracing::info!("");
    tracing::info!("💡 Categories are hierarchical:");
    tracing::info!("   • Top-level: Food and Drink, Shopping, Services");
    tracing::info!("   • Subcategories: Restaurant → Fast Food → Burger Joint");
    tracing::info!("   • Use category_id for filtering searches");

    Ok(())
}

/// Demonstrates basic nearby place search.
async fn demonstrate_nearby_search(places: &PlacesClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Nearby Place Search ===");
    tracing::info!("Find all POI within 500m of a location");
    tracing::info!("");

    // Downtown Los Angeles coordinates
    let la_downtown = (-118.2437, 34.0522);

    tracing::info!("   Search location: Downtown LA");
    tracing::info!("     Longitude: {}", la_downtown.0);
    tracing::info!("     Latitude: {}", la_downtown.1);
    tracing::info!("     Radius: 500 meters");
    tracing::info!("");

    let params = PlaceSearchParametersBuilder::default()
        .x(la_downtown.0)
        .y(la_downtown.1)
        .radius(500.0)
        .page_size(10u32)
        .build()?;

    let results = places.find_places_near_point(params).await?;

    tracing::info!("✅ Found {} places", results.results().len());
    tracing::info!("");

    // Show nearest places
    for (i, place) in results.results().iter().take(5).enumerate() {
        let distance = place.distance().unwrap_or(0.0);
        let categories = place
            .categories()
            .iter()
            .map(|c| c.label().as_str())
            .collect::<Vec<_>>()
            .join(", ");

        tracing::info!("   {}. {} ({:.0}m)", i + 1, place.name(), distance);
        if !categories.is_empty() {
            tracing::info!("      Categories: {}", categories);
        }
        if let Some(addr) = place.address() {
            if let Some(street) = addr.street_address() {
                tracing::info!("      Address: {}", street);
            }
        }
    }

    tracing::info!("");
    tracing::info!("💡 Results are automatically sorted by distance");
    tracing::info!("   • Nearest places appear first");
    tracing::info!("   • Distance field shows meters from search point");

    Ok(())
}

/// Demonstrates category-filtered search.
async fn demonstrate_category_filtering(places: &PlacesClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Category Filtering ===");
    tracing::info!("Find specific types of POI using category IDs");
    tracing::info!("");

    // Downtown LA coordinates
    let la_downtown = (-118.2437, 34.0522);

    // Common category IDs (from Places API documentation)
    let scenarios = vec![
        ("Restaurants", "13065", "food"),
        ("Coffee Shops", "13032", "coffee"),
        ("Gas Stations", "17119", "gas"),
    ];

    for (name, category_id, search_text) in scenarios {
        tracing::info!("   Searching for: {}", name);

        let params = PlaceSearchParametersBuilder::default()
            .x(la_downtown.0)
            .y(la_downtown.1)
            .radius(1000.0)
            .category_ids(category_id.to_string())
            .search_text(search_text.to_string())
            .page_size(5u32)
            .build()?;

        let results = places.find_places_near_point(params).await?;

        tracing::info!("     Found: {} places", results.results().len());

        if let Some(place) = results.results().first() {
            let distance = place.distance().unwrap_or(0.0);
            tracing::info!("     Nearest: {} ({:.0}m)", place.name(), distance);
        }

        tracing::info!("");
    }

    tracing::info!("💡 Combine filters for precise results:");
    tracing::info!("   • category_ids: Restrict to POI types");
    tracing::info!("   • search_text: Keyword matching (\"pizza\", \"urgent care\")");
    tracing::info!("   • radius: Limit search area");

    Ok(())
}

/// Demonstrates retrieving detailed place information.
async fn demonstrate_place_details(places: &PlacesClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 4: Place Details ===");
    tracing::info!("Get extended information about a specific place");
    tracing::info!("");

    // First, search to get a place ID
    let la_downtown = (-118.2437, 34.0522);

    let params = PlaceSearchParametersBuilder::default()
        .x(la_downtown.0)
        .y(la_downtown.1)
        .radius(500.0)
        .search_text("restaurant".to_string())
        .page_size(1u32)
        .build()?;

    let search_results = places.find_places_near_point(params).await?;

    if let Some(place) = search_results.results().first() {
        tracing::info!("   Fetching details for: {}", place.name());
        tracing::info!("     Place ID: {}", place.place_id());
        tracing::info!("");

        // Get detailed information
        let details = places.get_place_details(place.place_id()).await?;

        tracing::info!("✅ Retrieved place details");
        tracing::info!("");

        // Show operating hours
        if let Some(hours) = details.hours() {
            tracing::info!("   Operating Hours:");
            for day_hours in hours.opening_hours().iter().take(7) {
                if *day_hours.is_closed() {
                    tracing::info!("     {}: Closed", day_hours.day());
                } else {
                    let open = day_hours.open().as_deref().unwrap_or("?");
                    let close = day_hours.close().as_deref().unwrap_or("?");
                    tracing::info!("     {}: {} - {}", day_hours.day(), open, close);
                }
            }
            tracing::info!("");
        }

        // Show rating
        if let Some(rating) = details.rating() {
            if let Some(user_rating) = rating.user() {
                let stars = "★".repeat(user_rating.round() as usize);
                tracing::info!("   Rating: {:.1}/5.0 {}", user_rating, stars);
            }
            if let Some(price) = rating.price() {
                let dollar_signs = "$".repeat(*price as usize);
                tracing::info!("   Price Level: {}", dollar_signs);
            }
            tracing::info!("");
        }

        // Show contact information
        if let Some(contact) = details.place().contact_info() {
            tracing::info!("   Contact Information:");
            if let Some(phone) = contact.telephone() {
                tracing::info!("     Phone: {}", phone);
            }
            if let Some(website) = contact.website() {
                tracing::info!("     Website: {}", website);
            }
            if let Some(email) = contact.email() {
                tracing::info!("     Email: {}", email);
            }
            tracing::info!("");
        }

        // Show description
        if let Some(desc) = details.description() {
            tracing::info!("   Description:");
            tracing::info!("     {}", desc);
            tracing::info!("");
        }
    } else {
        tracing::warn!("   No places found for details demo");
    }

    tracing::info!("💡 Place details include:");
    tracing::info!("   • Operating hours (open/close times by day)");
    tracing::info!("   • Ratings (user reviews, price level)");
    tracing::info!("   • Contact (phone, website, email)");
    tracing::info!("   • Social media links (if available)");
    tracing::info!("   • Descriptions and photos");

    Ok(())
}

/// Demonstrates pagination for large result sets.
async fn demonstrate_pagination(places: &PlacesClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 5: Pagination ===");
    tracing::info!("Handle large result sets with pagination tokens");
    tracing::info!("");

    // Downtown LA with larger radius to get more results
    let la_downtown = (-118.2437, 34.0522);

    let params = PlaceSearchParametersBuilder::default()
        .x(la_downtown.0)
        .y(la_downtown.1)
        .radius(2000.0)
        .search_text("restaurant".to_string())
        .page_size(5u32) // Small page size to demonstrate pagination
        .build()?;

    let first_page = places.find_places_near_point(params.clone()).await?;

    tracing::info!("   First page: {} results", first_page.results().len());

    if let Some(next_token) = first_page.next_page_token() {
        tracing::info!(
            "   Next page token: {}...",
            &next_token[..20.min(next_token.len())]
        );
        tracing::info!("");

        // Fetch next page
        let params_page_2 = PlaceSearchParametersBuilder::default()
            .x(la_downtown.0)
            .y(la_downtown.1)
            .radius(2000.0)
            .search_text("restaurant".to_string())
            .page_size(5u32)
            .page_token(next_token.clone())
            .build()?;

        let second_page = places.find_places_near_point(params_page_2).await?;

        tracing::info!("   Second page: {} results", second_page.results().len());

        if let Some(place) = second_page.results().first() {
            let distance = place.distance().unwrap_or(0.0);
            tracing::info!("     Sample: {} ({:.0}m)", place.name(), distance);
        }

        tracing::info!("");
        tracing::info!("💡 Pagination pattern:");
        tracing::info!("   1. Make initial search with page_size");
        tracing::info!("   2. Check for next_page_token in results");
        tracing::info!("   3. Use token in next request for more results");
        tracing::info!("   4. Repeat until no next_page_token");
    } else {
        tracing::info!("   No next page (all results fit in first page)");
        tracing::info!("");
        tracing::info!("💡 No pagination token means:");
        tracing::info!("   • All matching places returned in current page");
        tracing::info!("   • Try larger radius or different search terms");
    }

    Ok(())
}

/// Prints best practices for Places API usage.
fn print_best_practices() {
    tracing::info!("\n💡 Places API Best Practices:");
    tracing::info!("   - Cache search results to minimize API calls");
    tracing::info!("   - Use appropriate radius (500m for walking, 5000m for driving)");
    tracing::info!("   - Filter by category to reduce irrelevant results");
    tracing::info!("   - Combine search_text with category_ids for precision");
    tracing::info!("   - Request place details only when needed (costs credits)");
    tracing::info!("   - Monitor credit usage (check ArcGIS Online dashboard)");
    tracing::info!("");
    tracing::info!("🎯 Search Optimization:");
    tracing::info!("   - Start narrow, expand radius if too few results");
    tracing::info!("   - Use specific search terms (\"italian restaurant\" vs \"food\")");
    tracing::info!("   - Limit page_size to what you'll display (5-20)");
    tracing::info!("   - Don't fetch all pages unless necessary");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Batch multiple searches if possible");
    tracing::info!("   - Cache category list (changes infrequently)");
    tracing::info!("   - Store place IDs for later details lookup");
    tracing::info!("   - Use pagination to load results progressively");
    tracing::info!("");
    tracing::info!("💰 Credit Conservation:");
    tracing::info!("   - Search: ~0.016 credits per request");
    tracing::info!("   - Details: ~0.004 credits per request");
    tracing::info!("   - Categories: Free (no credits)");
    tracing::info!("   - Cache aggressively for frequently accessed data");
    tracing::info!("   - Consider daily/hourly result limits");
    tracing::info!("");
    tracing::info!("📱 Mobile App Patterns:");
    tracing::info!("   - \"Near me\": Use device GPS + small radius");
    tracing::info!("   - \"Along route\": Multiple searches at route points");
    tracing::info!("   - \"In area\": Larger radius + category filter");
    tracing::info!("   - \"Details\": Fetch on tap/selection only");
    tracing::info!("");
    tracing::info!("🔍 Common Categories:");
    tracing::info!("   - Restaurants: 13065");
    tracing::info!("   - Gas Stations: 17119");
    tracing::info!("   - Hotels: 13003");
    tracing::info!("   - Hospitals: 15000");
    tracing::info!("   - Parks: 16000");
    tracing::info!("   - Use get_categories() to discover all available");
}
