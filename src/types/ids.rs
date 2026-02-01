//! Newtype wrappers for various ID types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Layer ID within a feature or map service.
///
/// This newtype prevents mixing layer IDs with other numeric values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LayerId(pub u32);

impl LayerId {
    /// Creates a new layer ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the inner ID value.
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for LayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for LayerId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// Object ID for a feature.
///
/// This is the unique identifier for features within a layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObjectId(pub u32);

impl ObjectId {
    /// Creates a new object ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the inner ID value.
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for ObjectId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// Attachment ID for a feature attachment.
///
/// This is the unique identifier for attachments within a feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttachmentId(pub u32);

impl AttachmentId {
    /// Creates a new attachment ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the inner ID value.
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for AttachmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for AttachmentId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}
