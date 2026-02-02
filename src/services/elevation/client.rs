//! Elevation service client implementation.

use crate::{ArcGISClient, ErrorKind, FeatureSet, GPExecuteResult, Result};
use tracing::instrument;

use super::types::{
    ProfileParameters, ProfileResult, SummarizeElevationParameters, SummarizeElevationResult,
    ViewshedParameters, ViewshedResult,
};

/// Client for interacting with ArcGIS Elevation Services.
///
/// The Elevation Service provides terrain analysis operations including
/// elevation profiles, statistics, and viewshed analysis.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient, ElevationClient, ProfileParametersBuilder};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// let elevation = ElevationClient::new(&client);
///
/// // Get elevation profile along a line
/// let params = ProfileParametersBuilder::default()
///     .input_geometry("{\"paths\":[[[-120,40],[-119,41]]]}")
///     .geometry_type("esriGeometryPolyline")
///     .build()
///     .expect("Valid parameters");
///
/// let result = elevation.profile(params).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ElevationClient<'a> {
    /// Base URL of the elevation service.
    url: String,

    /// Reference to the ArcGIS client.
    client: &'a ArcGISClient,
}

impl<'a> ElevationClient<'a> {
    /// Creates a new elevation service client.
    ///
    /// # Arguments
    ///
    /// * `client` - Reference to an [`ArcGISClient`] for making requests
    ///
    /// Uses the default ArcGIS Online Elevation Service URL.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ElevationClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let elevation = ElevationClient::new(&client);
    /// ```
    pub fn new(client: &'a ArcGISClient) -> Self {
        ElevationClient {
            url: "https://elevation.arcgis.com/arcgis/rest/services/Tools/ElevationSync/GPServer"
                .to_string(),
            client,
        }
    }

    /// Creates a new elevation service client with a custom URL.
    ///
    /// # Arguments
    ///
    /// * `url` - Base URL of the elevation service
    /// * `client` - Reference to an [`ArcGISClient`] for making requests
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ElevationClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let elevation = ElevationClient::with_url(
    ///     "https://custom-elevation.example.com/GPServer",
    ///     &client
    /// );
    /// ```
    pub fn with_url(url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        ElevationClient {
            url: url.into(),
            client,
        }
    }

    /// Generates an elevation profile along a line or points.
    ///
    /// Returns elevation values sampled along the input geometry,
    /// useful for creating cross-sections and elevation transects.
    ///
    /// # Arguments
    ///
    /// * `params` - Profile parameters (geometry, resolution, etc.)
    ///
    /// # Returns
    ///
    /// Profile result containing elevation data along the line.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ElevationClient, ProfileParametersBuilder};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let elevation = ElevationClient::new(&client);
    ///
    /// let params = ProfileParametersBuilder::default()
    ///     .input_geometry("{\"paths\":[[[-120.5,38.5],[-120.0,39.0]]]}")
    ///     .geometry_type("esriGeometryPolyline")
    ///     .dem_resolution("30m")
    ///     .return_first_point(true)
    ///     .return_last_point(true)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = elevation.profile(params).await?;
    ///
    /// if let Some(first) = result.first_point_z() {
    ///     tracing::info!(elevation = first, "First point elevation");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn profile(&self, params: ProfileParameters) -> Result<ProfileResult> {
        tracing::debug!("Generating elevation profile");

        let profile_url = format!("{}/Profile/execute", self.url);

        let mut request = self
            .client
            .http()
            .get(&profile_url)
            .query(&[("f", "json")])
            .query(&params);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        tracing::debug!(url = %profile_url, "Sending profile request");

        let response = request.send().await?;
        let response_body = response.text().await?;

        tracing::debug!(
            response_length = response_body.len(),
            response_body = %response_body,
            "Received profile response"
        );

        let gp_result: GPExecuteResult = serde_json::from_str(&response_body)?;

        tracing::debug!(
            result_count = gp_result.results().len(),
            message_count = gp_result.messages().len(),
            "Parsed GP execute result"
        );

        // Extract the OutputProfile FeatureSet from the GP result
        let output_param = gp_result
            .results()
            .first()
            .ok_or_else(|| {
                tracing::error!("GP result missing results array");
                crate::Error::from(ErrorKind::Api {
                    code: 0,
                    message: "Elevation profile result missing results array".to_string(),
                })
            })?;

        tracing::debug!(
            param_name = ?output_param.param_name(),
            data_type = ?output_param.data_type(),
            "Extracting profile parameter"
        );

        let feature_set_value = output_param.value().as_ref().ok_or_else(|| {
            tracing::error!("OutputProfile parameter missing value");
            crate::Error::from(ErrorKind::Api {
                code: 0,
                message: "Elevation profile parameter missing value field".to_string(),
            })
        })?;

        let feature_set: FeatureSet = serde_json::from_value(feature_set_value.clone())?;

        tracing::debug!(
            feature_count = feature_set.features().len(),
            geometry_type = ?feature_set.geometry_type(),
            "Extracted profile FeatureSet"
        );

        let result = ProfileResult::new(feature_set);

        tracing::debug!("Profile generated");

        Ok(result)
    }

