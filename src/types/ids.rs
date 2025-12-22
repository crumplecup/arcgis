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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn test_layer_id_creation() {
        let id = LayerId::new(42);
        assert_eq!(id.get(), 42);
        assert_eq!(id.to_string(), "42");
    }

    #[test]
    fn test_layer_id_from_u32() {
        let id: LayerId = 42.into();
        assert_eq!(id.get(), 42);
    }

    #[test]
    fn test_object_id_serialization() -> Result<()> {
        let id = ObjectId::new(123);
        let json = serde_json::to_string(&id)?;
        assert_eq!(json, "123");
        Ok(())
    }

    #[test]
    fn test_object_id_deserialization() -> Result<()> {
        let json = "456";
        let id: ObjectId = serde_json::from_str(json)?;
        assert_eq!(id.get(), 456);
        Ok(())
    }
}
