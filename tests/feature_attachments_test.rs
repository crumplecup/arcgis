//! Tests for Feature Service attachment operations.

mod common;

use arcgis::{AttachmentId, AttachmentSource, DownloadTarget};

#[test]
fn test_attachment_source_from_path() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_attachment_source_from_path: Starting");

    tracing::info!("test_attachment_source_from_path: Creating attachment source from path");
    let source = AttachmentSource::from_path("/path/to/file.jpg");
    
    tracing::info!("test_attachment_source_from_path: Verifying source is Path variant");
    assert!(matches!(source, AttachmentSource::Path(_)));
    
    tracing::info!("test_attachment_source_from_path: Completed");
    Ok(())
}

#[test]
fn test_attachment_source_from_bytes() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_attachment_source_from_bytes: Starting");

    tracing::info!("test_attachment_source_from_bytes: Creating attachment source from bytes");
    let source = AttachmentSource::from_bytes("test.txt", vec![1, 2, 3]);
    
    if let AttachmentSource::Bytes {
        filename,
        data,
        content_type,
    } = source
    {
        tracing::info!(
            filename = %filename,
            data_len = data.len(),
            has_content_type = content_type.is_some(),
            "test_attachment_source_from_bytes: Verified Bytes variant"
        );
        assert_eq!(filename, "test.txt");
        assert_eq!(data, vec![1, 2, 3]);
        assert!(content_type.is_none());
    } else {
        anyhow::bail!("Expected Bytes variant");
    }
    
    tracing::info!("test_attachment_source_from_bytes: Completed");
    Ok(())
}

#[test]
fn test_attachment_source_from_bytes_with_type() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_attachment_source_from_bytes_with_type: Starting");

    tracing::info!("test_attachment_source_from_bytes_with_type: Creating attachment with content type");
    let source = AttachmentSource::from_bytes_with_type(
        "test.bin",
        vec![1, 2, 3],
        "application/octet-stream",
    );
    
    if let AttachmentSource::Bytes {
        filename,
        data,
        content_type,
    } = source
    {
        tracing::info!(
            filename = %filename,
            data_len = data.len(),
            content_type = ?content_type,
            "test_attachment_source_from_bytes_with_type: Verified Bytes variant with content type"
        );
        assert_eq!(filename, "test.bin");
        assert_eq!(data, vec![1, 2, 3]);
        assert_eq!(content_type, Some("application/octet-stream".to_string()));
    } else {
        anyhow::bail!("Expected Bytes variant");
    }
    
    tracing::info!("test_attachment_source_from_bytes_with_type: Completed");
    Ok(())
}

#[test]
fn test_download_target_to_path() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_download_target_to_path: Starting");

    tracing::info!("test_download_target_to_path: Creating download target for path");
    let target = DownloadTarget::to_path("/path/to/save/file.jpg");
    
    tracing::info!("test_download_target_to_path: Verifying target is Path variant");
    assert!(matches!(target, DownloadTarget::Path(_)));
    
    tracing::info!("test_download_target_to_path: Completed");
    Ok(())
}

#[test]
fn test_download_target_to_bytes() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_download_target_to_bytes: Starting");

    tracing::info!("test_download_target_to_bytes: Creating download target for bytes");
    let target = DownloadTarget::to_bytes();
    
    tracing::info!("test_download_target_to_bytes: Verifying target is Bytes variant");
    assert!(matches!(target, DownloadTarget::Bytes));
    
    tracing::info!("test_download_target_to_bytes: Completed");
    Ok(())
}

#[test]
fn test_attachment_id_operations() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_attachment_id_operations: Starting");

    tracing::info!("test_attachment_id_operations: Creating attachment IDs");
    let id1 = AttachmentId::new(123);
    let id2: AttachmentId = 123.into();

    tracing::info!(
        id1 = id1.get(),
        id2 = id2.get(),
        "test_attachment_id_operations: Verifying ID equality"
    );
    assert_eq!(id1, id2);
    assert_eq!(id1.get(), 123);
    assert_eq!(id1.to_string(), "123");
    
    tracing::info!("test_attachment_id_operations: Completed");
    Ok(())
}

// API tests are feature-gated and should only run when explicitly requested
// via `cargo test --features api` to avoid hitting live endpoints during CI/CD

#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_query_attachments_api() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_query_attachments_api: Starting");

    // This test requires:
    // - ARCGIS_API_KEY environment variable
    // - ARCGIS_TEST_SERVICE_URL environment variable pointing to a service with attachments

    tracing::info!("test_query_attachments_api: API test not yet implemented");
    
    // Example implementation (commented out to avoid accidental API calls):
    // let api_key = std::env::var("ARCGIS_API_KEY")?;
    // let service_url = std::env::var("ARCGIS_TEST_SERVICE_URL")?;
    //
    // let auth = arcgis::ApiKeyAuth::new(&api_key);
    // let client = arcgis::ArcGISClient::new(auth);
    // let service = arcgis::FeatureServiceClient::new(&service_url, &client);
    //
    // let attachments = service
    //     .query_attachments(arcgis::LayerId::new(0), arcgis::ObjectId::new(1))
    //     .await?;
    //
    // tracing::info!(count = attachments.len(), "Found attachments");

    tracing::info!("test_query_attachments_api: Completed");
    Ok(())
}
