//! Routing Service client for network analysis operations.

use crate::{ArcGISClient, Result};
use tracing::instrument;

use super::{RouteParameters, RouteResult};

/// Client for interacting with an ArcGIS Routing Service (Network Analyst Server).
///
/// Provides operations for routing, service areas, closest facility, and origin-destination matrices.
///
/// # Example
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient, RoutingServiceClient};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
///
/// let routing_service = RoutingServiceClient::new(
///     "https://route.arcgis.com/arcgis/rest/services/World/Route/NAServer/Route_World",
///     &client,
/// );
/// # Ok(())
/// # }
/// ```
pub struct RoutingServiceClient<'a> {
    /// Base URL of the routing service.
    base_url: String,
    /// Reference to the ArcGIS client for HTTP operations.
    client: &'a ArcGISClient,
}

impl<'a> RoutingServiceClient<'a> {
    /// Creates a new Routing Service client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the routing service (e.g., `https://route.arcgis.com/.../Route_World`)
    /// * `client` - Reference to an authenticated ArcGIS client
    ///
    /// # Example
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, RoutingServiceClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let routing_service = RoutingServiceClient::new(
    ///     "https://route.arcgis.com/arcgis/rest/services/World/Route/NAServer/Route_World",
    ///     &client
    /// );
    /// ```
    pub fn new(base_url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        let base_url = base_url.into();
        tracing::debug!(base_url = %base_url, "Creating RoutingServiceClient");
        Self { base_url, client }
    }

    /// Solves a route between multiple stops.
    ///
    /// Calculates the optimal route connecting all stops, with options for
    /// turn-by-turn directions, barriers, and traffic-aware routing.
    ///
    /// # Arguments
    ///
    /// * `params` - Route parameters including stops and routing options
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, RoutingServiceClient, ArcGISPoint, ArcGISGeometry};
    /// use arcgis::{RouteParameters, NALocation};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let routing_service = RoutingServiceClient::new(
    ///     "https://route.arcgis.com/arcgis/rest/services/World/Route/NAServer/Route_World",
    ///     &client
    /// );
    ///
    /// let stop1 = NALocation::new(ArcGISGeometry::Point(ArcGISPoint {
    ///     x: -122.4194,
    ///     y: 37.7749,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// })).with_name("San Francisco");
    ///
    /// let stop2 = NALocation::new(ArcGISGeometry::Point(ArcGISPoint {
    ///     x: -118.2437,
    ///     y: 34.0522,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// })).with_name("Los Angeles");
    ///
    /// let params = RouteParameters::builder()
    ///     .stops(vec![stop1, stop2])
    ///     .return_directions(true)
    ///     .return_routes(true)
    ///     .return_stops(true)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = routing_service.solve_route(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(stop_count = params.stops().len()))]
    pub async fn solve_route(&self, params: RouteParameters) -> Result<RouteResult> {
        tracing::debug!("Solving route");

        let url = format!("{}/solve", self.base_url);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending route solve request");

        // Serialize stops as features
        let stops_json = serde_json::to_string(&serde_json::json!({
            "features": params.stops().iter().map(|stop| {
                serde_json::json!({
                    "geometry": stop.geometry,
                    "attributes": {
                        "Name": stop.name.as_ref().unwrap_or(&String::new()),
                    }
                })
            }).collect::<Vec<_>>()
        }))?;

        // Prepare string values that need to outlive the form vector
        let out_sr_str = params.out_sr().map(|sr| sr.to_string());

        let mut form = vec![
            ("stops", stops_json.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

        // Add optional parameters
        if let Some(return_directions) = params.return_directions() {
            form.push(("returnDirections", if *return_directions { "true" } else { "false" }));
        }
        if let Some(return_routes) = params.return_routes() {
            form.push(("returnRoutes", if *return_routes { "true" } else { "false" }));
        }
        if let Some(return_stops) = params.return_stops() {
            form.push(("returnStops", if *return_stops { "true" } else { "false" }));
        }
        if let Some(ref out_sr) = out_sr_str {
            form.push(("outSR", out_sr.as_str()));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "solve route request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: RouteResult = response.json().await?;

        tracing::info!(
            route_count = result.routes().len(),
            "solve route completed"
        );

        Ok(result)
    }
}
