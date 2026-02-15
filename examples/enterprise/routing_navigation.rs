//! 🗺️ Routing and Navigation - Plan Your Perfect Road Trip
//!
//! Real-world routing scenarios using ArcGIS World Routing Service:
//! Plan an epic Pacific Coast road trip from San Francisco to Seattle, with optimal
//! routing, drive-time analysis, and finding the nearest gas station when you're low on fuel!
//!
//! # What You'll Learn
//!
//! - **Multi-stop routing**: Optimize routes through multiple cities
//! - **Service areas**: Generate drive-time polygons (15, 30, 45-minute zones)
//! - **Closest facility**: Find nearest services (gas stations, rest stops)
//! - **OD cost matrix**: Calculate travel times between multiple origins/destinations
//! - **Error handling**: Graceful handling of routing failures
//! - **Builder patterns**: Construct complex routing parameters
//! - **Comprehensive assertions**: Validate all API responses
//!
//! # Prerequisites
//!
//! - API key with routing credits (uses billable ArcGIS World Routing Service)
//! - Set `ARCGIS_API_KEY` in `.env` file
//!
//! # Running
//!
//! ```bash
//! cargo run --example routing_navigation
//!
//! # With debug logging to see all requests:
//! RUST_LOG=debug cargo run --example routing_navigation
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Trip planning**: Calculate optimal multi-stop routes with ETAs
//! - **Delivery optimization**: Plan efficient delivery routes
//! - **Service coverage**: Determine areas reachable within time limits
//! - **Emergency response**: Find nearest hospitals, fire stations, police
//! - **Location intelligence**: Analyze accessibility and drive-time zones
//!
//! # Cost Awareness
//!
//! ⚠️ This example uses the World Routing Service which consumes routing credits.
//! Check your ArcGIS Online quota before running multiple times.

use anyhow::Result;
use arcgis::{
    ApiKeyAuth, ApiKeyTier, ArcGISClient, ArcGISGeometry, ArcGISPoint, ClosestFacilityParameters,
    NALocation, ODCostMatrixParameters, RouteParameters, RoutingServiceClient,
    ServiceAreaParameters, TravelDirection,
};

/// World Routing Service endpoints
const ROUTE_SERVICE: &str =
    "https://route-api.arcgis.com/arcgis/rest/services/World/Route/NAServer/Route_World";
