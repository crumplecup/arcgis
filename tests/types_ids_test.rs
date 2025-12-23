//! Tests for ID types (LayerId, ObjectId).

use arcgis::{LayerId, ObjectId, Result};

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
