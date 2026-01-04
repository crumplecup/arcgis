//! Tests for Feature Service attachment operations.

use arcgis::{AttachmentId, AttachmentSource, DownloadTarget};

#[test]
fn test_attachment_source_from_path() {
    let source = AttachmentSource::from_path("/path/to/file.jpg");
    matches!(source, AttachmentSource::Path(_));
}

#[test]
fn test_attachment_source_from_bytes() {
    let source = AttachmentSource::from_bytes("test.txt", vec![1, 2, 3]);
    if let AttachmentSource::Bytes {
        filename,
        data,
        content_type,
    } = source
    {
        assert_eq!(filename, "test.txt");
        assert_eq!(data, vec![1, 2, 3]);
        assert!(content_type.is_none());
    } else {
        panic!("Expected Bytes variant");
    }
}

#[test]
fn test_attachment_source_from_bytes_with_type() {
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
        assert_eq!(filename, "test.bin");
        assert_eq!(data, vec![1, 2, 3]);
        assert_eq!(content_type, Some("application/octet-stream".to_string()));
    } else {
        panic!("Expected Bytes variant");
    }
}

#[test]
fn test_download_target_to_path() {
    let target = DownloadTarget::to_path("/path/to/save/file.jpg");
    matches!(target, DownloadTarget::Path(_));
}

#[test]
fn test_download_target_to_bytes() {
    let target = DownloadTarget::to_bytes();
    matches!(target, DownloadTarget::Bytes);
}

#[test]
fn test_attachment_id_operations() {
    let id1 = AttachmentId::new(123);
    let id2: AttachmentId = 123.into();

    assert_eq!(id1, id2);
    assert_eq!(id1.get(), 123);
    assert_eq!(id1.to_string(), "123");
}

// API tests are feature-gated and should only run when explicitly requested
// via `cargo test --features api` to avoid hitting live endpoints during CI/CD

#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_query_attachments_api() {
    // This test requires:
    // - ARCGIS_API_KEY environment variable
    // - ARCGIS_TEST_SERVICE_URL environment variable pointing to a service with attachments

    // Example implementation (commented out to avoid accidental API calls):
    // let api_key = std::env::var("ARCGIS_API_KEY").expect("ARCGIS_API_KEY not set");
    // let service_url = std::env::var("ARCGIS_TEST_SERVICE_URL")
    //     .expect("ARCGIS_TEST_SERVICE_URL not set");
    //
    // let auth = arcgis::ApiKeyAuth::new(&api_key);
    // let client = arcgis::ArcGISClient::new(auth);
    // let service = arcgis::FeatureServiceClient::new(&service_url, &client);
    //
    // let attachments = service
    //     .query_attachments(arcgis::LayerId::new(0), arcgis::ObjectId::new(1))
    //     .await
    //     .expect("Query attachments failed");
    //
    // println!("Found {} attachments", attachments.len());
}
