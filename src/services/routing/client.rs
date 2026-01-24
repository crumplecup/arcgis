//! Routing Service client for network analysis operations.

use crate::{ArcGISClient, Result};
use tracing::instrument;

use super::{
    ClosestFacilityParameters, ClosestFacilityResult, ODCostMatrixParameters, ODCostMatrixResult,
    RouteParameters, RouteResult, ServiceAreaParameters, ServiceAreaResult,
};

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

        tracing::debug!(url = %url, "Sending route solve request");

        // Serialize stops as features
        let stops_json = serde_json::to_string(&serde_json::json!({
            "features": params.stops().iter().map(|stop| {
                serde_json::json!({
                    "geometry": stop.geometry(),
                    "attributes": {
                        "Name": stop.name().as_ref().unwrap_or(&String::new()),
                    }
                })
            }).collect::<Vec<_>>()
        }))?;

        // Prepare string values that need to outlive the form vector
        let out_sr_str = params.out_sr().map(|sr| sr.to_string());

        let mut form = vec![("stops", stops_json.as_str()), ("f", "json")];

        // Add optional parameters
        if let Some(return_directions) = params.return_directions() {
            form.push((
                "returnDirections",
                if *return_directions { "true" } else { "false" },
            ));
        }
        if let Some(return_routes) = params.return_routes() {
            form.push((
                "returnRoutes",
                if *return_routes { "true" } else { "false" },
            ));
        }
        if let Some(return_stops) = params.return_stops() {
            form.push(("returnStops", if *return_stops { "true" } else { "false" }));
        }
        if let Some(ref out_sr) = out_sr_str {
            form.push(("outSR", out_sr.as_str()));
        }

        // Add token if required by auth provider
        let token_opt = self.client.get_token_if_required().await?;
        let token_str;
        if let Some(token) = token_opt {
            token_str = token;
            form.push(("token", token_str.as_str()));
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

        tracing::info!(route_count = result.routes().len(), "solve route completed");

        Ok(result)
    }

    /// Calculates service areas (drive-time or distance polygons).
    ///
    /// Generates polygons showing areas reachable from facilities within
    /// specified break values (time or distance).
    ///
    /// # Arguments
    ///
    /// * `params` - Service area parameters including facilities and break values
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, RoutingServiceClient, ArcGISPoint, ArcGISGeometry};
    /// use arcgis::{ServiceAreaParameters, NALocation};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let routing_service = RoutingServiceClient::new(
    ///     "https://route.arcgis.com/arcgis/rest/services/World/ServiceAreas/NAServer/ServiceArea_World",
    ///     &client
    /// );
    ///
    /// let facility = NALocation::new(ArcGISGeometry::Point(ArcGISPoint {
    ///     x: -122.4194,
    ///     y: 37.7749,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// })).with_name("Store");
    ///
    /// let params = ServiceAreaParameters::builder()
    ///     .facilities(vec![facility])
    ///     .default_breaks(vec![5.0, 10.0, 15.0])  // 5, 10, 15 minute drive times
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = routing_service.solve_service_area(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(facility_count = params.facilities().len()))]
    pub async fn solve_service_area(
        &self,
        params: ServiceAreaParameters,
    ) -> Result<ServiceAreaResult> {
        tracing::debug!("Solving service area");

        let url = format!("{}/solveServiceArea", self.base_url);

        tracing::debug!(url = %url, "Sending service area solve request");

        // Serialize facilities as features
        let facilities_json = serde_json::to_string(&serde_json::json!({
            "features": params.facilities().iter().map(|fac| {
                serde_json::json!({
                    "geometry": fac.geometry(),
                    "attributes": {
                        "Name": fac.name().as_ref().unwrap_or(&String::new()),
                    }
                })
            }).collect::<Vec<_>>()
        }))?;

        // Serialize break values
        let breaks_json = serde_json::to_string(params.default_breaks())?;

        let out_sr_str = params.out_sr().map(|sr| sr.to_string());

        let mut form = vec![
            ("facilities", facilities_json.as_str()),
            ("defaultBreaks", breaks_json.as_str()),
            ("f", "json"),
        ];

        if let Some(ref out_sr) = out_sr_str {
            form.push(("outSR", out_sr.as_str()));
        }

        if let Some(return_polygons) = params.return_polygons() {
            form.push((
                "returnPolygons",
                if *return_polygons { "true" } else { "false" },
            ));
        }

        // Add token if required by auth provider
        let token_opt = self.client.get_token_if_required().await?;
        let token_str;
        if let Some(token) = token_opt {
            token_str = token;
            form.push(("token", token_str.as_str()));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "solve service area request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: ServiceAreaResult = response.json().await?;

        tracing::info!(
            polygon_count = result.service_area_polygons().len(),
            "solve service area completed"
        );

        Ok(result)
    }

    /// Finds the closest facilities from incidents.
    ///
    /// Calculates routes from incidents to the N nearest facilities,
    /// useful for emergency response, service allocation, etc.
    ///
    /// # Arguments
    ///
    /// * `params` - Closest facility parameters including incidents and facilities
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, RoutingServiceClient, ArcGISPoint, ArcGISGeometry};
    /// use arcgis::{ClosestFacilityParameters, NALocation};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let routing_service = RoutingServiceClient::new(
    ///     "https://route.arcgis.com/arcgis/rest/services/World/ClosestFacility/NAServer/ClosestFacility_World",
    ///     &client
    /// );
    ///
    /// let incident = NALocation::new(ArcGISGeometry::Point(ArcGISPoint {
    ///     x: -122.4194,
    ///     y: 37.7749,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// })).with_name("Emergency");
    ///
    /// let facility = NALocation::new(ArcGISGeometry::Point(ArcGISPoint {
    ///     x: -122.4,
    ///     y: 37.8,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// })).with_name("Hospital");
    ///
    /// let params = ClosestFacilityParameters::builder()
    ///     .incidents(vec![incident])
    ///     .facilities(vec![facility])
    ///     .default_target_facility_count(1)
    ///     .return_routes(true)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = routing_service.solve_closest_facility(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(incident_count = params.incidents().len(), facility_count = params.facilities().len()))]
    pub async fn solve_closest_facility(
        &self,
        params: ClosestFacilityParameters,
    ) -> Result<ClosestFacilityResult> {
        tracing::debug!("Solving closest facility");

        let url = format!("{}/solveClosestFacility", self.base_url);

        tracing::debug!(url = %url, "Sending closest facility solve request");

        // Serialize incidents and facilities as features
        let incidents_json = serde_json::to_string(&serde_json::json!({
            "features": params.incidents().iter().map(|inc| {
                serde_json::json!({
                    "geometry": inc.geometry(),
                    "attributes": {
                        "Name": inc.name().as_ref().unwrap_or(&String::new()),
                    }
                })
            }).collect::<Vec<_>>()
        }))?;

        let facilities_json = serde_json::to_string(&serde_json::json!({
            "features": params.facilities().iter().map(|fac| {
                serde_json::json!({
                    "geometry": fac.geometry(),
                    "attributes": {
                        "Name": fac.name().as_ref().unwrap_or(&String::new()),
                    }
                })
            }).collect::<Vec<_>>()
        }))?;

        let out_sr_str = params.out_sr().map(|sr| sr.to_string());
        let target_count_str = params
            .default_target_facility_count()
            .map(|c| c.to_string());

        let mut form = vec![
            ("incidents", incidents_json.as_str()),
            ("facilities", facilities_json.as_str()),
            ("f", "json"),
        ];

        if let Some(ref out_sr) = out_sr_str {
            form.push(("outSR", out_sr.as_str()));
        }
        if let Some(ref target_count) = target_count_str {
            form.push(("defaultTargetFacilityCount", target_count.as_str()));
        }
        if let Some(return_routes) = params.return_routes() {
            form.push((
                "returnCFRoutes",
                if *return_routes { "true" } else { "false" },
            ));
        }

        if let Some(travel_direction) = params.travel_direction() {
            let direction_str = match travel_direction {
                crate::TravelDirection::FromFacility => "esriNATravelDirectionFromFacility",
                crate::TravelDirection::ToFacility => "esriNATravelDirectionToFacility",
            };
            form.push(("travelDirection", direction_str));
        }

        // Add token if required by auth provider
        let token_opt = self.client.get_token_if_required().await?;
        let token_str;
        if let Some(token) = token_opt {
            token_str = token;
            form.push(("token", token_str.as_str()));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "solve closest facility request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: ClosestFacilityResult = response.json().await?;

        tracing::info!(
            route_count = result.routes().len(),
            "solve closest facility completed"
        );

        Ok(result)
    }

    /// Generates an origin-destination cost matrix.
    ///
    /// Calculates travel costs between all origin-destination pairs,
    /// useful for logistics, fleet management, and coverage analysis.
    ///
    /// # Arguments
    ///
    /// * `params` - OD cost matrix parameters including origins and destinations
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, RoutingServiceClient, ArcGISPoint, ArcGISGeometry};
    /// use arcgis::{ODCostMatrixParameters, NALocation};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let routing_service = RoutingServiceClient::new(
    ///     "https://route.arcgis.com/arcgis/rest/services/World/OriginDestinationCostMatrix/NAServer/OriginDestinationCostMatrix_World",
    ///     &client
    /// );
    ///
    /// let origin = NALocation::new(ArcGISGeometry::Point(ArcGISPoint {
    ///     x: -122.4194,
    ///     y: 37.7749,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// })).with_name("Warehouse");
    ///
    /// let destination = NALocation::new(ArcGISGeometry::Point(ArcGISPoint {
    ///     x: -118.2437,
    ///     y: 34.0522,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// })).with_name("Customer");
    ///
    /// let params = ODCostMatrixParameters::builder()
    ///     .origins(vec![origin])
    ///     .destinations(vec![destination])
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = routing_service.generate_od_cost_matrix(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(origin_count = params.origins().len(), destination_count = params.destinations().len()))]
    pub async fn generate_od_cost_matrix(
        &self,
        params: ODCostMatrixParameters,
    ) -> Result<ODCostMatrixResult> {
        tracing::debug!("Generating OD cost matrix");

        let url = format!("{}/generateOriginDestinationCostMatrix", self.base_url);

        tracing::debug!(url = %url, "Sending OD cost matrix request");

        // Serialize origins and destinations as features
        let origins_json = serde_json::to_string(&serde_json::json!({
            "features": params.origins().iter().map(|org| {
                serde_json::json!({
                    "geometry": org.geometry(),
                    "attributes": {
                        "Name": org.name().as_ref().unwrap_or(&String::new()),
                    }
                })
            }).collect::<Vec<_>>()
        }))?;

        let destinations_json = serde_json::to_string(&serde_json::json!({
            "features": params.destinations().iter().map(|dest| {
                serde_json::json!({
                    "geometry": dest.geometry(),
                    "attributes": {
                        "Name": dest.name().as_ref().unwrap_or(&String::new()),
                    }
                })
            }).collect::<Vec<_>>()
        }))?;

        let out_sr_str = params.out_sr().map(|sr| sr.to_string());

        let mut form = vec![
            ("origins", origins_json.as_str()),
            ("destinations", destinations_json.as_str()),
            ("f", "json"),
        ];

        if let Some(ref out_sr) = out_sr_str {
            form.push(("outSR", out_sr.as_str()));
        }

        // Add token if required by auth provider
        let token_opt = self.client.get_token_if_required().await?;
        let token_str;
        if let Some(token) = token_opt {
            token_str = token;
            form.push(("token", token_str.as_str()));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "generate OD cost matrix request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: ODCostMatrixResult = response.json().await?;

        tracing::info!(
            od_line_count = result.od_lines().len(),
            "generate OD cost matrix completed"
        );

        Ok(result)
    }
}
