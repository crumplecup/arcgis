//! Tests for ID types (LayerId, ObjectId, AttachmentId).

mod common;

use arcgis::{AttachmentId, LayerId, ObjectId};

#[test]
fn test_layer_id_creation() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_layer_id_creation: Starting");

    tracing::info!("test_layer_id_creation: Creating LayerId");
    let id = LayerId::new(42);

    tracing::info!(
        id = id.get(),
        id_string = %id.to_string(),
        "test_layer_id_creation: Verifying LayerId"
    );
    assert_eq!(id.get(), 42);
    assert_eq!(id.to_string(), "42");

    tracing::info!("test_layer_id_creation: Completed");
    Ok(())
}

#[test]
fn test_layer_id_from_u32() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_layer_id_from_u32: Starting");

    tracing::info!("test_layer_id_from_u32: Converting u32 to LayerId");
    let id: LayerId = 42.into();

    tracing::info!(
        id = id.get(),
        "test_layer_id_from_u32: Verifying conversion"
    );
    assert_eq!(id.get(), 42);

    tracing::info!("test_layer_id_from_u32: Completed");
    Ok(())
}

#[test]
fn test_object_id_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_object_id_serialization: Starting");

    tracing::info!("test_object_id_serialization: Creating ObjectId");
    let id = ObjectId::new(123);

    tracing::info!("test_object_id_serialization: Serializing to JSON");
    let json = serde_json::to_string(&id)?;

    tracing::info!(
        json = %json,
        "test_object_id_serialization: Verifying serialization"
    );
    assert_eq!(json, "123");

    tracing::info!("test_object_id_serialization: Completed");
    Ok(())
}

#[test]
fn test_object_id_deserialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_object_id_deserialization: Starting");

    let json = "456";

    tracing::info!("test_object_id_deserialization: Deserializing from JSON");
    let id: ObjectId = serde_json::from_str(json)?;

    tracing::info!(
        id = id.get(),
        "test_object_id_deserialization: Verifying deserialization"
    );
    assert_eq!(id.get(), 456);

    tracing::info!("test_object_id_deserialization: Completed");
    Ok(())
}

#[test]
fn test_attachment_id_creation() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_attachment_id_creation: Starting");

    tracing::info!("test_attachment_id_creation: Creating AttachmentId");
    let id = AttachmentId::new(789);

    tracing::info!(
        id = id.get(),
        id_string = %id.to_string(),
        "test_attachment_id_creation: Verifying AttachmentId"
    );
    assert_eq!(id.get(), 789);
    assert_eq!(id.to_string(), "789");

    tracing::info!("test_attachment_id_creation: Completed");
    Ok(())
}

#[test]
fn test_attachment_id_serialization() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_attachment_id_serialization: Starting");

    tracing::info!("test_attachment_id_serialization: Creating AttachmentId");
    let id = AttachmentId::new(101);

    tracing::info!("test_attachment_id_serialization: Serializing to JSON");
    let json = serde_json::to_string(&id)?;

    tracing::info!(
        json = %json,
        "test_attachment_id_serialization: Verifying serialization"
    );
    assert_eq!(json, "101");

    tracing::info!("test_attachment_id_serialization: Completed");
    Ok(())
}

#[test]
fn test_attachment_id_from_u32() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_attachment_id_from_u32: Starting");

    tracing::info!("test_attachment_id_from_u32: Converting u32 to AttachmentId");
    let id: AttachmentId = 55.into();

    tracing::info!(
        id = id.get(),
        "test_attachment_id_from_u32: Verifying conversion"
    );
    assert_eq!(id.get(), 55);

    tracing::info!("test_attachment_id_from_u32: Completed");
    Ok(())
}