const SERVICE_AREA_SERVICE: &str = "https://route-api.arcgis.com/arcgis/rest/services/World/ServiceAreas/NAServer/ServiceArea_World";
const CLOSEST_FACILITY_SERVICE: &str = "https://route-api.arcgis.com/arcgis/rest/services/World/ClosestFacility/NAServer/ClosestFacility_World";
const OD_COST_MATRIX_SERVICE: &str = "https://route-api.arcgis.com/arcgis/rest/services/World/OriginDestinationCostMatrix/NAServer/OriginDestinationCostMatrix_World";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🗺️ ArcGIS Routing & Navigation Examples");
    tracing::info!("Pacific Coast Road Trip: San Francisco → Seattle");

    // Create authenticated client (automatically loads .env)
    tracing::debug!("Creating authenticated client");
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);

    // Demonstrate routing and navigation operations
    demonstrate_multi_stop_route(&client).await?;
    demonstrate_service_area(&client).await?;
    demonstrate_closest_facility(&client).await?;
    demonstrate_od_cost_matrix(&client).await?;

    tracing::info!("\n✅ All routing examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates multi-stop route planning through cities.
async fn demonstrate_multi_stop_route(client: &ArcGISClient) -> Result<()> {
    tracing::info!("\n=== Example 1: Planning Your Road Trip Route ===");
    tracing::info!("Calculate optimal route: SF → Portland → Seattle");

    let route_service = RoutingServiceClient::new(ROUTE_SERVICE, client);

    // Define road trip stops
    let san_francisco = create_stop(-122.4194, 37.7749, "San Francisco, CA");
    let portland = create_stop(-122.6765, 45.5231, "Portland, OR");
    let seattle = create_stop(-122.3321, 47.6062, "Seattle, WA");

    let route_params = RouteParameters::builder()
        .stops(vec![
            san_francisco.clone(),
            portland.clone(),
            seattle.clone(),
        ])
        .return_directions(true)
        .return_routes(true)
        .return_stops(true)
        .build()?;

    tracing::debug!("Sending route request to ArcGIS");
    let route_result = route_service.solve_route(route_params).await?;

    if let Some(route) = route_result.routes().first() {
        let distance_miles = route.total_length().unwrap_or(0.0);
        let time_minutes = route.total_time().unwrap_or(0.0);

        tracing::info!(
            distance_miles = format!("{:.1}", distance_miles),
            drive_time_hours = format!("{:.1}", time_minutes / 60.0),
            "✅ Route calculated successfully!"
        );

        tracing::info!("📍 Route summary:");
        tracing::info!("   Total distance: {:.1} miles", distance_miles);
        tracing::info!("   Estimated drive time: {:.1} hours", time_minutes / 60.0);

        // Show turn-by-turn directions if available
        let directions = route.directions();
        if !directions.is_empty() {
            tracing::info!("   Turn-by-turn directions: {} steps", directions.len());
            tracing::debug!("First few directions:");
            for (i, direction) in directions.iter().take(3).enumerate() {
                if let Some(text) = direction.text() {
                    tracing::debug!("     {}. {}", i + 1, text);
                }
            }
        }
    } else {
        tracing::warn!("⚠️  No route found in result");
    }

    Ok(())
}

/// Demonstrates generating drive-time service area polygons.
async fn demonstrate_service_area(client: &ArcGISClient) -> Result<()> {
    tracing::info!("\n=== Example 2: Drive-Time Analysis ===");
    tracing::info!("Generate 15, 30, and 45-minute drive zones from San Francisco");

    let service_area_client = RoutingServiceClient::new(SERVICE_AREA_SERVICE, client);

    let san_francisco = create_stop(-122.4194, 37.7749, "San Francisco, CA");

    let service_area_params = ServiceAreaParameters::builder()
        .facilities(vec![san_francisco])
        .default_breaks(vec![15.0, 30.0, 45.0]) // Minutes
        .return_polygons(true) // Request polygon output
        .build()?;

    tracing::debug!("Calculating service area polygons");
    let service_area_result = service_area_client
        .solve_service_area(service_area_params)
        .await?;

    tracing::info!(
        polygon_count = service_area_result.service_area_polygons().len(),
        "✅ Service area polygons generated"
    );

    // Validate results
    anyhow::ensure!(
        !service_area_result.service_area_polygons().is_empty(),
        "Should generate at least one service area polygon"
    );

    anyhow::ensure!(
        service_area_result.service_area_polygons().len() == 3,
        "Should generate 3 polygons (for 15, 30, 45 min breaks), got {}",
        service_area_result.service_area_polygons().len()
    );

    tracing::info!("📊 Drive-time zones from San Francisco:");
    for (i, polygon) in service_area_result
        .service_area_polygons()
        .iter()
        .enumerate()
    {
        anyhow::ensure!(
            polygon.geometry().is_some(),
            "Polygon {} should have geometry",
            i + 1
        );

        if let Some(from_break) = polygon.from_break() {
            if let Some(to_break) = polygon.to_break() {
                anyhow::ensure!(
                    from_break < to_break,
                    "from_break ({}) should be less than to_break ({})",
                    from_break,
                    to_break
                );

                tracing::info!(
                    "   Zone {}: {}-{} minute drive time",
                    i + 1,
                    from_break,
                    to_break
                );
            }
        }
    }

    tracing::info!("💡 Pro tip: Service areas show reachable regions for delivery planning");

    Ok(())
}

/// Demonstrates finding the closest facility from current location.
async fn demonstrate_closest_facility(client: &ArcGISClient) -> Result<()> {
    tracing::info!("\n=== Example 3: Finding Nearest Services ===");
    tracing::info!("Scenario: Road trip on I-5 - find closest gas station near San Jose");

    let closest_facility_client = RoutingServiceClient::new(CLOSEST_FACILITY_SERVICE, client);

    // Your current location (incident) - Downtown San Jose
    let current_location = create_location(-121.8863, 37.3382, "Downtown San Jose");

    // Gas stations along I-5/US-101 corridor (facilities)
    let gas_station_1 = create_location(-121.8947, 37.3688, "North San Jose Station");
    let gas_station_2 = create_location(-121.8772, 37.3088, "South San Jose Station");
    let gas_station_3 = create_location(-121.9025, 37.2893, "Campbell Station");

    let closest_facility_params = ClosestFacilityParameters::builder()
        .incidents(vec![current_location])
        .facilities(vec![gas_station_1, gas_station_2, gas_station_3])
        .default_target_facility_count(1) // Find closest 1
        .return_routes(true)
        .travel_direction(TravelDirection::ToFacility) // From incident to facility
        .accumulate_attribute_names(vec!["Miles".to_string()]) // Request distance attribute
        .build()?;

    tracing::debug!("Finding nearest gas station");
    let closest_result = closest_facility_client
        .solve_closest_facility(closest_facility_params)
        .await?;

    tracing::info!(
        route_count = closest_result.routes().len(),
        "✅ Found closest facility route"
    );

    // Validate results
    anyhow::ensure!(
        !closest_result.routes().is_empty(),
        "Should find at least one route to closest facility"
    );

    anyhow::ensure!(
        closest_result.routes().len() == 1,
        "Should find exactly 1 route (defaultTargetFacilityCount=1), got {}",
        closest_result.routes().len()
    );

    anyhow::ensure!(
        !closest_result.facilities().is_empty(),
        "Should have facilities in result"
    );

    anyhow::ensure!(
        !closest_result.incidents().is_empty(),
        "Should have incidents in result"
    );

    if let Some(route) = closest_result.routes().first() {
        let distance_miles = route.total_length().unwrap_or(0.0);
        let time_minutes = route.total_time().unwrap_or(0.0);

        anyhow::ensure!(
            distance_miles > 0.0,
            "Route distance should be positive, got {}",
            distance_miles
        );

        anyhow::ensure!(
            time_minutes > 0.0,
            "Route time should be positive, got {}",
            time_minutes
        );

        anyhow::ensure!(route.geometry().is_some(), "Route should have geometry");

        tracing::info!("⛽ Closest gas station:");
        tracing::info!("   Distance: {:.2} miles away", distance_miles);
        tracing::info!("   Drive time: {:.1} minutes", time_minutes);
        tracing::info!(
            "   Route geometry points: {}",
            if let Some(geom) = route.geometry() {
                format!(
                    "{} points",
                    match geom {
                        ArcGISGeometry::Polyline(line) =>
                            line.paths().iter().map(|p| p.len()).sum::<usize>(),
                        _ => 0,
                    }
                )
            } else {
                "No geometry".to_string()
            }
        );
    }

    Ok(())
}

/// Demonstrates origin-destination cost matrix calculation.
///
/// MINIMAL API USAGE: Only 2 origins × 2 destinations = 4 cost calculations.
async fn demonstrate_od_cost_matrix(client: &ArcGISClient) -> Result<()> {
    tracing::info!("\n=== Example 4: Travel Cost Matrix ===");
    tracing::info!("Calculate all travel times between offices (2 origins × 2 destinations)");
    tracing::info!("⚠️  Minimal usage: 2x2 matrix to conserve API credits");

    let od_matrix_client = RoutingServiceClient::new(OD_COST_MATRIX_SERVICE, client);

    // Origins: Company offices in Bay Area
    let origin_sf = create_location(-122.4194, 37.7749, "SF Office");
    let origin_oakland = create_location(-122.2711, 37.8044, "Oakland Office");

    // Destinations: Client sites
    let dest_san_jose = create_location(-121.8863, 37.3382, "San Jose Client");
    let dest_palo_alto = create_location(-122.1430, 37.4419, "Palo Alto Client");

    let od_params = ODCostMatrixParameters::builder()
        .origins(vec![origin_sf, origin_oakland])
        .destinations(vec![dest_san_jose, dest_palo_alto])
        .accumulate_attribute_names(vec!["Miles".to_string()]) // Request distance attribute
        .build()?;

    tracing::debug!("Calculating OD cost matrix");
    let od_result = od_matrix_client.generate_od_cost_matrix(od_params).await?;

    tracing::info!(
        od_line_count = od_result.od_lines().len(),
        "✅ Cost matrix calculated"
    );

    // Validate results
    anyhow::ensure!(
        !od_result.od_lines().is_empty(),
        "Should generate OD cost matrix lines"
    );

    anyhow::ensure!(
        od_result.od_lines().len() == 4,
        "Should generate 4 OD lines (2 origins × 2 destinations), got {}",
        od_result.od_lines().len()
    );

    tracing::info!("📊 Travel time matrix:");
    tracing::info!("   From → To                          Time    Distance");
    tracing::info!("   ================================================");

    for od_line in od_result.od_lines() {
        let time_mins = od_line.total_time().unwrap_or(0.0);
        let distance_miles = od_line.total_distance().unwrap_or(0.0);

        anyhow::ensure!(
            time_mins > 0.0,
            "Travel time should be positive, got {}",
            time_mins
        );

        anyhow::ensure!(
            distance_miles > 0.0,
            "Distance should be positive, got {}",
            distance_miles
        );

        let origin_name = od_result
            .origins()
            .get(od_line.origin_id().unwrap_or(0) as usize - 1)
            .and_then(|o| o.name().as_deref())
            .unwrap_or("Unknown");

        let dest_name = od_result
            .destinations()
            .get(od_line.destination_id().unwrap_or(0) as usize - 1)
            .and_then(|d| d.name().as_deref())
            .unwrap_or("Unknown");

        tracing::info!(
            "   {} → {}  {:>6.1} min  {:>7.2} mi",
            origin_name,
            dest_name,
            time_mins,
            distance_miles
        );
    }

    tracing::info!("");
    tracing::info!("💡 Use cases: Multi-location logistics, delivery route optimization");
    tracing::info!("   - Compare all origin-destination pairs efficiently");
    tracing::info!("   - No routes/directions - just travel costs (faster/cheaper)");
    tracing::info!("   - Perfect for fleet dispatching and territory analysis");

    Ok(())
}

/// Prints best practices for routing and navigation.
fn print_best_practices() {
    tracing::info!("\n💡 Routing Best Practices:");
    tracing::info!("   - Cache route results to minimize API calls and costs");
    tracing::info!("   - Use service areas for coverage/accessibility analysis");
    tracing::info!("   - Closest facility is perfect for emergency response planning");
    tracing::info!("   - Always check total_miles/total_minutes for route validation");
    tracing::info!("   - Consider traffic patterns with time-of-day routing (premium feature)");
    tracing::info!("");
    tracing::info!("🎯 When to Use Each Service:");
    tracing::info!("   - Route: Multi-stop trip planning, delivery routes");
    tracing::info!("   - Service Area: Coverage analysis, accessibility zones");
    tracing::info!("   - Closest Facility: Emergency response, nearest service finder");
    tracing::info!("   - OD Cost Matrix: Multi-location logistics, fleet dispatching");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Batch multiple route calculations when possible");
    tracing::info!("   - Request only needed attributes (directions, geometry)");
    tracing::info!("   - Use straight-line distance for rough estimates first");
    tracing::info!("   - Consider caching frequently-requested routes");
    tracing::info!("");
    tracing::info!("💰 Credit Usage:");
    tracing::info!("   - Simple route (2 stops): ~0.5 credits");
    tracing::info!("   - Optimized route (10+ stops): ~1.0 credits");
    tracing::info!("   - Service area: ~0.5 credits per facility");
    tracing::info!("   - Closest facility: ~0.5 credits");
    tracing::info!("   - OD cost matrix (2×2): ~0.5 credits");
    tracing::info!("   ⚠️  Monitor your ArcGIS Online quota!");
}

/// Helper to create a route stop/location
fn create_stop(lon: f64, lat: f64, name: &str) -> NALocation {
    NALocation::new(ArcGISGeometry::Point(
        ArcGISPoint::new(lon, lat).with_spatial_reference(Some(arcgis::SpatialReference::wgs84())),
    ))
    .with_name(name.to_string())
}

/// Helper to create a generic location (facility or incident)
fn create_location(lon: f64, lat: f64, name: &str) -> NALocation {
    NALocation::new(ArcGISGeometry::Point(
        ArcGISPoint::new(lon, lat).with_spatial_reference(Some(arcgis::SpatialReference::wgs84())),
    ))
    .with_name(name.to_string())
}