    /// Summarizes elevation statistics within a polygon.
    ///
    /// Computes minimum, maximum, mean elevation and optionally
    /// slope and aspect statistics for the input area.
    ///
    /// # Arguments
    ///
    /// * `params` - Summarize parameters (polygon, resolution, etc.)
    ///
    /// # Returns
    ///
    /// Summary result containing elevation statistics.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ElevationClient, SummarizeElevationParametersBuilder};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let elevation = ElevationClient::new(&client);
    ///
    /// let params = SummarizeElevationParametersBuilder::default()
    ///     .input_geometry("{\"rings\":[[[-120,38],[-119,38],[-119,39],[-120,39],[-120,38]]]}")
    ///     .geometry_type("esriGeometryPolygon")
    ///     .dem_resolution("30m")
    ///     .include_slope(true)
    ///     .include_aspect(true)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = elevation.summarize_elevation(params).await?;
    ///
    /// if let Some(mean) = result.mean_elevation() {
    ///     tracing::info!(elevation = mean, "Mean elevation");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn summarize_elevation(
        &self,
        params: SummarizeElevationParameters,
    ) -> Result<SummarizeElevationResult> {
        tracing::debug!("Summarizing elevation");

        let summarize_url = format!("{}/SummarizeElevation/execute", self.url);

        let mut request = self
            .client
            .http()
            .get(&summarize_url)
            .query(&[("f", "json")])
            .query(&params);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        tracing::debug!(url = %summarize_url, "Sending summarize elevation request");

        let response = request.send().await?;
        let response_body = response.text().await?;

        tracing::debug!(
            response_length = response_body.len(),
            response_body = %response_body,
            "Received summarize elevation response"
        );

        let result: SummarizeElevationResult = serde_json::from_str(&response_body)?;

        tracing::debug!(
            min = ?result.min_elevation(),
            max = ?result.max_elevation(),
            mean = ?result.mean_elevation(),
            "Elevation summarized"
        );

        Ok(result)
    }

    /// Performs viewshed analysis from observer points.
    ///
    /// Determines visible areas from observer locations,
    /// accounting for terrain and viewing parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Viewshed parameters (observer points, distance, height, etc.)
    ///
    /// # Returns
    ///
    /// Viewshed result containing visible area polygons and statistics.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ElevationClient, ViewshedParametersBuilder};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let elevation = ElevationClient::new(&client);
    ///
    /// let params = ViewshedParametersBuilder::default()
    ///     .input_points("{\"points\":[[-120.0,38.5]]}")
    ///     .geometry_type("esriGeometryMultipoint")
    ///     .maximum_distance(5000.0)
    ///     .observer_height(2.0)
    ///     .dem_resolution("30m")
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = elevation.viewshed(params).await?;
    ///
    /// if let Some(percent) = result.percent_visible() {
    ///     tracing::info!(visible = percent, "Percent visible");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn viewshed(&self, params: ViewshedParameters) -> Result<ViewshedResult> {
        tracing::debug!(
            max_distance = ?params.maximum_distance(),
            observer_height = ?params.observer_height(),
            "Computing viewshed"
        );

        let viewshed_url = format!("{}/Viewshed/execute", self.url);

        let mut request = self
            .client
            .http()
            .get(&viewshed_url)
            .query(&[("f", "json")])
            .query(&params);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        tracing::debug!(url = %viewshed_url, "Sending viewshed request");

        let response = request.send().await?;
        let response_body = response.text().await?;

        tracing::debug!(
            response_length = response_body.len(),
            response_body = %response_body,
            "Received viewshed response"
        );

        let result: ViewshedResult = serde_json::from_str(&response_body)?;

        tracing::debug!(
            visible_area = ?result.visible_area(),
            percent = ?result.percent_visible(),
            "Viewshed computed"
        );

        Ok(result)
    }
}
