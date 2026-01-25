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
//! ## Configure Environment Variables
//!
//! Set these in your `.env` file:
//!
//! ```env
//! # API keys for different privilege levels
//! ARCGIS_CONTENT_KEY=key_with_content_create_privileges    # For creating/deleting services
//! ARCGIS_FEATURES_KEY=key_with_feature_edit_privileges      # For editing features
//! ```
//!
//! ## How This Example Works
//!
//! This example is fully automated - zero manual setup required:
//! 1. **Creates** a hosted feature service with attachments enabled (ARCGIS_CONTENT_KEY)
//! 2. **Adds** layer definition with proper attachment configuration (ARCGIS_CONTENT_KEY)
//! 3. **Creates** a test feature (ARCGIS_FEATURES_KEY)
//! 4. **Demonstrates** all attachment operations (ARCGIS_FEATURES_KEY)
//! 5. **Deletes** the test feature (ARCGIS_FEATURES_KEY)
//! 6. **Deletes** the entire service (ARCGIS_CONTENT_KEY)
//!
//! Run it multiple times - it creates and cleans up everything each time!
//!
//! # Running
//!
//! ```bash
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

use anyhow::{Context, Result};
use arcgis::{
    ApiKeyAuth, ArcGISClient, ArcGISGeometry, ArcGISPoint, AttachmentId, AttachmentSource,
    CreateServiceParams, DownloadTarget, EditOptions, EnvConfig, Feature, FeatureServiceClient,
    ObjectId, PortalClient,
};
use secrecy::ExposeSecret;
use std::collections::HashMap;

/// Service information needed throughout the example.
struct ServiceInfo {
    service_item_id: String,
    service_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("📎 ArcGIS Feature Attachments Examples");
    tracing::info!("Demonstrating file attachment management workflows");

    // Load environment configuration (automatically loads .env)
    let config = EnvConfig::global();

    // Validate required keys are present
    let content_key = config.arcgis_content_key.as_ref().context(
        "ARCGIS_CONTENT_KEY not set. Add to .env:\n\
         ARCGIS_CONTENT_KEY=your_api_key_here\n\
         \n\
         This key is used to create and delete services.",
    )?;

    let features_key = config.arcgis_features_key.as_ref().context(
        "ARCGIS_FEATURES_KEY not set. Add to .env:\n\
         ARCGIS_FEATURES_KEY=your_api_key_here\n\
         \n\
         This key is used to create/edit features and attachments.",
    )?;

    // Step 1: Create feature service with attachments enabled
    let service_info = create_feature_service(content_key.expose_secret()).await?;

    // Step 2: Create clients for feature operations
    let features_auth = ApiKeyAuth::new(features_key.expose_secret());
    let features_client = ArcGISClient::new(features_auth);
    let feature_service = FeatureServiceClient::new(&service_info.service_url, &features_client);
    let layer_id = arcgis::LayerId::new(0);

    // Step 3: Create test feature
    let object_id = create_test_feature(&feature_service, layer_id).await?;

    // Step 4: Demonstrate attachment operations
    demonstrate_list_attachments(&feature_service, layer_id, object_id).await?;
    demonstrate_add_photo(&feature_service, layer_id, object_id).await?;
    demonstrate_add_pdf(&feature_service, layer_id, object_id).await?;
    demonstrate_download(&feature_service, layer_id, object_id).await?;
    demonstrate_update(&feature_service, layer_id, object_id).await?;
    demonstrate_delete(&feature_service, layer_id, object_id).await?;

    // Step 5: Cleanup
    cleanup(
        content_key.expose_secret(),
        &feature_service,
        layer_id,
        object_id,
        &service_info.service_item_id,
    )
    .await?;

    print_best_practices();

    Ok(())
}

