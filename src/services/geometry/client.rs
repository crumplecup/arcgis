//! Geometry Service client for geometric operations.

use crate::{ArcGISClient, ArcGISGeometry, Result};
use serde::Deserialize;
use tracing::instrument;

use super::{
    AreasAndLengthsParameters, AreasAndLengthsResult, BufferParameters, BufferResult,
    DistanceParameters, DistanceResult, ProjectParameters, ProjectResult, SimplifyParameters,
    SimplifyResult, Transformation, UnionParameters, UnionResult,
};

/// Client for interacting with an ArcGIS Geometry Service.
///
/// Provides operations for geometric transformations, analysis, and measurements.
///
/// # Example
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
///
/// let geometry_service = GeometryServiceClient::new(
///     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
///     &client,
/// );
/// # Ok(())
/// # }
/// ```
pub struct GeometryServiceClient<'a> {
    /// Base URL of the geometry service.
    base_url: String,
    /// Reference to the ArcGIS client for HTTP operations.
    client: &'a ArcGISClient,
}

impl<'a> GeometryServiceClient<'a> {
    /// Creates a new Geometry Service client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the geometry service (e.g., `https://utility.arcgisonline.com/.../GeometryServer`)
    /// * `client` - Reference to an authenticated ArcGIS client
    ///
    /// # Example
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geometry_service = GeometryServiceClient::new(
    ///     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
    ///     &client
    /// );
    /// ```
    pub fn new(base_url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        let base_url = base_url.into();
        tracing::debug!(base_url = %base_url, "Creating GeometryServiceClient");
        Self { base_url, client }
    }

    /// Projects geometries from one spatial reference to another.
    ///
    /// This operation transforms coordinates from the input spatial reference system
    /// to the output spatial reference system. Optionally uses datum transformations
    /// for accurate conversion between different datums.
    ///
    /// # Arguments
    ///
    /// * `geometries` - Geometries to project
    /// * `in_sr` - Input spatial reference WKID
    /// * `out_sr` - Output spatial reference WKID
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient, ArcGISPoint, ArcGISGeometry};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geometry_service = GeometryServiceClient::new(
    ///     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
    ///     &client
    /// );
    ///
    /// // Project from WGS84 (4326) to Web Mercator (3857)
    /// let point = ArcGISPoint {
    ///     x: -122.4194,
    ///     y: 37.7749,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// };
    /// let result = geometry_service
    ///     .project(vec![ArcGISGeometry::Point(point)], 4326, 3857)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, geometries), fields(in_sr = in_sr, out_sr = out_sr, geom_count = geometries.len()))]
    pub async fn project(
        &self,
        geometries: Vec<ArcGISGeometry>,
        in_sr: i32,
        out_sr: i32,
    ) -> Result<ProjectResult> {
        tracing::debug!("Projecting geometries");

        let params = ProjectParameters::builder()
            .geometries(geometries)
            .in_sr(in_sr)
            .out_sr(out_sr)
            .build()
            .expect("Valid parameters");

        self.project_with_params(params).await
    }

    /// Projects geometries with custom parameters.
    ///
    /// Allows specifying datum transformations and transformation direction.
    ///
    /// # Arguments
    ///
    /// * `params` - Project parameters including transformations
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient, ArcGISPoint, ArcGISGeometry, ProjectParameters};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geometry_service = GeometryServiceClient::new(
    ///     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
    ///     &client
    /// );
    ///
    /// let point = ArcGISPoint {
    ///     x: -122.4194,
    ///     y: 37.7749,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// };
    ///
    /// let params = ProjectParameters::builder()
    ///     .geometries(vec![ArcGISGeometry::Point(point)])
    ///     .in_sr(4326)
    ///     .out_sr(3857)
    ///     .transformation(1188)  // NAD_1983_To_WGS_1984_1
    ///     .transform_forward(true)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = geometry_service.project_with_params(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(in_sr = params.in_sr(), out_sr = params.out_sr()))]
    pub async fn project_with_params(&self, params: ProjectParameters) -> Result<ProjectResult> {
        tracing::debug!("Projecting geometries with params");

        let url = format!("{}/project", self.base_url);

        tracing::debug!(url = %url, "Sending project request");

        // Determine geometry type from first geometry
        let geometry_type = match params.geometries().first() {
            Some(crate::ArcGISGeometry::Point(_)) => "esriGeometryPoint",
            Some(crate::ArcGISGeometry::Multipoint(_)) => "esriGeometryMultipoint",
            Some(crate::ArcGISGeometry::Polyline(_)) => "esriGeometryPolyline",
            Some(crate::ArcGISGeometry::Polygon(_)) => "esriGeometryPolygon",
            Some(crate::ArcGISGeometry::Envelope(_)) => "esriGeometryEnvelope",
            None => {
                return Err(crate::Error::from(crate::ErrorKind::Other(
                    "No geometries to project".to_string(),
                )));
            }
        };

        // ArcGIS expects geometries parameter as JSON object with geometryType and geometries array
        // Format: {"geometryType":"esriGeometryPoint","geometries":[{"x":...,"y":...}]}
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct GeometriesWrapper<'a> {
            geometry_type: &'a str,
            geometries: &'a [crate::ArcGISGeometry],
        }

        let wrapper = GeometriesWrapper {
            geometry_type,
            geometries: params.geometries(),
        };
        let geometries_json = serde_json::to_string(&wrapper)?;
        tracing::debug!(geometries_json = %geometries_json, "Serialized geometries wrapper");

        let in_sr_str = params.in_sr().to_string();
        let out_sr_str = params.out_sr().to_string();

        // Use GET request with query parameters
        let mut request = self
            .client
            .http()
            .get(&url)
            .query(&[
                ("geometries", geometries_json.as_str()),
                ("inSR", in_sr_str.as_str()),
                ("outSR", out_sr_str.as_str()),
                ("f", "json"),
            ]);

        // Add token as query parameter if required
        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token.as_str())]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "project request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let response_text = response.text().await?;

        // Check for ArcGIS error response (HTTP 200 but with error payload)
        if response_text.contains("\"error\"") {
            tracing::error!(response = %response_text, "API returned error in response body");

            // Try to parse error details
            #[derive(serde::Deserialize)]
            struct ErrorResponse {
                error: ErrorDetail,
            }
            #[derive(serde::Deserialize)]
            struct ErrorDetail {
                code: i32,
                message: String,
            }

            if let Ok(err_resp) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(crate::Error::from(crate::ErrorKind::Api {
                    code: err_resp.error.code,
                    message: err_resp.error.message,
                }));
            } else {
                return Err(crate::Error::from(crate::ErrorKind::Api {
                    code: 0,
                    message: response_text,
                }));
            }
        }

        let result: ProjectResult = serde_json::from_str(&response_text)?;

        tracing::info!(
            result_count = result.geometries().len(),
            "project completed"
        );

        Ok(result)
    }

    /// Creates buffer polygons around geometries.
    ///
    /// Generates polygons at a specified distance from the input geometries.
    /// Supports geodesic (spherical) and planar buffering.
    ///
    /// # Arguments
    ///
    /// * `params` - Buffer parameters including geometries, distances, and units
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient, ArcGISPoint, ArcGISGeometry, BufferParameters, LinearUnit};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geometry_service = GeometryServiceClient::new(
    ///     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
    ///     &client
    /// );
    ///
    /// let point = ArcGISPoint {
    ///     x: -122.4194,
    ///     y: 37.7749,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// };
    ///
    /// let params = BufferParameters::builder()
    ///     .geometries(vec![ArcGISGeometry::Point(point)])
    ///     .in_sr(4326)
    ///     .distances(vec![1000.0])  // 1000 meters
    ///     .unit(LinearUnit::Meters)
    ///     .geodesic(true)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = geometry_service.buffer(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(geom_count = params.geometries().len()))]
    pub async fn buffer(&self, params: BufferParameters) -> Result<BufferResult> {
        tracing::debug!("Buffering geometries");

        let url = format!("{}/buffer", self.base_url);

        tracing::debug!(url = %url, "Sending buffer request");

        // Buffer uses simplified format for points: x1,y1,x2,y2,...
        // Unlike project operation which needs wrapper format!
        let geometries_param = match params.geometries().first() {
            Some(crate::ArcGISGeometry::Point(_)) => {
                // Simplified point format: x1,y1,x2,y2,...
                let coords: Vec<String> = params
                    .geometries()
                    .iter()
                    .filter_map(|g| {
                        if let crate::ArcGISGeometry::Point(p) = g {
                            Some(format!("{},{}", p.x, p.y))
                        } else {
                            None
                        }
                    })
                    .collect();
                coords.join(",")
            }
            Some(_) => {
                // For other geometry types, use JSON wrapper format
                let geometry_type = match params.geometries().first() {
                    Some(crate::ArcGISGeometry::Multipoint(_)) => "esriGeometryMultipoint",
                    Some(crate::ArcGISGeometry::Polyline(_)) => "esriGeometryPolyline",
                    Some(crate::ArcGISGeometry::Polygon(_)) => "esriGeometryPolygon",
                    Some(crate::ArcGISGeometry::Envelope(_)) => "esriGeometryEnvelope",
                    _ => unreachable!(),
                };

                #[derive(serde::Serialize)]
                #[serde(rename_all = "camelCase")]
                struct GeometriesWrapper<'a> {
                    geometry_type: &'a str,
                    geometries: &'a [crate::ArcGISGeometry],
                }

                let wrapper = GeometriesWrapper {
                    geometry_type,
                    geometries: params.geometries(),
                };
                serde_json::to_string(&wrapper)?
            }
            None => {
                return Err(crate::Error::from(crate::ErrorKind::Other(
                    "No geometries to buffer".to_string(),
                )));
            }
        };

        tracing::debug!(geometries_param = %geometries_param, "Serialized geometries for buffer");

        // Convert distances to comma-separated string
        let distances_str: Vec<String> = params.distances().iter().map(|d| d.to_string()).collect();
        let distances_param = distances_str.join(",");

        let in_sr_str = params.in_sr().to_string();

        // Determine bufferSR based on input SR
        // For geographic coordinate systems (like WGS84/4326), use Web Mercator (3857) for buffering
        // This allows proper distance calculations in meters
        let buffer_sr = if *params.in_sr() == 4326 {
            3857 // Web Mercator for WGS84 inputs
        } else {
            *params.in_sr() // Use input SR for projected systems
        };
        let buffer_sr_str = buffer_sr.to_string();

        // Build base query parameters
        // Note: When using geodesic=true, the unit parameter should NOT be sent
        // The bufferSR determines the distance units
        let mut request = self
            .client
            .http()
            .get(&url)
            .query(&[
                ("geometries", geometries_param.as_str()),
                ("inSR", in_sr_str.as_str()),
                ("bufferSR", buffer_sr_str.as_str()),
                ("distances", distances_param.as_str()),
                ("f", "json"),
            ]);

        // Add optional parameters
        if let Some(union) = params.union_results() {
            let union_str = union.to_string();
            request = request.query(&[("unionResults", union_str.as_str())]);
        }
        if let Some(geodesic) = params.geodesic() {
            let geodesic_str = geodesic.to_string();
            request = request.query(&[("geodesic", geodesic_str.as_str())]);
        }
        if let Some(out_sr) = params.out_sr() {
            let out_sr_str = out_sr.to_string();
            request = request.query(&[("outSR", out_sr_str.as_str())]);
        }

        // Add token as query parameter if required
        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token.as_str())]);
        }

        // Log all request parameters for debugging
        tracing::debug!(
            geometries = %geometries_param,
            inSR = %in_sr_str,
            bufferSR = %buffer_sr_str,
            distances = %distances_param,
            unionResults = ?params.union_results(),
            geodesic = ?params.geodesic(),
            outSR = ?params.out_sr(),
            "Buffer request parameters"
        );

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "buffer request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let response_text = response.text().await?;
        tracing::debug!(response = %response_text, "Buffer raw response");

        // Check for ArcGIS error response (HTTP 200 but with error payload)
        if response_text.contains("\"error\"") {
            tracing::error!(response = %response_text, "API returned error in response body");

            // Try to parse error details
            #[derive(serde::Deserialize)]
            struct ErrorResponse {
                error: ErrorDetail,
            }
            #[derive(serde::Deserialize)]
            struct ErrorDetail {
                code: i32,
                message: String,
            }

            if let Ok(err_resp) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(crate::Error::from(crate::ErrorKind::Api {
                    code: err_resp.error.code,
                    message: err_resp.error.message,
                }));
            } else {
                return Err(crate::Error::from(crate::ErrorKind::Api {
                    code: 0,
                    message: response_text,
                }));
            }
        }

        let result: BufferResult = serde_json::from_str(&response_text)?;

        tracing::info!(result_count = result.geometries().len(), "buffer completed");

        Ok(result)
    }

    /// Finds available datum transformations between spatial references.
    ///
    /// Returns a list of transformations that can be used to convert coordinates
    /// between the specified spatial reference systems with improved accuracy.
    ///
    /// # Arguments
    ///
    /// * `in_sr` - Input spatial reference WKID
    /// * `out_sr` - Output spatial reference WKID
    /// * `extent_of_interest` - Optional extent to filter transformations
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geometry_service = GeometryServiceClient::new(
    ///     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
    ///     &client
    /// );
    ///
    /// // Find transformations from NAD83 to WGS84
    /// let transformations = geometry_service
    ///     .find_transformations(4269, 4326, None)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(in_sr = in_sr, out_sr = out_sr))]
    pub async fn find_transformations(
        &self,
        in_sr: i32,
        out_sr: i32,
        extent_of_interest: Option<crate::ArcGISEnvelope>,
    ) -> Result<Vec<Transformation>> {
        tracing::debug!("Finding datum transformations");

        let url = format!("{}/findTransformations", self.base_url);

        tracing::debug!(url = %url, "Sending findTransformations request");

        let mut form = vec![
            ("inSR", in_sr.to_string()),
            ("outSR", out_sr.to_string()),
            ("f", "json".to_string()),
        ];

        // Add token if required by auth provider
        let token_opt = self.client.get_token_if_required().await?;
        let token_str;
        if let Some(token) = token_opt {
            token_str = token;
            form.push(("token", token_str));
        }

        if let Some(extent) = extent_of_interest {
            let extent_json = serde_json::to_string(&extent)?;
            form.push(("extentOfInterest", extent_json));
        }

        let response = self
            .client
            .http()
            .get(&url)
            .query(
                &form
                    .iter()
                    .map(|(k, v)| (*k, v.as_str()))
                    .collect::<Vec<_>>(),
            )
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "findTransformations request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        #[derive(Deserialize)]
        struct TransformationsResponse {
            transformations: Vec<Transformation>,
        }

        let response_data: TransformationsResponse = response.json().await?;

        tracing::info!(
            transformation_count = response_data.transformations.len(),
            "findTransformations completed"
        );

        Ok(response_data.transformations)
    }

    /// Simplifies geometries to remove topological errors.
    ///
    /// Fixes issues like self-intersections, ring orientation, and vertex ordering.
    /// This is often required before other geometric operations.
    ///
    /// # Arguments
    ///
    /// * `params` - Simplify parameters including geometries and spatial reference
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient, ArcGISPolygon, ArcGISGeometry, SimplifyParameters};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geometry_service = GeometryServiceClient::new(
    ///     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
    ///     &client
    /// );
    ///
    /// let polygon = ArcGISPolygon {
    ///     rings: vec![vec![
    ///         [-122.0, 37.0],
    ///         [-122.0, 38.0],
    ///         [-121.0, 38.0],
    ///         [-121.0, 37.0],
    ///         [-122.0, 37.0],
    ///     ]],
    ///     spatial_reference: None,
    /// };
    ///
    /// let params = SimplifyParameters::builder()
    ///     .geometries(vec![ArcGISGeometry::Polygon(polygon)])
    ///     .sr(4326)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = geometry_service.simplify(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(geom_count = params.geometries().len()))]
    pub async fn simplify(&self, params: SimplifyParameters) -> Result<SimplifyResult> {
        tracing::debug!("Simplifying geometries");

        let url = format!("{}/simplify", self.base_url);

        tracing::debug!(url = %url, "Sending simplify request");

        let params_json = serde_json::to_string(&params)?;
        let mut form = vec![
            ("geometries", params_json.as_str()),
            ("f", "json"),
        ];

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
            tracing::error!(status = %status, error = %error_text, "simplify request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: SimplifyResult = response.json().await?;

        tracing::info!(
            result_count = result.geometries().len(),
            "simplify completed"
        );

        Ok(result)
    }

    /// Unions multiple geometries into a single geometry.
    ///
    /// Combines all input geometries into one unified geometry, removing overlaps.
    ///
    /// # Arguments
    ///
    /// * `params` - Union parameters including geometries and spatial reference
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient, ArcGISPolygon, ArcGISGeometry, UnionParameters};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geometry_service = GeometryServiceClient::new(
    ///     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
    ///     &client
    /// );
    ///
    /// let polygon1 = ArcGISPolygon {
    ///     rings: vec![vec![
    ///         [-122.0, 37.0],
    ///         [-122.0, 38.0],
    ///         [-121.0, 38.0],
    ///         [-121.0, 37.0],
    ///         [-122.0, 37.0],
    ///     ]],
    ///     spatial_reference: None,
    /// };
    ///
    /// let polygon2 = ArcGISPolygon {
    ///     rings: vec![vec![
    ///         [-121.5, 37.5],
    ///         [-121.5, 38.5],
    ///         [-120.5, 38.5],
    ///         [-120.5, 37.5],
    ///         [-121.5, 37.5],
    ///     ]],
    ///     spatial_reference: None,
    /// };
    ///
    /// let params = UnionParameters::builder()
    ///     .geometries(vec![ArcGISGeometry::Polygon(polygon1), ArcGISGeometry::Polygon(polygon2)])
    ///     .sr(4326)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = geometry_service.union(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(geom_count = params.geometries().len()))]
    pub async fn union(&self, params: UnionParameters) -> Result<UnionResult> {
        tracing::debug!("Unioning geometries");

        let url = format!("{}/union", self.base_url);

        tracing::debug!(url = %url, "Sending union request");

        let params_json = serde_json::to_string(&params)?;
        let mut form = vec![
            ("geometries", params_json.as_str()),
            ("f", "json"),
        ];

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
            tracing::error!(status = %status, error = %error_text, "union request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: UnionResult = response.json().await?;

        tracing::info!("union completed");

        Ok(result)
    }

    /// Calculates areas and perimeter lengths for polygon geometries.
    ///
    /// Supports both planar and geodesic calculations with configurable units.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including polygons, spatial reference, and units
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient, ArcGISPolygon, ArcGISGeometry};
    /// use arcgis::{AreasAndLengthsParameters, LinearUnit, AreaUnit, CalculationType};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geometry_service = GeometryServiceClient::new(
    ///     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
    ///     &client
    /// );
    ///
    /// let polygon = ArcGISPolygon {
    ///     rings: vec![vec![
    ///         [-122.0, 37.0],
    ///         [-122.0, 38.0],
    ///         [-121.0, 38.0],
    ///         [-121.0, 37.0],
    ///         [-122.0, 37.0],
    ///     ]],
    ///     spatial_reference: None,
    /// };
    ///
    /// let params = AreasAndLengthsParameters::builder()
    ///     .polygons(vec![ArcGISGeometry::Polygon(polygon)])
    ///     .sr(4326)
    ///     .length_unit(LinearUnit::Meters)
    ///     .area_unit(AreaUnit::SquareMeters)
    ///     .calculation_type(CalculationType::Geodesic)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = geometry_service.areas_and_lengths(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(polygon_count = params.polygons().len()))]
    pub async fn areas_and_lengths(
        &self,
        params: AreasAndLengthsParameters,
    ) -> Result<AreasAndLengthsResult> {
        tracing::debug!("Calculating areas and lengths");

        let url = format!("{}/areasAndLengths", self.base_url);

        tracing::debug!(url = %url, "Sending areasAndLengths request");

        let params_json = serde_json::to_string(&params)?;
        let mut form = vec![
            ("polygons", params_json.as_str()),
            ("f", "json"),
        ];

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
            tracing::error!(status = %status, error = %error_text, "areasAndLengths request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: AreasAndLengthsResult = response.json().await?;

        tracing::info!(
            result_count = result.areas().len(),
            "areasAndLengths completed"
        );

        Ok(result)
    }

    /// Calculates the distance between two geometries.
    ///
    /// Supports both planar and geodesic distance calculations.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including geometries, spatial reference, and units
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient, ArcGISPoint, ArcGISGeometry};
    /// use arcgis::{DistanceParameters, LinearUnit};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geometry_service = GeometryServiceClient::new(
    ///     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
    ///     &client
    /// );
    ///
    /// let point1 = ArcGISPoint {
    ///     x: -122.4194,
    ///     y: 37.7749,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// };
    ///
    /// let point2 = ArcGISPoint {
    ///     x: -118.2437,
    ///     y: 34.0522,
    ///     z: None,
    ///     m: None,
    ///     spatial_reference: None,
    /// };
    ///
    /// let params = DistanceParameters::builder()
    ///     .geometry1(ArcGISGeometry::Point(point1))
    ///     .geometry2(ArcGISGeometry::Point(point2))
    ///     .sr(4326)
    ///     .distance_unit(LinearUnit::Kilometers)
    ///     .geodesic(true)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = geometry_service.distance(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn distance(&self, params: DistanceParameters) -> Result<DistanceResult> {
        tracing::debug!("Calculating distance");

        let url = format!("{}/distance", self.base_url);

        tracing::debug!(url = %url, "Sending distance request");

        // Distance operation requires nested format: {"geometryType":"esriGeometryPoint","geometry":{...}}
        let geometry_type1 = match params.geometry1() {
            crate::ArcGISGeometry::Point(_) => "esriGeometryPoint",
            crate::ArcGISGeometry::Multipoint(_) => "esriGeometryMultipoint",
            crate::ArcGISGeometry::Polyline(_) => "esriGeometryPolyline",
            crate::ArcGISGeometry::Polygon(_) => "esriGeometryPolygon",
            crate::ArcGISGeometry::Envelope(_) => "esriGeometryEnvelope",
        };

        let geometry_type2 = match params.geometry2() {
            crate::ArcGISGeometry::Point(_) => "esriGeometryPoint",
            crate::ArcGISGeometry::Multipoint(_) => "esriGeometryMultipoint",
            crate::ArcGISGeometry::Polyline(_) => "esriGeometryPolyline",
            crate::ArcGISGeometry::Polygon(_) => "esriGeometryPolygon",
            crate::ArcGISGeometry::Envelope(_) => "esriGeometryEnvelope",
        };

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct GeometryWrapper<'a> {
            geometry_type: &'a str,
            geometry: &'a crate::ArcGISGeometry,
        }

        let geometry1_wrapper = GeometryWrapper {
            geometry_type: geometry_type1,
            geometry: params.geometry1(),
        };

        let geometry2_wrapper = GeometryWrapper {
            geometry_type: geometry_type2,
            geometry: params.geometry2(),
        };

        let geometry1_json = serde_json::to_string(&geometry1_wrapper)?;
        let geometry2_json = serde_json::to_string(&geometry2_wrapper)?;
        let sr_str = params.sr().to_string();

        tracing::debug!(
            geometry1 = %geometry1_json,
            geometry2 = %geometry2_json,
            sr = %sr_str,
            geodesic = ?params.geodesic(),
            distance_unit = ?params.distance_unit(),
            "Distance request parameters"
        );

        // Build GET request (like other geometry operations)
        let mut request = self
            .client
            .http()
            .get(&url)
            .query(&[
                ("geometry1", geometry1_json.as_str()),
                ("geometry2", geometry2_json.as_str()),
                ("sr", sr_str.as_str()),
                ("f", "json"),
            ]);

        // Add optional parameters
        // Note: When using geodesic=true, do NOT send distanceUnit
        // Geodesic calculations return distance in meters
        if let Some(geodesic) = params.geodesic() {
            let geodesic_str = geodesic.to_string();
            request = request.query(&[("geodesic", geodesic_str.as_str())]);

            // Only add distanceUnit if NOT using geodesic
            if !geodesic {
                if let Some(unit) = params.distance_unit() {
                    let unit_str = match unit {
                        crate::LinearUnit::Meters => "esriMeters",
                        crate::LinearUnit::Kilometers => "esriKilometers",
                        crate::LinearUnit::Feet => "esriFeet",
                        crate::LinearUnit::Miles => "esriMiles",
                        crate::LinearUnit::NauticalMiles => "esriNauticalMiles",
                        crate::LinearUnit::Yards => "esriYards",
                    };
                    request = request.query(&[("distanceUnit", unit_str)]);
                }
            }
        } else {
            // No geodesic parameter, add distanceUnit if provided
            if let Some(unit) = params.distance_unit() {
                let unit_str = match unit {
                    crate::LinearUnit::Meters => "esriMeters",
                    crate::LinearUnit::Kilometers => "esriKilometers",
                    crate::LinearUnit::Feet => "esriFeet",
                    crate::LinearUnit::Miles => "esriMiles",
                    crate::LinearUnit::NauticalMiles => "esriNauticalMiles",
                    crate::LinearUnit::Yards => "esriYards",
                };
                request = request.query(&[("distanceUnit", unit_str)]);
            }
        }

        // Add token as query parameter if required
        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token.as_str())]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "distance request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let response_text = response.text().await?;
        tracing::debug!(response = %response_text, "Distance raw response");

        // Check for ArcGIS error response (HTTP 200 but with error payload)
        if response_text.contains("\"error\"") {
            tracing::error!(response = %response_text, "API returned error in response body");

            #[derive(serde::Deserialize)]
            struct ErrorResponse {
                error: ErrorDetail,
            }
            #[derive(serde::Deserialize)]
            struct ErrorDetail {
                code: i32,
                message: String,
            }

            if let Ok(err_resp) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(crate::Error::from(crate::ErrorKind::Api {
                    code: err_resp.error.code,
                    message: err_resp.error.message,
                }));
            } else {
                return Err(crate::Error::from(crate::ErrorKind::Api {
                    code: 0,
                    message: response_text,
                }));
            }
        }

        let result: DistanceResult = serde_json::from_str(&response_text)?;

        tracing::info!(distance = result.distance(), "distance completed");

        Ok(result)
    }
}
