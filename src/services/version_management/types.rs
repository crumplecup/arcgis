//! Version Management Service types.

use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A session identifier for edit operations.
///
/// Edit sessions track multi-request editing workflows in branch-versioned
/// geodatabases. Each session is identified by a unique GUID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Creates a new random session ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a session ID from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SessionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<SessionId> for Uuid {
    fn from(id: SessionId) -> Self {
        id.0
    }
}

/// A version identifier (GUID).
///
/// Versions in ArcGIS geodatabases are identified by GUIDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionGuid(Uuid);

impl VersionGuid {
    /// Creates a version GUID from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl fmt::Display for VersionGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for VersionGuid {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<VersionGuid> for Uuid {
    fn from(guid: VersionGuid) -> Self {
        guid.0
    }
}

/// Response from startEditing operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct StartEditingResponse {
    /// Whether the operation succeeded
    success: bool,

    /// Moment (timestamp) when the edit session started
    #[serde(skip_serializing_if = "Option::is_none")]
    moment: Option<String>,

    /// Error information if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<EditSessionError>,
}

/// Response from stopEditing operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct StopEditingResponse {
    /// Whether the operation succeeded
    success: bool,

    /// Moment (timestamp) when the edit session stopped
    #[serde(skip_serializing_if = "Option::is_none")]
    moment: Option<String>,

    /// Error information if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<EditSessionError>,
}

/// Response from startReading operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct StartReadingResponse {
    /// Whether the operation succeeded
    success: bool,

    /// Moment (timestamp) when the read session started
    #[serde(skip_serializing_if = "Option::is_none")]
    moment: Option<String>,

    /// Error information if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<EditSessionError>,
}

/// Response from stopReading operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct StopReadingResponse {
    /// Whether the operation succeeded
    success: bool,

    /// Moment (timestamp) when the read session stopped
    #[serde(skip_serializing_if = "Option::is_none")]
    moment: Option<String>,

    /// Error information if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<EditSessionError>,
}

/// Error information from edit session operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct EditSessionError {
    /// Error code
    code: i32,

    /// Error message
    message: String,

    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Vec<String>>,
}

/// Versioning type for a geodatabase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersioningType {
    /// Branch versioning (modern, lightweight)
    Branch,
    /// Traditional versioning (delta tables)
    Traditional,
}

impl fmt::Display for VersioningType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Branch => write!(f, "branch"),
            Self::Traditional => write!(f, "traditional"),
        }
    }
}

/// Information about a version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    /// Version GUID
    version_guid: String,

    /// Version name
    version_name: String,

    /// Version description
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    /// Access level (public, protected, private)
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<String>,

    /// Created date
    #[serde(skip_serializing_if = "Option::is_none")]
    created_date: Option<String>,

    /// Modified date
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_date: Option<String>,
}

/// Access permission level for a version.
///
/// Controls who can view and edit a version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionPermission {
    /// Anyone can view and edit (most permissive)
    Public,
    /// Only the owner and administrators can edit, others can view
    Protected,
    /// Only the owner can view and edit (most restrictive)
    Private,
}

impl fmt::Display for VersionPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Protected => write!(f, "protected"),
            Self::Private => write!(f, "private"),
        }
    }
}