/// Creates a hosted feature service with attachments enabled.
async fn create_feature_service(content_key: &str) -> Result<ServiceInfo> {
    tracing::info!("\n=== Step 1: Creating Feature Service ===");
    tracing::info!("Creating hosted feature service with attachments enabled");

    // Create portal client with content management key
    let content_auth = ApiKeyAuth::new(content_key);
    let content_client = ArcGISClient::new(content_auth);
    let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &content_client);

    // Create unique service name to avoid conflicts
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let service_name = format!("AttachmentTest_{}", timestamp);

    // Create the empty hosted feature service
    let create_params = CreateServiceParams::new(&service_name)
        .with_description("Temporary service for testing attachments - will be deleted")
        .with_capabilities("Query,Create,Update,Delete,Editing")
        .with_max_record_count(1000);

    let create_result = portal.create_service(create_params).await?;

    let service_item_id = create_result
        .service_item_id()
        .clone()
        .context("Created service should have item ID")?;
    let service_url = create_result
        .service_url()
        .clone()
        .context("Created service should have URL")?;

    tracing::info!(
        service_item_id = %service_item_id,
        service_url = %service_url,
        "✅ Empty feature service created"
    );

    // Add layer definition to the service using admin endpoint
    tracing::info!("Adding layer with attachments to the service");
    let admin_url = service_url.replace("/rest/services/", "/rest/admin/services/");
    let add_def_url = format!("{}/addToDefinition", admin_url);

    let layer_definition = create_layer_definition_for_add();

    let mut form = reqwest::multipart::Form::new()
        .text("f", "json")
        .text("addToDefinition", layer_definition.to_string());

    // Get token if required
    if let Some(token) = content_client.get_token_if_required().await? {
        form = form.text("token", token);
    }

    let response = content_client
        .http()
        .post(&add_def_url)
        .multipart(form)
        .send()
        .await?;

    let response_text = response.text().await?;
    tracing::debug!(response = %response_text, "addToDefinition response");

    // Parse response to check for errors
    let add_result: serde_json::Value = serde_json::from_str(&response_text)?;
    if let Some(error) = add_result.get("error") {
        anyhow::bail!("Failed to add layer: {}", error);
    }

    tracing::info!("✅ Layer with attachments enabled added to service");

    Ok(ServiceInfo {
        service_item_id,
        service_url,
    })
}

/// Creates a test feature to demonstrate attachment operations.
async fn create_test_feature(
    feature_service: &FeatureServiceClient<'_>,
    layer_id: arcgis::LayerId,
) -> Result<ObjectId> {
    tracing::info!("\n=== Step 2: Creating Test Feature ===");
    tracing::info!("Creating a test feature to demonstrate attachments");

    let mut attributes = HashMap::new();
    attributes.insert(
        "Name".to_string(),
        serde_json::json!("Attachment Test Feature"),
    );

    let geometry = ArcGISGeometry::Point(ArcGISPoint {
        x: -122.4194,
        y: 37.7749,
        z: None,
        m: None,
        spatial_reference: None,
    });

    let test_feature = Feature::new(attributes, Some(geometry));

    let add_result = feature_service
        .add_features(layer_id, vec![test_feature], EditOptions::new())
        .await?;

    let object_id = if let Some(result) = add_result.add_results().first() {
        if *result.success() {
            let oid = result
                .object_id()
                .as_ref()
                .copied()
                .context("Added feature should have ObjectID")?;
            tracing::info!(object_id = oid.0, "✅ Test feature created");
            oid
        } else {
            anyhow::bail!("Failed to create test feature: {:?}", result.error());
        }
    } else {
        anyhow::bail!("No results from add_features operation");
    };

    Ok(object_id)
}

