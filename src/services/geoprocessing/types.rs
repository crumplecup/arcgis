//! Geoprocessing result types and enums.

use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result from a geoprocessing execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct GPExecuteResult {
    /// Output parameters from the GP task.
    results: Vec<GPResultParameter>,

    /// Messages generated during execution.
    messages: Vec<GPMessage>,
}

/// A geoprocessing result parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct GPResultParameter {
    /// Parameter name.
    #[serde(default)]
    param_name: Option<String>,

    /// Data type of the parameter.
    #[serde(default)]
    data_type: Option<String>,

    /// Parameter value (type varies by dataType).
    #[serde(default)]
    value: Option<serde_json::Value>,

    /// URL to fetch parameter value (for async jobs).
    #[serde(default)]
    param_url: Option<String>,
}

/// A message from geoprocessing execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct GPMessage {
    /// Message type (informative, warning, error, etc.).
    #[serde(rename = "type")]
    message_type: GPMessageType,

    /// Message description.
    description: String,
}

/// Type of geoprocessing message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GPMessageType {
    /// Informative message.
    #[serde(rename = "esriJobMessageTypeInformative")]
    Informative,

    /// Warning message.
    #[serde(rename = "esriJobMessageTypeWarning")]
    Warning,

    /// Error message.
    #[serde(rename = "esriJobMessageTypeError")]
    Error,

    /// Empty message.
    #[serde(rename = "esriJobMessageTypeEmpty")]
    Empty,

    /// Abort message.
    #[serde(rename = "esriJobMessageTypeAbort")]
    Abort,
}

/// Status of an asynchronous geoprocessing job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GPJobStatus {
    /// Job has been created but not yet submitted.
    #[serde(rename = "esriJobNew")]
    New,

    /// Job has been submitted but not yet started.
    #[serde(rename = "esriJobSubmitted")]
    Submitted,

    /// Job is being submitted.
    #[serde(rename = "esriJobSubmitting")]
    Submitting,

    /// Job is waiting in the queue.
    #[serde(rename = "esriJobWaiting")]
    Waiting,

    /// Job is currently executing.
    #[serde(rename = "esriJobExecuting")]
    Executing,

    /// Job completed successfully.
    #[serde(rename = "esriJobSucceeded")]
    Succeeded,

    /// Job failed.
    #[serde(rename = "esriJobFailed")]
    Failed,

    /// Job timed out.
    #[serde(rename = "esriJobTimedOut")]
    TimedOut,

    /// Job was cancelled.
    #[serde(rename = "esriJobCancelling")]
    Cancelling,

    /// Job has been cancelled.
    #[serde(rename = "esriJobCancelled")]
    Cancelled,

    /// Job is being deleted.
    #[serde(rename = "esriJobDeleting")]
    Deleting,

    /// Job has been deleted.
    #[serde(rename = "esriJobDeleted")]
    Deleted,
}

impl GPJobStatus {
    /// Returns true if the job is in a terminal state (succeeded, failed, cancelled, etc.).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            GPJobStatus::Succeeded
                | GPJobStatus::Failed
                | GPJobStatus::TimedOut
                | GPJobStatus::Cancelled
                | GPJobStatus::Deleted
        )
    }

    /// Returns true if the job is still running.
    pub fn is_running(&self) -> bool {
        matches!(
            self,
            GPJobStatus::New
                | GPJobStatus::Submitted
                | GPJobStatus::Submitting
                | GPJobStatus::Waiting
                | GPJobStatus::Executing
        )
    }
}

/// Progress information for a running geoprocessing job.
///
/// Available in ArcGIS Server 10.8.1+.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct GPProgress {
    /// Type of progress indicator ("default" or "step").
    #[serde(rename = "type")]
    progress_type: String,

    /// Progress message.
    message: String,

    /// Completion percentage (only for "step" type).
    #[serde(default)]
    percent: Option<f64>,
}

/// Information about an asynchronous geoprocessing job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct GPJobInfo {
    /// Job identifier.
    job_id: String,

    /// Current job status.
    job_status: GPJobStatus,

    /// Messages generated during job execution.
    #[serde(default)]
    messages: Vec<GPMessage>,

    /// Results (only present when job succeeds).
    #[serde(default)]
    results: HashMap<String, GPResultParameter>,

    /// Input parameters submitted with the job.
    #[serde(default)]
    inputs: HashMap<String, GPResultParameter>,

    /// Progress information (available in ArcGIS Server 10.8.1+).
    #[serde(default)]
    progress: Option<GPProgress>,
}
