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
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending project request");

        let params_json = serde_json::to_string(&params)?;
        let form = vec![
            ("geometries", params_json.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

        let response = self.client.http().post(&url).form(&form).send().await?;

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

        let result: ProjectResult = response.json().await?;

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
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending buffer request");

        let params_json = serde_json::to_string(&params)?;
        let form = vec![
            ("bufferParameters", params_json.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

        let response = self.client.http().post(&url).form(&form).send().await?;

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

        let result: BufferResult = response.json().await?;

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
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending findTransformations request");

        let mut form = vec![
            ("inSR", in_sr.to_string()),
            ("outSR", out_sr.to_string()),
            ("f", "json".to_string()),
            ("token", token.to_string()),
        ];

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
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending simplify request");

        let params_json = serde_json::to_string(&params)?;
        let form = vec![
            ("geometries", params_json.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

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
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending union request");

        let params_json = serde_json::to_string(&params)?;
        let form = vec![
            ("geometries", params_json.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

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
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending areasAndLengths request");

        let params_json = serde_json::to_string(&params)?;
        let form = vec![
            ("polygons", params_json.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

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
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending distance request");

        let params_json = serde_json::to_string(&params)?;
        let form = vec![
            ("geometry1", params_json.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

        let response = self.client.http().post(&url).form(&form).send().await?;

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

        let result: DistanceResult = response.json().await?;

        tracing::info!(distance = result.distance(), "distance completed");

        Ok(result)
    }
}