/// Demonstrates listing existing attachments on a feature.
async fn demonstrate_list_attachments(
    feature_service: &FeatureServiceClient<'_>,
    layer_id: arcgis::LayerId,
    object_id: ObjectId,
) -> Result<()> {
    tracing::info!("\n=== Example 1: Listing Existing Attachments ===");
    tracing::info!("Query attachments for feature {}", object_id);

    let attachments = feature_service
        .query_attachments(layer_id, object_id)
        .await?;

    tracing::info!(
        attachment_count = attachments.len(),
        "Found existing attachments"
    );

    for attachment in &attachments {
        tracing::info!(
            id = attachment.id().0,
            name = attachment.name(),
            size = attachment.size(),
            content_type = attachment.content_type(),
            "Existing attachment"
        );
    }

    Ok(())
}

/// Demonstrates adding a photo attachment.
async fn demonstrate_add_photo(
    feature_service: &FeatureServiceClient<'_>,
    layer_id: arcgis::LayerId,
    object_id: ObjectId,
) -> Result<()> {
    tracing::info!("\n=== Example 2: Adding Photo Attachment ===");
    tracing::info!("Upload an inspection photo to the feature");

    // Create a mock JPEG image (minimal valid JPEG header + data)
    let mock_jpeg_data = create_mock_jpeg();
    let source = AttachmentSource::from_bytes("inspection_photo.jpg", mock_jpeg_data.clone());

    let add_result = feature_service
        .add_attachment(layer_id, object_id, source)
        .await?;

    if *add_result.success() {
        tracing::info!(
            object_id = ?add_result.object_id().as_ref().map(|id| id.0),
            global_id = ?add_result.global_id(),
            "✅ Photo attached successfully"
        );
    } else {
        tracing::warn!("Failed to attach photo");
    }

    tracing::info!("💡 Tip: Use AttachmentSource::from_path() for large files to stream from disk");

    Ok(())
}

/// Demonstrates adding a PDF document attachment.
async fn demonstrate_add_pdf(
    feature_service: &FeatureServiceClient<'_>,
    layer_id: arcgis::LayerId,
    object_id: ObjectId,
) -> Result<()> {
    tracing::info!("\n=== Example 3: Adding PDF Document ===");
    tracing::info!("Attach an inspection report document");

    let mock_pdf_data = create_mock_pdf();
    let source = AttachmentSource::from_bytes("inspection_report.pdf", mock_pdf_data.clone());

    let pdf_result = feature_service
        .add_attachment(layer_id, object_id, source)
        .await?;

    if *pdf_result.success() {
        tracing::info!(
            object_id = ?pdf_result.object_id().as_ref().map(|id| id.0),
            global_id = ?pdf_result.global_id(),
            "✅ PDF attached successfully"
        );
    } else {
        tracing::warn!("Failed to attach PDF");
    }

    Ok(())
}

/// Demonstrates downloading attachments to file and memory.
async fn demonstrate_download(
    feature_service: &FeatureServiceClient<'_>,
    layer_id: arcgis::LayerId,
    object_id: ObjectId,
) -> Result<()> {
    tracing::info!("\n=== Example 4: Downloading Attachments ===");
    tracing::info!("Retrieve attachment files for reporting");

    // Query current attachments to get IDs
    let attachments = feature_service
        .query_attachments(layer_id, object_id)
        .await?;

    if let Some(attachment) = attachments.first() {
        let attachment_id = *attachment.id();

        // Download to file
        let target = DownloadTarget::to_path("/tmp/downloaded_attachment.dat");
        let download_result = feature_service
            .download_attachment(layer_id, object_id, attachment_id, target)
            .await?;

        if let Some(path) = download_result.path() {
            tracing::info!(path = ?path, "✅ Downloaded to file");
        }

        // Download to memory
        let target = DownloadTarget::to_bytes();
        let download_result = feature_service
            .download_attachment(layer_id, object_id, attachment_id, target)
            .await?;

        if let Some(bytes) = download_result.bytes() {
            tracing::info!(
                size = bytes.len(),
                "✅ Downloaded to memory ({} bytes)",
                bytes.len()
            );
        }
    } else {
        tracing::info!("No attachments available to download");
    }

    tracing::info!("💡 Tip: Use to_path() for large files to avoid loading into memory");

    Ok(())
}

