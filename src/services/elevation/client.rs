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
/// // Get elevation profile along a line (FeatureSet JSON with a polyline)
/// let line_features = r#"{"geometryType":"esriGeometryPolyline","features":[{"geometry":{"paths":[[[-120,40],[-119,41]]]}}],"spatialReference":{"wkid":4326}}"#;
/// let params = ProfileParametersBuilder::default()
///     .input_line_features(line_features)
///     .dem_resolution("30m")
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
    /// let line_features = r#"{"geometryType":"esriGeometryPolyline","features":[{"geometry":{"paths":[[[-120.5,38.5],[-120.0,39.0]]]}}],"spatialReference":{"wkid":4326}}"#;
    /// let params = ProfileParametersBuilder::default()
    ///     .input_line_features(line_features)
    ///     .dem_resolution("30m")
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = elevation.profile(params).await?;
    ///
    /// tracing::info!(
    ///     point_count = result.output_profile().features().len(),
    ///     "Elevation profile generated"
    /// );
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
        let output_param = gp_result.results().first().ok_or_else(|| {
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

    /// Submits an asynchronous SummarizeElevation job.
    ///
    /// Computes elevation, slope, and aspect statistics for input features.
    /// Returns a job ID that can be used to poll for completion.
    ///
    /// # Arguments
    ///
    /// * `params` - Summarize parameters (features, resolution, etc.)
    ///
    /// # Returns
    ///
    /// Job information including job ID and initial status.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ApiKeyTier, ArcGISClient, ElevationClient, SummarizeElevationParametersBuilder, DemResolution};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::agol(ApiKeyTier::Location)?;
    /// let client = ArcGISClient::new(auth);
    /// let elevation = ElevationClient::new(&client);
    ///
    /// let polygon = r#"{"geometryType":"esriGeometryPolygon","spatialReference":{"wkid":4326},"features":[{"geometry":{"rings":[[[-119.5,37.8],[-119.4,37.8],[-119.4,37.9],[-119.5,37.9],[-119.5,37.8]]]},"attributes":{"OID":1}}]}"#;
    ///
    /// let params = SummarizeElevationParametersBuilder::default()
    ///     .input_features(polygon)
    ///     .dem_resolution(DemResolution::ThirtyMeter.as_str())
    ///     .include_slope_aspect(true)
    ///     .build()?;
    ///
    /// let job = elevation.submit_summarize_elevation(params).await?;
    /// tracing::info!(job_id = %job.job_id(), "Job submitted");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn submit_summarize_elevation(
        &self,
        params: SummarizeElevationParameters,
    ) -> Result<crate::GPJobInfo> {
        tracing::debug!("Submitting SummarizeElevation job");

        // Create GP service client for async Elevation service
        let gp_service = crate::GeoprocessingServiceClient::new(
            "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/SummarizeElevation",
            self.client,
        );

        // Convert params to HashMap
        let param_map = self.params_to_hashmap(&params)?;

        // Submit job
        let job = gp_service.submit_job(param_map).await?;

        tracing::info!(
            job_id = %job.job_id(),
            status = ?job.job_status(),
            "SummarizeElevation job submitted"
        );

        Ok(job)
    }

    /// Polls a SummarizeElevation job until completion and returns the typed result.
    ///
    /// This is a convenience method that combines status polling with result extraction.
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job identifier from `submit_summarize_elevation`
    /// * `timeout_ms` - Optional timeout in milliseconds (default: 60000)
    ///
    /// # Returns
    ///
    /// Typed result containing elevation statistics.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ApiKeyAuth, ApiKeyTier, ArcGISClient, ElevationClient};
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::agol(ApiKeyTier::Location)?;
    /// # let client = ArcGISClient::new(auth);
    /// # let elevation = ElevationClient::new(&client);
    /// # let job = elevation.submit_summarize_elevation(Default::default()).await?;
    /// let result = elevation.poll_summarize_elevation(job.job_id(), None).await?;
    ///
    /// if let Some(mean) = result.mean_elevation() {
    ///     tracing::info!(elevation_m = mean, "Mean elevation");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(job_id, timeout_ms))]
    pub async fn poll_summarize_elevation(
        &self,
        job_id: &str,
        timeout_ms: Option<u64>,
    ) -> Result<SummarizeElevationResult> {
        tracing::debug!("Polling SummarizeElevation job");

        let gp_service = crate::GeoprocessingServiceClient::new(
            "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/SummarizeElevation",
            self.client,
        );

        // Poll until complete
        let job_info = gp_service
            .poll_until_complete(job_id, 2000, 5000, timeout_ms.or(Some(60000)))
            .await?;

        tracing::debug!(
            job_id = %job_info.job_id(),
            status = ?job_info.job_status(),
            "Job completed"
        );

        // Extract result from GP response
        self.extract_summarize_result(&job_info).await
    }

    /// Helper to convert typed params to HashMap for GP service.
    fn params_to_hashmap<T: serde::Serialize>(
        &self,
        params: &T,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        use std::collections::HashMap;

        let json_value = serde_json::to_value(params)?;
        let map = json_value
            .as_object()
            .ok_or_else(|| crate::BuilderError::new("Failed to convert params to map"))?
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<String, serde_json::Value>>();
        Ok(map)
    }

    /// Helper to extract SummarizeElevationResult from GP job info.
    async fn extract_summarize_result(
        &self,
        job_info: &crate::GPJobInfo,
    ) -> Result<SummarizeElevationResult> {
        tracing::debug!("Extracting SummarizeElevation result from GP response");

        // Get results map
        let results = job_info.results();

        // Log all available result parameters
        tracing::debug!(
            result_count = results.len(),
            result_keys = ?results.keys().collect::<Vec<_>>(),
            "Available result parameters"
        );

        // Get OutputSummary parameter
        let output_summary_param = results.get("OutputSummary").ok_or_else(|| {
            tracing::error!("Results missing OutputSummary parameter");
            crate::Error::from(ErrorKind::Api {
                code: 0,
                message: "SummarizeElevation result missing OutputSummary parameter".to_string(),
            })
        })?;

        // Get the value - either directly or by fetching from paramUrl
        let output_summary = if let Some(value) = output_summary_param.value() {
            // Value is directly available
            tracing::debug!("Using OutputSummary value from inline response");
            value.clone()
        } else if let Some(param_url) = output_summary_param.param_url() {
            // Need to fetch from URL
            tracing::debug!(param_url = %param_url, "Fetching OutputSummary from paramUrl");

            // Create GP client to fetch the result
            let gp_service = crate::GeoprocessingServiceClient::new(
                "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/SummarizeElevation",
                self.client,
            );

            // Fetch the result data
            let result_json = gp_service
                .get_result_data(job_info.job_id(), "OutputSummary")
                .await?;

            // Extract the "value" field which contains the actual FeatureSet
            result_json
                .get("value")
                .ok_or_else(|| {
                    tracing::error!("Result data missing 'value' field");
                    crate::Error::from(ErrorKind::Api {
                        code: 0,
                        message: "OutputSummary result data missing 'value' field".to_string(),
                    })
                })?
                .clone()
        } else {
            tracing::error!("OutputSummary parameter has neither value nor paramUrl");
            return Err(crate::Error::from(ErrorKind::Api {
                code: 0,
                message: "OutputSummary parameter has no value or paramUrl".to_string(),
            }));
        };

        // Parse as FeatureSet
        let feature_set: FeatureSet = serde_json::from_value(output_summary.clone())?;

        tracing::debug!(
            feature_count = feature_set.features().len(),
            "Parsed OutputSummary FeatureSet"
        );

        // Extract statistics from feature attributes
        let result = SummarizeElevationResult::from_feature_set(&feature_set).map_err(|e| {
            crate::Error::from(ErrorKind::Api {
                code: 0,
                message: format!("Failed to parse elevation statistics: {}", e),
            })
        })?;

        Ok(result)
    }

    /// Submits an asynchronous Viewshed job.
    ///
    /// Determines visible areas from observer points based on terrain.
    /// Returns a job ID that can be used to poll for completion.
    ///
    /// # Arguments
    ///
    /// * `params` - Viewshed parameters (observer points, distance, height, etc.)
    ///
    /// # Returns
    ///
    /// Job information including job ID and initial status.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ApiKeyTier, ArcGISClient, ElevationClient, ViewshedParametersBuilder, DemResolution};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::agol(ApiKeyTier::Location)?;
    /// let client = ArcGISClient::new(auth);
    /// let elevation = ElevationClient::new(&client);
    ///
    /// let observer = r#"{"geometryType":"esriGeometryMultipoint","spatialReference":{"wkid":4326},"points":[[-119.5,37.85]]}"#;
    ///
    /// let params = ViewshedParametersBuilder::default()
    ///     .input_points(observer)
    ///     .maximum_distance(5000.0)
    ///     .maximum_distance_units("Meters")
    ///     .observer_height(1.75)
    ///     .dem_resolution(DemResolution::ThirtyMeter.as_str())
    ///     .build()?;
    ///
    /// let job = elevation.submit_viewshed(params).await?;
    /// tracing::info!(job_id = %job.job_id(), "Job submitted");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn submit_viewshed(&self, params: ViewshedParameters) -> Result<crate::GPJobInfo> {
        tracing::debug!("Submitting Viewshed job");

        // Create GP service client for async Elevation service
        let gp_service = crate::GeoprocessingServiceClient::new(
            "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/Viewshed",
            self.client,
        );

        // Convert params to HashMap
        let param_map = self.params_to_hashmap(&params)?;

        // Submit job
        let job = gp_service.submit_job(param_map).await?;

        tracing::info!(
            job_id = %job.job_id(),
            status = ?job.job_status(),
            "Viewshed job submitted"
        );

        Ok(job)
    }

    /// Polls a Viewshed job until completion and returns the typed result.
    ///
    /// This is a convenience method that combines status polling with result extraction.
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job identifier from `submit_viewshed`
    /// * `timeout_ms` - Optional timeout in milliseconds (default: 60000)
    ///
    /// # Returns
    ///
    /// Typed result containing viewshed polygon FeatureSet.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ApiKeyAuth, ApiKeyTier, ArcGISClient, ElevationClient};
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::agol(ApiKeyTier::Location)?;
    /// # let client = ArcGISClient::new(auth);
    /// # let elevation = ElevationClient::new(&client);
    /// # let job = elevation.submit_viewshed(Default::default()).await?;
    /// let result = elevation.poll_viewshed(job.job_id(), None).await?;
    ///
    /// tracing::info!(
    ///     viewshed_count = result.viewshed_count(),
    ///     "Viewshed analysis complete"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(job_id, timeout_ms))]
    pub async fn poll_viewshed(
        &self,
        job_id: &str,
        timeout_ms: Option<u64>,
    ) -> Result<ViewshedResult> {
        tracing::debug!("Polling Viewshed job");

        let gp_service = crate::GeoprocessingServiceClient::new(
            "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/Viewshed",
            self.client,
        );

        // Poll until complete
        let job_info = gp_service
            .poll_until_complete(job_id, 2000, 5000, timeout_ms.or(Some(60000)))
            .await?;

        tracing::debug!(
            job_id = %job_info.job_id(),
            status = ?job_info.job_status(),
            "Job completed"
        );

        // Extract result from GP response
        self.extract_viewshed_result(&job_info).await
    }

    /// Helper to extract ViewshedResult from GP job info.
    async fn extract_viewshed_result(&self, job_info: &crate::GPJobInfo) -> Result<ViewshedResult> {
        tracing::debug!("Extracting Viewshed result from GP response");

        // Get results map
        let results = job_info.results();

        // Get OutputViewshed parameter
        let output_viewshed_param = results.get("OutputViewshed").ok_or_else(|| {
            tracing::error!("Results missing OutputViewshed parameter");
            crate::Error::from(ErrorKind::Api {
                code: 0,
                message: "Viewshed result missing OutputViewshed parameter".to_string(),
            })
        })?;

        // Get the value - either directly or by fetching from paramUrl
        let output_viewshed = if let Some(value) = output_viewshed_param.value() {
            // Value is directly available
            tracing::debug!("Using OutputViewshed value from inline response");
            value.clone()
        } else if let Some(param_url) = output_viewshed_param.param_url() {
            // Need to fetch from URL
            tracing::debug!(param_url = %param_url, "Fetching OutputViewshed from paramUrl");

            // Create GP client to fetch the result
            let gp_service = crate::GeoprocessingServiceClient::new(
                "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/Viewshed",
                self.client,
            );

            // Fetch the result data
            let result_json = gp_service
                .get_result_data(job_info.job_id(), "OutputViewshed")
                .await?;

            // Extract the "value" field which contains the actual FeatureSet
            result_json
                .get("value")
                .ok_or_else(|| {
                    tracing::error!("Result data missing 'value' field");
                    crate::Error::from(ErrorKind::Api {
                        code: 0,
                        message: "OutputViewshed result data missing 'value' field".to_string(),
                    })
                })?
                .clone()
        } else {
            tracing::error!("OutputViewshed parameter has neither value nor paramUrl");
            return Err(crate::Error::from(ErrorKind::Api {
                code: 0,
                message: "OutputViewshed parameter has no value or paramUrl".to_string(),
            }));
        };

        // Parse as FeatureSet
        let feature_set: FeatureSet = serde_json::from_value(output_viewshed.clone())?;

        tracing::debug!(
            feature_count = feature_set.features().len(),
            "Parsed OutputViewshed FeatureSet"
        );

        let result = ViewshedResult::new(feature_set);

        Ok(result)
    }
}
