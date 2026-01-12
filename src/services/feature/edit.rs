//! Feature editing operations (add, update, delete).
//!
//! This module provides type-safe interfaces to ArcGIS feature editing operations.
//! All edit operations support transaction semantics via `rollbackOnFailure`.
//!
//! # Operations
//!
//! - [`add_features`](FeatureServiceClient::add_features) - Add new features
//! - [`update_features`](FeatureServiceClient::update_features) - Update existing features
//! - [`delete_features`](FeatureServiceClient::delete_features) - Delete features
//! - [`apply_edits`](FeatureServiceClient::apply_edits) - Batch operation (add + update + delete)
//!
//! # Example
//!
//! ```no_run
//! use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, Feature, EditOptions};
//! use serde_json::json;
//! use std::collections::HashMap;
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//! let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
//!
//! // Add a new feature
//! let mut attributes = HashMap::new();
//! attributes.insert("NAME".to_string(), json!("New City"));
//! attributes.insert("POPULATION".to_string(), json!(50000));
//!
//! let new_feature = Feature::new(attributes, None);
//!
//! let result = service
//!     .add_features(LayerId::new(0), vec![new_feature], EditOptions::default())
//!     .await?;
//!
//! for item in result.add_results() {
//!     if *item.success() {
//!         println!("Added feature with ObjectID: {}", item.object_id().expect("Has ID"));
//!     } else {
//!         eprintln!("Failed: {:?}", item.error());
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::{ObjectId, SessionId};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// Result of an edit operation (add, update, or delete).
///
/// Contains arrays of results for each type of edit performed.
/// When using individual operations (addFeatures, updateFeatures, deleteFeatures),
/// only the corresponding result array will be populated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct EditResult {
    /// Results from adding features
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    add_results: Vec<EditResultItem>,

    /// Results from updating features
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    update_results: Vec<EditResultItem>,

    /// Results from deleting features
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    delete_results: Vec<EditResultItem>,
}

impl EditResult {
    /// Returns true if all edit operations succeeded.
    pub fn all_succeeded(&self) -> bool {
        let all_adds = self.add_results.iter().all(|r| r.success);
        let all_updates = self.update_results.iter().all(|r| r.success);
        let all_deletes = self.delete_results.iter().all(|r| r.success);

        all_adds && all_updates && all_deletes
    }

    /// Returns the total number of successful edits across all operations.
    pub fn success_count(&self) -> usize {
        self.add_results.iter().filter(|r| r.success).count()
            + self.update_results.iter().filter(|r| r.success).count()
            + self.delete_results.iter().filter(|r| r.success).count()
    }

    /// Returns the total number of failed edits across all operations.
    pub fn failure_count(&self) -> usize {
        self.add_results.iter().filter(|r| !r.success).count()
            + self.update_results.iter().filter(|r| !r.success).count()
            + self.delete_results.iter().filter(|r| !r.success).count()
    }
}

/// Individual result for a single feature edit.
///
/// Returned as part of an [`EditResult`] to indicate success or failure
/// for each feature in an edit operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct EditResultItem {
    /// The ObjectID of the feature (for add operations, this is the newly assigned ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    object_id: Option<ObjectId>,

    /// The GlobalID of the feature (if GlobalIDs are enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    global_id: Option<String>,

    /// Whether the operation succeeded
    success: bool,

    /// Error details if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<EditError>,
}

/// Error details for a failed edit operation.
///
/// Includes field-level validation errors when available, making it easier
/// to identify which fields caused the failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct EditError {
    /// Error code
    code: i32,

    /// Human-readable error description
    description: String,

    /// The field name that caused the error (if field-specific).
    ///
    /// Present when the error is related to a specific field validation failure.
    #[serde(rename = "fieldName", skip_serializing_if = "Option::is_none")]
    field_name: Option<String>,

    /// The invalid value(s) that caused the error.
    ///
    /// Contains the actual values that failed validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<serde_json::Value>>,

    /// Additional error details.
    ///
    /// May contain more specific information about the validation failure,
    /// such as constraint violations or data type mismatches.
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Vec<String>>,
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref field_name) = self.field_name {
            write!(
                f,
                "Edit error {} (field: {}): {}",
                self.code, field_name, self.description
            )
        } else {
            write!(f, "Edit error {}: {}", self.code, self.description)
        }
    }
}

/// Options for controlling edit behavior.
///
/// These options apply to all edit operations (add, update, delete, and batch).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditOptions {
    /// Geodatabase version to target (for versioned data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gdb_version: Option<String>,

    /// If true, all edits are applied only if all succeed.
    /// If false, successful edits are applied even if some fail.
    /// Default: true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback_on_failure: Option<bool>,

    /// Use GlobalIDs instead of ObjectIDs for identification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_global_ids: Option<bool>,

    /// Return detailed edit results (default: true)
    /// When false with rollbackOnFailure=true, returns simple {success: true/false}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_edit_results: Option<bool>,

    /// Edit session ID for versioned editing workflows
    ///
    /// Required when editing branch-versioned geodatabases. Obtain by calling
    /// [`VersionManagementClient::start_editing`](crate::VersionManagementClient::start_editing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
}

impl Default for EditOptions {
    fn default() -> Self {
        Self {
            gdb_version: None,
            rollback_on_failure: Some(true),
            use_global_ids: None,
            return_edit_results: Some(true),
            session_id: None,
        }
    }
}

impl EditOptions {
    /// Creates EditOptions with all fields set to their defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the geodatabase version.
    pub fn with_gdb_version(mut self, version: impl Into<String>) -> Self {
        self.gdb_version = Some(version.into());
        self
    }

    /// Sets the rollback behavior.
    ///
    /// When `true`, all edits are applied only if all succeed (atomic transaction).
    /// When `false`, successful edits apply even if some fail.
    pub fn with_rollback_on_failure(mut self, rollback: bool) -> Self {
        self.rollback_on_failure = Some(rollback);
        self
    }

    /// Use GlobalIDs instead of ObjectIDs.
    pub fn with_use_global_ids(mut self, use_global_ids: bool) -> Self {
        self.use_global_ids = Some(use_global_ids);
        self
    }

    /// Control whether detailed results are returned.
    pub fn with_return_edit_results(mut self, return_results: bool) -> Self {
        self.return_edit_results = Some(return_results);
        self
    }

    /// Sets the edit session ID for versioned editing.
    ///
    /// Required when editing branch-versioned geodatabases. The session ID is
    /// obtained by calling
    /// [`VersionManagementClient::start_editing`](crate::VersionManagementClient::start_editing).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{EditOptions, SessionId};
    ///
    /// let session_id = SessionId::new();
    /// let options = EditOptions::new().with_session_id(session_id);
    /// ```
    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }
}