/// Parameters for creating a new version.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVersionParams {
    /// Name of the version (without owner prefix)
    pub version_name: String,

    /// Access permission level
    pub access: VersionPermission,

    /// Description of the version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CreateVersionParams {
    /// Creates parameters for a new version.
    ///
    /// # Arguments
    ///
    /// * `version_name` - Name of the version (e.g., "workplan_2024")
    /// * `access` - Permission level (Public, Protected, or Private)
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::{CreateVersionParams, VersionPermission};
    ///
    /// let params = CreateVersionParams::new("workplan_2024", VersionPermission::Public)
    ///     .with_description("Work plan for 2024 projects");
    /// ```
    pub fn new(version_name: impl Into<String>, access: VersionPermission) -> Self {
        Self {
            version_name: version_name.into(),
            access,
            description: None,
        }
    }

    /// Sets the description for the version.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Parameters for altering an existing version's properties.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlterVersionParams {
    /// New version name (if renaming)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_name: Option<String>,

    /// New access permission level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<VersionPermission>,

    /// New description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl AlterVersionParams {
    /// Creates empty alter parameters.
    ///
    /// Use builder methods to set properties to change.
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::{AlterVersionParams, VersionPermission};
    ///
    /// let params = AlterVersionParams::new()
    ///     .with_access(VersionPermission::Protected)
    ///     .with_description("Updated description");
    /// ```
    pub fn new() -> Self {
        Self {
            version_name: None,
            access: None,
            description: None,
        }
    }

    /// Sets a new name for the version.
    pub fn with_version_name(mut self, version_name: impl Into<String>) -> Self {
        self.version_name = Some(version_name.into());
        self
    }

    /// Sets a new access permission level.
    pub fn with_access(mut self, access: VersionPermission) -> Self {
        self.access = Some(access);
        self
    }

    /// Sets a new description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl Default for AlterVersionParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from create operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct CreateVersionResponse {
    /// Whether the operation succeeded
    success: bool,

    /// The created version's information
    #[serde(skip_serializing_if = "Option::is_none")]
    version_info: Option<VersionInfo>,

    /// Error information if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<EditSessionError>,
}

/// Response from alter operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct AlterResponse {
    /// Whether the operation succeeded
    success: bool,

    /// Moment (timestamp) when the alteration occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    moment: Option<String>,

    /// Error information if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<EditSessionError>,
}

/// Response from delete operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct DeleteResponse {
    /// Whether the operation succeeded
    success: bool,

    /// Moment (timestamp) when the deletion occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    moment: Option<String>,

    /// Error information if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<EditSessionError>,
}

/// Response from versionInfos operation (list all versions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct VersionInfosResponse {
    /// List of versions
    versions: Vec<VersionInfo>,
}

/// Conflict detection type for reconcile operations.
///
/// Determines how conflicts are detected when reconciling versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ConflictDetection {
    /// Conflicts detected at object level (default)
    #[default]
    ByObject,
    /// Conflicts detected at attribute level (more granular)
    ByAttribute,
}

impl fmt::Display for ConflictDetection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ByObject => write!(f, "byObject"),
            Self::ByAttribute => write!(f, "byAttribute"),
        }
    }
}

/// Response from reconcile operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct ReconcileResponse {
    /// Whether the operation succeeded
    success: bool,

    /// Whether conflicts were detected during reconcile
    #[serde(skip_serializing_if = "Option::is_none")]
    has_conflicts: Option<bool>,

    /// Moment (timestamp) when the reconcile occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    moment: Option<String>,

    /// Whether the post operation was performed (if withPost=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    did_post: Option<bool>,

    /// Error information if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<EditSessionError>,
}

/// Response from post operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct PostResponse {
    /// Whether the operation succeeded
    success: bool,

    /// Moment (timestamp) when the post occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    moment: Option<String>,

    /// Error information if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<EditSessionError>,
}

/// Specifies a subset of edits to post for partial post operations.
///
/// Each row identifies a layer and the specific object IDs within that layer
/// to post to the parent version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialPostRow {
    /// The layer ID containing the objects to post
    pub layer_id: i32,

    /// The object IDs to post from this layer
    pub object_ids: Vec<i64>,
}

impl PartialPostRow {
    /// Creates a new partial post row specification.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer ID
    /// * `object_ids` - Vector of object IDs to post
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::PartialPostRow;
    ///
    /// let row = PartialPostRow::new(0, vec![1, 2, 3]);
    /// assert_eq!(row.layer_id, 0);
    /// assert_eq!(row.object_ids, vec![1, 2, 3]);
    /// ```
    pub fn new(layer_id: i32, object_ids: Vec<i64>) -> Self {
        Self {
            layer_id,
            object_ids,
        }
    }
}