/// Demonstrates updating an existing attachment.
async fn demonstrate_update(
    feature_service: &FeatureServiceClient<'_>,
    layer_id: arcgis::LayerId,
    object_id: ObjectId,
) -> Result<()> {
    tracing::info!("\n=== Example 5: Updating an Attachment ===");
    tracing::info!("Replace an outdated photo with a new one");

    // Update the first attachment if it exists
    let attachments = feature_service
        .query_attachments(layer_id, object_id)
        .await?;

    if let Some(attachment) = attachments.first() {
        let attachment_id = *attachment.id();

        // Create updated JPEG content
        let updated_jpeg = create_mock_jpeg();
        let source = AttachmentSource::from_bytes("updated_photo.jpg", updated_jpeg);

        let update_result = feature_service
            .update_attachment(layer_id, object_id, attachment_id, source)
            .await?;

        if *update_result.success() {
            tracing::info!(
                attachment_id = attachment.id().0,
                "✅ Attachment updated successfully"
            );
        } else {
            tracing::warn!("Failed to update attachment");
        }
    } else {
        tracing::info!("No attachments available to update");
    }

    Ok(())
}

/// Demonstrates deleting attachments.
async fn demonstrate_delete(
    feature_service: &FeatureServiceClient<'_>,
    layer_id: arcgis::LayerId,
    object_id: ObjectId,
) -> Result<()> {
    tracing::info!("\n=== Example 6: Deleting Attachments ===");
    tracing::info!("Clean up test attachments created in this example");

    // Query all current attachments
    let final_attachments = feature_service
        .query_attachments(layer_id, object_id)
        .await?;

    if !final_attachments.is_empty() {
        // Delete attachments we created (filter by name to avoid deleting user's existing data)
        let test_attachment_ids: Vec<AttachmentId> = final_attachments
            .iter()
            .filter(|att| {
                att.name() == "inspection_photo.jpg"
                    || att.name() == "inspection_report.pdf"
                    || att.name() == "updated_photo.jpg"
            })
            .map(|att| *att.id())
            .collect();

        if !test_attachment_ids.is_empty() {
            tracing::info!(
                count = test_attachment_ids.len(),
                "Deleting test attachments"
            );

            let delete_result = feature_service
                .delete_attachments(layer_id, object_id, test_attachment_ids)
                .await?;

            for item in &delete_result.delete_attachment_results {
                if *item.success() {
                    tracing::info!(
                        object_id = item.object_id().0,
                        "✅ Deleted attachment from feature"
                    );
                } else {
                    tracing::warn!(
                        object_id = item.object_id().0,
                        "Failed to delete attachment"
                    );
                }
            }
        } else {
            tracing::info!("No test attachments to clean up");
        }
    } else {
        tracing::info!("No attachments found");
    }

    Ok(())
}

/// Cleans up test data - deletes feature and service.
async fn cleanup(
    content_key: &str,
    feature_service: &FeatureServiceClient<'_>,
    layer_id: arcgis::LayerId,
    object_id: ObjectId,
    service_item_id: &str,
) -> Result<()> {
    // Cleanup Step 1: Delete the test feature
    tracing::info!("\n=== Step 7: Cleanup - Deleting Test Feature ===");

    let delete_result = feature_service
        .delete_features(layer_id, vec![object_id], EditOptions::new())
        .await?;

    if let Some(result) = delete_result.delete_results().first() {
        if *result.success() {
            tracing::info!(object_id = object_id.0, "✅ Test feature deleted");
        } else {
            tracing::warn!(
                object_id = object_id.0,
                error = ?result.error(),
                "Failed to delete test feature"
            );
        }
    }

    // Cleanup Step 2: Delete the feature service
    tracing::info!("\n=== Step 8: Cleanup - Deleting Feature Service ===");

    let content_auth = ApiKeyAuth::new(content_key);
    let content_client = ArcGISClient::new(content_auth);
    let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &content_client);

    portal.delete_service(service_item_id).await?;
    tracing::info!(service_item_id = %service_item_id, "✅ Feature service deleted");

    Ok(())
}

