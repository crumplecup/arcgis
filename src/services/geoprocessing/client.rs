//! Geoprocessing service client implementation.

use crate::{ArcGISClient, BuilderError, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::instrument;

use super::types::{GPExecuteResult, GPJobInfo, GPMessage};

/// Client for interacting with ArcGIS Geoprocessing Services (GPServer).
///
/// The Geoprocessing Service enables execution of server-side geoprocessing tasks.
/// It supports both synchronous (immediate) and asynchronous (job-based) execution.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient, GeoprocessingServiceClient};
/// use std::collections::HashMap;
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// let gp_service = GeoprocessingServiceClient::new(
///     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/Elevation/ESRI_Elevation_World/GPServer/ProfileService",
///     &client
/// );
///
/// // Execute synchronous task
/// let mut params = HashMap::new();
/// params.insert("InputParameter".to_string(), serde_json::json!("value"));
/// let result = gp_service.execute(params).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct GeoprocessingServiceClient<'a> {
    /// Base URL of the geoprocessing service.
    url: String,

    /// Reference to the ArcGIS client.
    client: &'a ArcGISClient,
}

impl<'a> GeoprocessingServiceClient<'a> {
    /// Creates a new geoprocessing service client.
    ///
    /// # Arguments
    ///
    /// * `url` - Base URL of the geoprocessing service (e.g., `https://server/arcgis/rest/services/Folder/ServiceName/GPServer/TaskName`)
    /// * `client` - Reference to an [`ArcGISClient`] for making requests
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeoprocessingServiceClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let gp_service = GeoprocessingServiceClient::new(
    ///     "https://server/arcgis/rest/services/GP/MyTask/GPServer/Execute",
    ///     &client
    /// );
    /// ```
    pub fn new(url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        GeoprocessingServiceClient {
            url: url.into(),
            client,
        }
    }

    /// Executes a synchronous geoprocessing task.
    ///
    /// Synchronous execution is appropriate for tasks that complete quickly (typically < 30 seconds).
    /// For longer-running tasks, use [`submit_job`](Self::submit_job) for asynchronous execution.
    ///
    /// # Arguments
    ///
    /// * `parameters` - HashMap of parameter name to parameter value (as JSON)
    ///
    /// # Returns
    ///
    /// Result containing execution results and messages.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeoprocessingServiceClient};
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let gp_service = GeoprocessingServiceClient::new(
    ///     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/Elevation/ESRI_Elevation_World/GPServer/ProfileService",
    ///     &client
    /// );
    ///
    /// let mut params = HashMap::new();
    /// params.insert("InputLineFeatures".to_string(), serde_json::json!({
    ///     "geometryType": "esriGeometryPolyline",
    ///     "features": []
    /// }));
    ///
    /// let result = gp_service.execute(params).await?;
    /// tracing::info!(result_count = result.results().len(), "Task completed");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, parameters), fields(param_count = parameters.len()))]
    pub async fn execute(&self, parameters: HashMap<String, Value>) -> Result<GPExecuteResult> {
        tracing::debug!("Executing synchronous geoprocessing task");

        let execute_url = format!("{}/execute", self.url);

        let mut form: Vec<(&str, String)> = Vec::new();
        form.push(("f", "json".to_string()));

        // Serialize parameters as JSON
        for (key, value) in parameters.iter() {
            let value_str = serde_json::to_string(value)?;
            form.push((key.as_str(), value_str));
        }

        let response = self
            .client
            .http()
            .post(&execute_url)
            .form(&form)
            .send()
            .await?;

        let result: GPExecuteResult = response.json().await?;

        tracing::debug!(
            result_count = result.results().len(),
            message_count = result.messages().len(),
            "Task execution completed"
        );

        Ok(result)
    }

    /// Submits an asynchronous geoprocessing job.
    ///
    /// Use this for long-running tasks. After submission, use [`get_job_status`](Self::get_job_status)
    /// to check progress and [`get_job_result`](Self::get_job_result) to retrieve results.
    ///
    /// # Arguments
    ///
    /// * `parameters` - HashMap of parameter name to parameter value (as JSON)
    ///
    /// # Returns
    ///
    /// Job information including the job ID.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeoprocessingServiceClient};
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let gp_service = GeoprocessingServiceClient::new(
    ///     "https://server/arcgis/rest/services/GP/LongTask/GPServer/Execute",
    ///     &client
    /// );
    ///
    /// let mut params = HashMap::new();
    /// params.insert("InputParameter".to_string(), serde_json::json!("value"));
    ///
    /// let job = gp_service.submit_job(params).await?;
    /// tracing::info!(job_id = %job.job_id(), "Job submitted");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, parameters), fields(param_count = parameters.len()))]
    pub async fn submit_job(&self, parameters: HashMap<String, Value>) -> Result<GPJobInfo> {
        tracing::debug!("Submitting asynchronous geoprocessing job");

        let submit_url = format!("{}/submitJob", self.url);

        let mut form: Vec<(&str, String)> = Vec::new();
        form.push(("f", "json".to_string()));

        // Serialize parameters as JSON
        for (key, value) in parameters.iter() {
            let value_str = serde_json::to_string(value)?;
            form.push((key.as_str(), value_str));
        }

        let response = self
            .client
            .http()
            .post(&submit_url)
            .form(&form)
            .send()
            .await?;

        let job_info: GPJobInfo = response.json().await?;

        tracing::info!(
            job_id = %job_info.job_id(),
            status = ?job_info.job_status(),
            "Job submitted"
        );

        Ok(job_info)
    }

    /// Gets the current status of an asynchronous job.
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job identifier returned from [`submit_job`](Self::submit_job)
    ///
    /// # Returns
    ///
    /// Current job information including status.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeoprocessingServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// # let client = ArcGISClient::new(auth);
    /// # let gp_service = GeoprocessingServiceClient::new("https://server/gp", &client);
    /// # let job_id = "job123";
    /// let status = gp_service.get_job_status(job_id).await?;
    /// tracing::info!(status = ?status.job_status(), "Job status");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(job_id))]
    pub async fn get_job_status(&self, job_id: &str) -> Result<GPJobInfo> {
        tracing::debug!("Getting job status");

        let status_url = format!("{}/jobs/{}", self.url, job_id);

        let response = self
            .client
            .http()
            .get(&status_url)
            .query(&[("f", "json")])
            .send()
            .await?;

        let job_info: GPJobInfo = response.json().await?;

        tracing::debug!(status = ?job_info.job_status(), "Job status retrieved");

        Ok(job_info)
    }

    /// Gets the results of a completed asynchronous job.
    ///
    /// The job must be in a completed state (succeeded) before calling this method.
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job identifier
    ///
    /// # Returns
    ///
    /// Job information including results.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeoprocessingServiceClient, GPJobStatus};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// # let client = ArcGISClient::new(auth);
    /// # let gp_service = GeoprocessingServiceClient::new("https://server/gp", &client);
    /// # let job_id = "job123";
    /// let result = gp_service.get_job_result(job_id).await?;
    /// if result.job_status() == &GPJobStatus::Succeeded {
    ///     tracing::info!(result_count = result.results().len(), "Job succeeded");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(job_id))]
    pub async fn get_job_result(&self, job_id: &str) -> Result<GPJobInfo> {
        tracing::debug!("Getting job result");

        // Get status first to ensure job is complete
        let job_info = self.get_job_status(job_id).await?;

        if !job_info.job_status().is_terminal() {
            tracing::warn!(
                status = ?job_info.job_status(),
                "Job is not in terminal state"
            );
        }

        Ok(job_info)
    }

    /// Cancels an asynchronous job.
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job identifier to cancel
    ///
    /// # Returns
    ///
    /// Updated job information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeoprocessingServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// # let client = ArcGISClient::new(auth);
    /// # let gp_service = GeoprocessingServiceClient::new("https://server/gp", &client);
    /// # let job_id = "job123";
    /// let result = gp_service.cancel_job(job_id).await?;
    /// tracing::info!("Job cancelled");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(job_id))]
    pub async fn cancel_job(&self, job_id: &str) -> Result<GPJobInfo> {
        tracing::debug!("Cancelling job");

        let cancel_url = format!("{}/jobs/{}/cancel", self.url, job_id);

        let response = self
            .client
            .http()
            .post(&cancel_url)
            .form(&[("f", "json")])
            .send()
            .await?;

        let job_info: GPJobInfo = response.json().await?;

        tracing::info!(status = ?job_info.job_status(), "Job cancel requested");

        Ok(job_info)
    }

    /// Gets messages for an asynchronous job.
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job identifier
    ///
    /// # Returns
    ///
    /// Vector of messages generated during job execution.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeoprocessingServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// # let client = ArcGISClient::new(auth);
    /// # let gp_service = GeoprocessingServiceClient::new("https://server/gp", &client);
    /// # let job_id = "job123";
    /// let messages = gp_service.get_job_messages(job_id).await?;
    /// for msg in messages.iter() {
    ///     tracing::info!(message_type = ?msg.message_type(), description = %msg.description());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(job_id))]
    pub async fn get_job_messages(&self, job_id: &str) -> Result<Vec<GPMessage>> {
        tracing::debug!("Getting job messages");

        let messages_url = format!("{}/jobs/{}/messages", self.url, job_id);

        #[derive(Deserialize)]
        struct MessagesResponse {
            messages: Vec<GPMessage>,
        }

        let response = self
            .client
            .http()
            .get(&messages_url)
            .query(&[("f", "json")])
            .send()
            .await?;

        let messages_response: MessagesResponse = response.json().await?;

        tracing::debug!(
            message_count = messages_response.messages.len(),
            "Messages retrieved"
        );

        Ok(messages_response.messages)
    }

    /// Polls a job until it reaches a terminal state (succeeded, failed, etc.).
    ///
    /// This is a convenience method that repeatedly checks job status with exponential backoff.
    ///
    /// # Arguments
    ///
    /// * `job_id` - Job identifier
    /// * `initial_delay_ms` - Initial delay between polls in milliseconds (default: 1000)
    /// * `max_delay_ms` - Maximum delay between polls in milliseconds (default: 30000)
    /// * `timeout_ms` - Optional timeout in milliseconds (None for no timeout)
    ///
    /// # Returns
    ///
    /// Final job information when terminal state is reached.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeoprocessingServiceClient};
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let gp_service = GeoprocessingServiceClient::new("https://server/gp", &client);
    ///
    /// let mut params = HashMap::new();
    /// params.insert("Input".to_string(), serde_json::json!("value"));
    ///
    /// let job = gp_service.submit_job(params).await?;
    /// let result = gp_service.poll_until_complete(job.job_id(), 1000, 30000, None).await?;
    /// tracing::info!(status = ?result.job_status(), "Job complete");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(job_id, initial_delay_ms, max_delay_ms, timeout_ms))]
    pub async fn poll_until_complete(
        &self,
        job_id: &str,
        initial_delay_ms: u64,
        max_delay_ms: u64,
        timeout_ms: Option<u64>,
    ) -> Result<GPJobInfo> {
        use tokio::time::{Duration, Instant, sleep};

        tracing::info!("Polling job until complete");

        let start_time = Instant::now();
        let mut delay_ms = initial_delay_ms;

        loop {
            // Check timeout
            if let Some(timeout) = timeout_ms {
                if start_time.elapsed().as_millis() > timeout as u128 {
                    tracing::error!("Job polling timed out");
                    return Err(BuilderError::new(format!(
                        "Job polling timed out after {}ms",
                        timeout
                    ))
                    .into());
                }
            }

            // Get job status
            let job_info = self.get_job_status(job_id).await?;

            tracing::debug!(
                status = ?job_info.job_status(),
                elapsed_ms = start_time.elapsed().as_millis(),
                "Polling job"
            );

            // Check if terminal state
            if job_info.job_status().is_terminal() {
                tracing::info!(
                    status = ?job_info.job_status(),
                    elapsed_ms = start_time.elapsed().as_millis(),
                    "Job reached terminal state"
                );
                return Ok(job_info);
            }

            // Wait before next poll with exponential backoff
            sleep(Duration::from_millis(delay_ms)).await;
            delay_ms = (delay_ms * 2).min(max_delay_ms);
        }
    }
}
