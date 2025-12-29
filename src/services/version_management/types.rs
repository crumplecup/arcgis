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