/// Prints best practices and tips for working with attachments.
fn print_best_practices() {
    tracing::info!("\n✅ All attachment operations completed successfully!");
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
    tracing::info!("   - Attachments count toward your ArcGIS storage quota");
    tracing::info!("   - Each attachment typically limited to 10MB");
    tracing::info!("   - Monitor total storage usage in organization settings");
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
        0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xC4, 0x00, 0x14, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xDA, 0x00,
        0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00, 0x7F, 0xFF, 0xD9,
    ]
}

/// Creates a minimal valid PDF file for demonstration purposes.
///
/// This is a tiny PDF containing just the text "Test" - enough to be valid.
fn create_mock_pdf() -> Vec<u8> {
    // Minimal valid PDF with "Test" text
    let pdf = b"%PDF-1.4
1 0 obj
<<
/Type /Catalog
/Pages 2 0 R
>>
endobj
2 0 obj
<<
/Type /Pages
/Kids [3 0 R]
/Count 1
>>
endobj
3 0 obj
<<
/Type /Page
/Parent 2 0 R
/MediaBox [0 0 612 792]
/Contents 4 0 R
/Resources <<
/Font <<
/F1 <<
/Type /Font
/Subtype /Type1
/BaseFont /Helvetica
>>
>>
>>
>>
endobj
4 0 obj
<<
/Length 44
>>
stream
BT
/F1 12 Tf
100 700 Td
(Test) Tj
ET
endstream
endobj
xref
0 5
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000317 00000 n
trailer
<<
/Size 5
/Root 1 0 R
>>
startxref
410
%%EOF
";
    pdf.to_vec()
}

/// Creates a proper layer definition for addToDefinition operation.
///
/// This creates a point layer with attachments enabled, following the ArcGIS REST API spec:
/// - objectIdField is required
/// - hasAttachments must be true
/// - attachmentProperties defines metadata captured with attachments
/// - templates provide editing support
/// - fields define the layer schema
fn create_layer_definition_for_add() -> serde_json::Value {
    serde_json::json!({
        "layers": [{
            "id": 0,
            "name": "AttachmentTestPoints",
            "type": "Feature Layer",
            "description": "Layer for testing attachments",
            "geometryType": "esriGeometryPoint",
            "hasAttachments": true,
            "attachmentProperties": [
                {"name": "name", "isEnabled": true},
                {"name": "size", "isEnabled": true},
                {"name": "contentType", "isEnabled": true},
                {"name": "keywords", "isEnabled": true}
            ],
            "objectIdField": "OBJECTID",
            "globalIdField": "GlobalID",
            "displayField": "Name",
            "fields": [
                {
                    "name": "OBJECTID",
                    "type": "esriFieldTypeOID",
                    "alias": "Object ID",
                    "editable": false,
                    "nullable": false
                },
                {
                    "name": "GlobalID",
                    "type": "esriFieldTypeGlobalID",
                    "alias": "Global ID",
                    "editable": false,
                    "nullable": false
                },
                {
                    "name": "Name",
                    "type": "esriFieldTypeString",
                    "alias": "Name",
                    "length": 256,
                    "editable": true,
                    "nullable": true
                },
                {
                    "name": "Description",
                    "type": "esriFieldTypeString",
                    "alias": "Description",
                    "length": 1024,
                    "editable": true,
                    "nullable": true
                }
            ],
            "templates": [{
                "name": "New Feature",
                "description": "",
                "drawingTool": "esriFeatureEditToolPoint",
                "prototype": {
                    "attributes": {
                        "Name": null,
                        "Description": null
                    }
                }
            }]
        }]
    })
}
