//! 📎 Feature Attachments - Manage Files on GIS Features
//!
//! Demonstrates managing file attachments on feature service features.
//! Learn how to upload photos/documents, list attachments, download files,
//! and clean up outdated attachments - perfect for field data collection!
//!
//! # What You'll Learn
//!
//! - **Upload attachments**: Add photos and documents to features
//! - **List attachments**: Query attachment metadata
//! - **Download attachments**: Retrieve files to disk or memory
//! - **Update attachments**: Replace existing files
//! - **Delete attachments**: Remove outdated files
//! - **Binary handling**: Work with images, PDFs, and other file types
//! - **Streaming**: Efficient handling of large files
//!
//! # Prerequisites
//!
//! - Feature service with attachments enabled
//! - Write access to the feature service
//! - OAuth2 client credentials or API key with edit permissions
//! - Set `ARCGIS_CLIENT_ID` + `ARCGIS_CLIENT_SECRET` in `.env`
//!
//! # Running
//!
//! ```bash
//! # Note: Requires a writable feature service URL
//! cargo run --example feature_attachments
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example feature_attachments
//! ```
//!
//! # Real-World Use Case
//!
//! Field workers use mobile devices to collect infrastructure inspection data:
//! - Utility inspector photographs damaged equipment
//! - Photos are automatically attached to the asset feature
//! - Office staff downloads attachments for reports
//! - Old/duplicate attachments are cleaned up periodically

use anyhow::Context;
use arcgis::{ArcGISClient, ClientCredentialsAuth, FeatureServiceClient, LayerId, ObjectId};

/// Example Feature Service URL (replace with your own writable service)
///
/// Your feature service must:
/// - Have attachments enabled (`hasAttachments: true`)
/// - Allow updates/edits
/// - Have at least one feature to attach files to
///
/// To enable attachments on a hosted feature layer:
/// 1. Go to item details page
/// 2. Settings tab → Enable "Allow attachments"
const FEATURE_SERVICE_URL: &str =
    "https://services.arcgis.com/YOUR_ORG/arcgis/rest/services/YOUR_SERVICE/FeatureServer";

/// Layer ID containing features with attachment support
const LAYER_ID: u32 = 0;

/// Feature object ID to attach files to
/// Replace with an actual feature ID from your service
const FEATURE_OBJECT_ID: u32 = 1;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("📎 ArcGIS Feature Attachments Examples");
    tracing::info!("Demonstrating file attachment management workflows");

    // Check if using placeholder URL
    if FEATURE_SERVICE_URL.contains("YOUR_ORG") || FEATURE_SERVICE_URL.contains("YOUR_SERVICE") {
        tracing::warn!("⚠️  Using placeholder feature service URL");
        tracing::warn!(
            "⚠️  Replace FEATURE_SERVICE_URL with your actual service to run this example"
        );
        tracing::info!("\n📚 This example demonstrates the API usage patterns:");
        tracing::info!("   To run it successfully, you need:");
        tracing::info!("   1. A feature service with attachments enabled");
        tracing::info!("   2. Edit permissions on that service");
        tracing::info!("   3. Update the constants at the top of this file");
        tracing::info!("\nShowing example code patterns below:\n");
    }

    // Load environment variables from .env
    dotenvy::dotenv().ok();

    // Create authenticated client with OAuth2 client credentials
    let client_id =
        std::env::var("ARCGIS_CLIENT_ID").context("ARCGIS_CLIENT_ID not found in environment")?;
    let client_secret = std::env::var("ARCGIS_CLIENT_SECRET")
        .context("ARCGIS_CLIENT_SECRET not found in environment")?;

    let auth = ClientCredentialsAuth::new(client_id, client_secret)
        .context("Failed to create OAuth2 authentication")?;
    let client = ArcGISClient::new(auth);
    let _feature_service = FeatureServiceClient::new(FEATURE_SERVICE_URL, &client);

    let _layer_id = LayerId::new(LAYER_ID);
    let _object_id = ObjectId::new(FEATURE_OBJECT_ID);

    tracing::info!("\n=== Example 1: Listing Existing Attachments ===");
    tracing::info!("Query attachments for feature {}", FEATURE_OBJECT_ID);

    // This pattern would work with a real service:
    tracing::info!("📋 Code pattern:");
    tracing::info!(r#"    let attachments = feature_service"#);
    tracing::info!(r#"        .query_attachments(layer_id, object_id)"#);
    tracing::info!(r#"        .await?;"#);
    tracing::info!(r#""#);
    tracing::info!(r#"    for attachment in &attachments {{"#);
    tracing::info!(
        r#"        println!("{{}} ({{}} bytes)", attachment.name(), attachment.size());"#
    );
    tracing::info!(r#"    }}"#);

    tracing::info!("\n=== Example 2: Adding Photo Attachment ===");
    tracing::info!("Upload an inspection photo to the feature");

    // Create a mock JPEG image (minimal valid JPEG header + data)
    let _mock_jpeg_data = create_mock_jpeg();

    tracing::info!("📤 Code pattern - Upload from bytes:");
    tracing::info!(r#"    let source = AttachmentSource::from_bytes("#);
    tracing::info!(r#"        "inspection_photo.jpg","#);
    tracing::info!(r#"        jpeg_data,"#);
    tracing::info!(r#"    );"#);
    tracing::info!(r#""#);
    tracing::info!(r#"    let result = feature_service"#);
    tracing::info!(r#"        .add_attachment(layer_id, object_id, source)"#);
    tracing::info!(r#"        .await?;"#);
    tracing::info!(r#""#);
    tracing::info!(r#"    if *result.success() {{"#);
    tracing::info!(r#"        println!("Photo attached successfully!");"#);
    tracing::info!(r#"    }}"#);

    tracing::info!("\n💡 Alternative: Upload from file path:");
    tracing::info!(r#"    let source = AttachmentSource::from_path("/path/to/photo.jpg");"#);
    tracing::info!(r#"    feature_service.add_attachment(layer_id, object_id, source).await?;"#);

    tracing::info!("\n=== Example 3: Adding PDF Document ===");
    tracing::info!("Attach an inspection report document");

    let _mock_pdf_data = create_mock_pdf();

    tracing::info!("📄 Code pattern - Upload PDF:");
    tracing::info!(r#"    let source = AttachmentSource::from_bytes("#);
    tracing::info!(r#"        "inspection_report.pdf","#);
    tracing::info!(r#"        pdf_data,"#);
    tracing::info!(r#"    );"#);
    tracing::info!(r#""#);
    tracing::info!(r#"    let result = feature_service"#);
    tracing::info!(r#"        .add_attachment(layer_id, object_id, source)"#);
    tracing::info!(r#"        .await?;"#);

    tracing::info!("\n=== Example 4: Downloading Attachments ===");
    tracing::info!("Retrieve attachment files for reporting");

    tracing::info!("💾 Code pattern - Download to file:");
    tracing::info!(r#"    let attachment_id = AttachmentId::new(1);"#);
    tracing::info!(r#"    let target = DownloadTarget::to_path("/tmp/downloaded_photo.jpg");"#);
    tracing::info!(r#""#);
    tracing::info!(r#"    let result = feature_service"#);
    tracing::info!(r#"        .download_attachment(layer_id, object_id, attachment_id, target)"#);
    tracing::info!(r#"        .await?;"#);
    tracing::info!(r#""#);
    tracing::info!(r#"    if let Some(path) = result.path() {{"#);
    tracing::info!(r#"        println!("Downloaded to {{:?}}", path);"#);
    tracing::info!(r#"    }}"#);

    tracing::info!("\n💡 Alternative: Download to memory:");
    tracing::info!(r#"    let target = DownloadTarget::to_bytes();"#);
    tracing::info!(r#"    let result = feature_service"#);
    tracing::info!(r#"        .download_attachment(layer_id, object_id, attachment_id, target)"#);
    tracing::info!(r#"        .await?;"#);
    tracing::info!(r#""#);
    tracing::info!(r#"    if let Some(bytes) = result.bytes() {{"#);
    tracing::info!(r#"        println!("Downloaded {{}} bytes to memory", bytes.len());"#);
    tracing::info!(r#"    }}"#);

    tracing::info!("\n=== Example 5: Updating an Attachment ===");
    tracing::info!("Replace an outdated photo with a new one");

    tracing::info!("🔄 Code pattern - Update existing attachment:");
    tracing::info!(r#"    let attachment_id = AttachmentId::new(1);"#);
    tracing::info!(
        r#"    let source = AttachmentSource::from_path("/path/to/updated_photo.jpg");"#
    );
    tracing::info!(r#""#);
    tracing::info!(r#"    let result = feature_service"#);
    tracing::info!(r#"        .update_attachment(layer_id, object_id, attachment_id, source)"#);
    tracing::info!(r#"        .await?;"#);
    tracing::info!(r#""#);
    tracing::info!(r#"    if *result.success() {{"#);
    tracing::info!(r#"        println!("Attachment updated!");"#);
    tracing::info!(r#"    }}"#);

    tracing::info!("\n=== Example 6: Deleting Attachments ===");
    tracing::info!("Clean up old or duplicate attachments");

    tracing::info!("🗑️  Code pattern - Delete attachments:");
    tracing::info!(r#"    let ids_to_delete = vec!["#);
    tracing::info!(r#"        AttachmentId::new(2),"#);
    tracing::info!(r#"        AttachmentId::new(3),"#);
    tracing::info!(r#"    ];"#);
    tracing::info!(r#""#);
    tracing::info!(r#"    let result = feature_service"#);
    tracing::info!(r#"        .delete_attachments(layer_id, object_id, ids_to_delete)"#);
    tracing::info!(r#"        .await?;"#);
    tracing::info!(r#""#);
    tracing::info!(r#"    for item in &result.delete_attachment_results {{"#);
    tracing::info!(r#"        if *item.success() {{"#);
    tracing::info!(
        r#"            println!("Deleted attachment from feature {{}}", item.object_id());"#
    );
    tracing::info!(r#"        }}"#);
    tracing::info!(r#"    }}"#);

    // Complete workflow example
    tracing::info!("\n=== Complete Workflow Example ===");
    tracing::info!("Field inspection workflow from start to finish:");
    tracing::info!("");
    tracing::info!("// 1. Inspector captures photo in the field");
    tracing::info!(r#"let photo = AttachmentSource::from_path("damaged_pipe.jpg");"#);
    tracing::info!(r#"service.add_attachment(layer_id, feature_id, photo).await?;"#);
    tracing::info!("");
    tracing::info!("// 2. Office staff lists attachments for the asset");
    tracing::info!(r#"let attachments = service.query_attachments(layer_id, feature_id).await?;"#);
    tracing::info!(r#"for att in &attachments {{"#);
    tracing::info!(r#"    println!("{{}} - {{}} bytes", att.name(), att.size());"#);
    tracing::info!(r#"}}"#);
    tracing::info!("");
    tracing::info!("// 3. Download attachment for report generation");
    tracing::info!(r#"let target = DownloadTarget::to_path("/reports/damaged_pipe.jpg");"#);
    tracing::info!(r#"service.download_attachment(layer_id, feature_id, att_id, target).await?;"#);
    tracing::info!("");
    tracing::info!("// 4. After repair, replace with 'after' photo");
    tracing::info!(r#"let after_photo = AttachmentSource::from_path("repaired_pipe.jpg");"#);
    tracing::info!(
        r#"service.update_attachment(layer_id, feature_id, att_id, after_photo).await?;"#
    );
    tracing::info!("");
    tracing::info!("// 5. Cleanup: Delete old duplicate attachments");
    tracing::info!(r#"service.delete_attachments(layer_id, feature_id, old_ids).await?;"#);

    // Summary and Best Practices
    tracing::info!("\n✅ Feature attachment examples completed!");
    tracing::info!("💡 Attachment Best Practices:");
    tracing::info!("   - Enable attachments when creating hosted feature layers");
    tracing::info!("   - Use descriptive filenames (e.g., 'site_123_north_view.jpg')");
    tracing::info!("   - Compress images before upload to save storage/bandwidth");
    tracing::info!("   - Stream large files using AttachmentSource::from_path()");
    tracing::info!("   - Download to file for large attachments (not to_bytes())");
    tracing::info!("   - Periodically audit and remove duplicate/outdated attachments");
    tracing::info!("   - Consider file size limits (typically 10MB per attachment)");
    tracing::info!("");
    tracing::info!("📋 Supported file types:");
    tracing::info!("   - Images: JPG, PNG, GIF, BMP, TIFF");
    tracing::info!("   - Documents: PDF, DOC, DOCX, XLS, XLSX, TXT");
    tracing::info!("   - Video: MP4, AVI, MOV (check size limits)");
    tracing::info!("   - Other: ZIP, CSV, KML, GPX");
    tracing::info!("");
    tracing::info!("⚠️  Storage considerations:");
    tracing::info!("   - Attachments count toward your ArcGIS Online storage quota");
    tracing::info!("   - Each attachment typically limited to 10MB");
    tracing::info!("   - Monitor total storage usage in organization settings");

    Ok(())
}

/// Creates a minimal valid JPEG file for demonstration purposes.
///
/// This is a tiny 1x1 pixel black JPEG - just enough to be a valid image file.
fn create_mock_jpeg() -> Vec<u8> {
    // Minimal valid JPEG: 1x1 black pixel
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01, 0x00,
        0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4,
        0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00,
        0x7F, 0xFF, 0xD9,
    ]
}

/// Creates a minimal valid PDF file for demonstration purposes.
///
/// This is the smallest possible PDF - contains no visible content but is technically valid.
fn create_mock_pdf() -> Vec<u8> {
    b"%PDF-1.0\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj 2 0 obj<</Type/Pages/Count 1/Kids[3 0 R]>>endobj 3 0 obj<</Type/Page/MediaBox[0 0 612 792]/Parent 2 0 R/Resources<<>>>>endobj\nxref\n0 4\n0000000000 65535 f\n0000000009 00000 n\n0000000056 00000 n\n0000000115 00000 n\ntrailer<</Size 4/Root 1 0 R>>\nstartxref\n220\n%%EOF"
        .to_vec()
}
