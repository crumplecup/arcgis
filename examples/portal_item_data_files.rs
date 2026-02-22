//! 📁 Portal Item Data - File Uploads Example
//!
//! Demonstrates uploading binary and text files using the `File` variant of `ItemDataUpload`.
//! This example shows how to work with CSV data, images, and PDF documents as Portal items.
//!
//! # What You'll Learn
//!
//! - **File-based uploads**: Use `ItemDataUpload::File` for binary and text file content
//! - **CSV items**: Upload structured CSV data with proper MIME types
//! - **Image items**: Upload PNG/JPEG images as Portal items
//! - **PDF items**: Upload PDF documents
//! - **MIME type handling**: Set correct content types for different file formats
//! - **Data verification**: Download and validate uploaded binary content
//! - **Round-trip integrity**: Ensure files match after upload/download cycle
//!
//! # Prerequisites
//!
//! - Required: API key with content creation privileges in `.env`
//! - Permissions: Create items, manage content
//!
//! ## Environment Variables
//!
//! ```env
//! ARCGIS_CONTENT_KEY=your_api_key_with_content_privileges
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example portal_item_data_files
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example portal_item_data_files
//! ```
//!
//! # Real-World Use Cases
//!
//! - **CSV data publishing**: Upload tabular data to Portal catalog
//! - **Map images**: Share static map images as Portal items
//! - **Document management**: Store PDFs and other documents
//! - **Asset management**: Organize project files and resources
//! - **Data packages**: Bundle and share datasets
//!
//! # Item Types Covered
//!
//! - **CSV**: Comma-separated value data files
//! - **Image**: PNG, JPEG, and other image formats
//! - **PDF**: Portable document format files

use anyhow::Result;
use arcgis::{AddItemParams, ApiKeyAuth, ApiKeyTier, ArcGISClient, ItemDataUpload, PortalClient};

/// Portal base URL for ArcGIS Online
const PORTAL_URL: &str = "https://www.arcgis.com/sharing/rest";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("📁 Portal Item Data - File Uploads Example");
    tracing::info!("");

    // Load API key from environment
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Content)
        .expect("ARCGIS_CONTENT_KEY environment variable required");

    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new(PORTAL_URL, &client);

    // Demonstrate different file-based upload scenarios
    test_csv_file_upload(&portal).await?;
    test_image_file_upload(&portal).await?;
    test_pdf_file_upload(&portal).await?;

    tracing::info!("\n✅ All file upload examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates uploading CSV data as a file.
async fn test_csv_file_upload(portal: &PortalClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: CSV File Upload ===");
    tracing::info!("Create CSV item and upload tabular data as file");
    tracing::info!("");

    // Create sample CSV data
    let csv_data = "City,State,Population,Latitude,Longitude
San Francisco,California,883305,37.7749,-122.4194
Los Angeles,California,3979576,34.0522,-118.2437
San Jose,California,1026908,37.3382,-121.8863
San Diego,California,1423851,32.7157,-117.1611
Sacramento,California,524943,38.5816,-121.4944
";

    let data_bytes = csv_data.as_bytes().to_vec();
    let original_size = data_bytes.len();
    tracing::info!("   CSV size: {} bytes", original_size);
    tracing::info!("   Rows: 6 (including header)");
    tracing::info!("");

    // STEP 1: Create CSV item (metadata only)
    tracing::info!("📋 STEP 1: Creating CSV item");
    let item_params = AddItemParams::new("California Cities CSV (File Upload Demo)", "CSV")
        .with_description("Demonstrates file-based CSV upload using ItemDataUpload::File")
        .with_tags(vec![
            "demo".to_string(),
            "csv".to_string(),
            "file-upload".to_string(),
        ]);

    let add_result = portal.add_item(item_params).await?;
    let item_id = add_result.id().to_string();
    tracing::info!("✅ Created item: {}", item_id);
    tracing::info!("");

    // STEP 2: Upload CSV data as file
    tracing::info!("📤 STEP 2: Uploading CSV as file");
    tracing::info!("   Method: update_item_data_v2()");
    tracing::info!("   Upload Type: ItemDataUpload::File");
    tracing::info!("   MIME Type: text/csv");
    tracing::info!("");

    let upload = ItemDataUpload::File {
        data: data_bytes.clone(),
        filename: "cities.csv".to_string(),
        mime_type: "text/csv".to_string(),
    };
    let update_result = portal.update_item_data_v2(&item_id, upload).await?;

    assert!(
        update_result.success(),
        "Failed to upload CSV file: {:?}",
        update_result
    );
    tracing::info!("✅ Uploaded {} bytes as CSV file", original_size);
    tracing::info!("");

    // STEP 3: Download and verify
    tracing::info!("📥 STEP 3: Downloading and verifying data");
    let retrieved_data = portal.get_item_data(&item_id).await?;
    let retrieved_size = retrieved_data.len();

    tracing::info!("   Retrieved size: {} bytes", retrieved_size);
    tracing::info!("   Original size:  {} bytes", original_size);

    // Verify data integrity
    assert!(retrieved_size > 0, "Retrieved data is empty!");
    assert_eq!(
        retrieved_size, original_size,
        "Size mismatch: expected {}, got {}",
        original_size, retrieved_size
    );

    // Verify CSV structure
    let retrieved_string = String::from_utf8(retrieved_data.to_vec())?;
    let lines: Vec<&str> = retrieved_string.lines().collect();

    assert_eq!(
        lines.len(),
        6,
        "Expected 6 lines (header + 5 cities), got {}",
        lines.len()
    );
    assert!(lines[0].starts_with("City,State"), "CSV header mismatch");

    tracing::info!("✅ Valid CSV with {} lines", lines.len());
    tracing::info!("");

    // STEP 4: Cleanup
    tracing::info!("🧹 STEP 4: Cleaning up");
    let delete_result = portal.delete_item(&item_id).await?;
    assert!(delete_result.success(), "Failed to delete item");
    tracing::info!("✅ Deleted item");
    tracing::info!("");

    tracing::info!("📊 CSV File Upload Summary:");
    tracing::info!("   ✓ Created CSV item");
    tracing::info!(
        "   ✓ Uploaded {} bytes via ItemDataUpload::File",
        original_size
    );
    tracing::info!("   ✓ Retrieved and verified {} lines", lines.len());
    tracing::info!("   ✓ Cleaned up resources");

    Ok(())
}

/// Demonstrates uploading an image file.
async fn test_image_file_upload(portal: &PortalClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Image File Upload ===");
    tracing::info!("Create image item and upload PNG data as file");
    tracing::info!("");

    // Create minimal valid 1x1 PNG image (67 bytes)
    // PNG signature + IHDR + IDAT + IEND chunks
    let png_data: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44,
        0x41, // IDAT chunk
        0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4,
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, // IEND chunk
        0x42, 0x60, 0x82,
    ];

    let original_size = png_data.len();
    tracing::info!("   PNG size: {} bytes", original_size);
    tracing::info!("   Dimensions: 1x1 pixel (minimal test image)");
    tracing::info!("");

    // STEP 1: Create Image item
    tracing::info!("📋 STEP 1: Creating Image item");
    let item_params = AddItemParams::new("Test Image (File Upload Demo)", "Image")
        .with_description("Demonstrates file-based image upload using ItemDataUpload::File")
        .with_tags(vec![
            "demo".to_string(),
            "image".to_string(),
            "file-upload".to_string(),
        ]);

    let add_result = portal.add_item(item_params).await?;
    let item_id = add_result.id().to_string();
    tracing::info!("✅ Created item: {}", item_id);
    tracing::info!("");

    // STEP 2: Upload PNG data as file
    tracing::info!("📤 STEP 2: Uploading PNG as file");
    tracing::info!("   Method: update_item_data_v2()");
    tracing::info!("   Upload Type: ItemDataUpload::File");
    tracing::info!("   MIME Type: image/png");
    tracing::info!("");

    let upload = ItemDataUpload::File {
        data: png_data.clone(),
        filename: "test.png".to_string(),
        mime_type: "image/png".to_string(),
    };
    let update_result = portal.update_item_data_v2(&item_id, upload).await?;

    assert!(
        update_result.success(),
        "Failed to upload PNG file: {:?}",
        update_result
    );
    tracing::info!("✅ Uploaded {} bytes as PNG file", original_size);
    tracing::info!("");

    // STEP 3: Download and verify
    tracing::info!("📥 STEP 3: Downloading and verifying data");
    let retrieved_data = portal.get_item_data(&item_id).await?;
    let retrieved_size = retrieved_data.len();

    tracing::info!("   Retrieved size: {} bytes", retrieved_size);
    tracing::info!("   Original size:  {} bytes", original_size);

    assert!(retrieved_size > 0, "Retrieved data is empty!");
    assert_eq!(
        retrieved_size, original_size,
        "Size mismatch: expected {}, got {}",
        original_size, retrieved_size
    );

    // Verify PNG signature
    let png_signature = &retrieved_data[0..8];
    let expected_signature: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    assert_eq!(png_signature, expected_signature, "PNG signature mismatch");

    tracing::info!("✅ Valid PNG with correct signature");
    tracing::info!("");

    // STEP 4: Cleanup
    tracing::info!("🧹 STEP 4: Cleaning up");
    let delete_result = portal.delete_item(&item_id).await?;
    assert!(delete_result.success(), "Failed to delete item");
    tracing::info!("✅ Deleted item");
    tracing::info!("");

    tracing::info!("📊 Image File Upload Summary:");
    tracing::info!("   ✓ Created Image item");
    tracing::info!(
        "   ✓ Uploaded {} bytes via ItemDataUpload::File",
        original_size
    );
    tracing::info!("   ✓ Retrieved and verified PNG signature");
    tracing::info!("   ✓ Cleaned up resources");

    Ok(())
}

/// Demonstrates uploading a PDF file.
async fn test_pdf_file_upload(portal: &PortalClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: PDF File Upload ===");
    tracing::info!("Create PDF item and upload document data as file");
    tracing::info!("");

    // Create minimal valid PDF (134 bytes)
    let pdf_data = b"%PDF-1.4
1 0 obj
<<
/Type /Catalog
/Pages 2 0 R
>>
endobj
2 0 obj
<<
/Type /Pages
/Count 0
/Kids []
>>
endobj
xref
0 3
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
trailer
<<
/Size 3
/Root 1 0 R
>>
startxref
109
%%EOF
";

    let data_bytes = pdf_data.to_vec();
    let original_size = data_bytes.len();
    tracing::info!("   PDF size: {} bytes", original_size);
    tracing::info!("   Pages: 0 (minimal test document)");
    tracing::info!("");

    // STEP 1: Create PDF item
    tracing::info!("📋 STEP 1: Creating PDF item");
    let item_params = AddItemParams::new("Test Document (File Upload Demo)", "PDF")
        .with_description("Demonstrates file-based PDF upload using ItemDataUpload::File")
        .with_tags(vec![
            "demo".to_string(),
            "pdf".to_string(),
            "file-upload".to_string(),
        ]);

    let add_result = portal.add_item(item_params).await?;
    let item_id = add_result.id().to_string();
    tracing::info!("✅ Created item: {}", item_id);
    tracing::info!("");

    // STEP 2: Upload PDF data as file
    tracing::info!("📤 STEP 2: Uploading PDF as file");
    tracing::info!("   Method: update_item_data_v2()");
    tracing::info!("   Upload Type: ItemDataUpload::File");
    tracing::info!("   MIME Type: application/pdf");
    tracing::info!("");

    let upload = ItemDataUpload::File {
        data: data_bytes.clone(),
        filename: "document.pdf".to_string(),
        mime_type: "application/pdf".to_string(),
    };
    let update_result = portal.update_item_data_v2(&item_id, upload).await?;

    assert!(
        update_result.success(),
        "Failed to upload PDF file: {:?}",
        update_result
    );
    tracing::info!("✅ Uploaded {} bytes as PDF file", original_size);
    tracing::info!("");

    // STEP 3: Download and verify
    tracing::info!("📥 STEP 3: Downloading and verifying data");
    let retrieved_data = portal.get_item_data(&item_id).await?;
    let retrieved_size = retrieved_data.len();

    tracing::info!("   Retrieved size: {} bytes", retrieved_size);
    tracing::info!("   Original size:  {} bytes", original_size);

    assert!(retrieved_size > 0, "Retrieved data is empty!");
    assert_eq!(
        retrieved_size, original_size,
        "Size mismatch: expected {}, got {}",
        original_size, retrieved_size
    );

    // Verify PDF header
    let pdf_header = &retrieved_data[0..8];
    assert_eq!(pdf_header, b"%PDF-1.4", "PDF header mismatch");

    tracing::info!("✅ Valid PDF with correct header");
    tracing::info!("");

    // STEP 4: Cleanup
    tracing::info!("🧹 STEP 4: Cleaning up");
    let delete_result = portal.delete_item(&item_id).await?;
    assert!(delete_result.success(), "Failed to delete item");
    tracing::info!("✅ Deleted item");
    tracing::info!("");

    tracing::info!("📊 PDF File Upload Summary:");
    tracing::info!("   ✓ Created PDF item");
    tracing::info!(
        "   ✓ Uploaded {} bytes via ItemDataUpload::File",
        original_size
    );
    tracing::info!("   ✓ Retrieved and verified PDF header");
    tracing::info!("   ✓ Cleaned up resources");

    Ok(())
}

/// Prints best practices for file-based uploads.
fn print_best_practices() {
    tracing::info!("\n💡 File Upload Best Practices:");
    tracing::info!("   - Use ItemDataUpload::File for binary and text files");
    tracing::info!("   - Set correct MIME types for file formats:");
    tracing::info!("     • CSV: text/csv");
    tracing::info!("     • PNG: image/png");
    tracing::info!("     • JPEG: image/jpeg");
    tracing::info!("     • PDF: application/pdf");
    tracing::info!("     • ZIP: application/zip");
    tracing::info!("   - Filename helps Portal identify content");
    tracing::info!("   - File uploads use multipart encoding");
    tracing::info!("");
    tracing::info!("🎯 When to Use File vs Text vs Url:");
    tracing::info!("   File:  Binary files, CSVs, images, PDFs, packages");
    tracing::info!("   Text:  JSON content, Web Maps, GeoJSON text");
    tracing::info!("   Url:   External service references");
    tracing::info!("");
    tracing::info!("⚠️  Important Notes:");
    tracing::info!("   - Binary data requires File variant (images, PDFs)");
    tracing::info!("   - CSV can use File even though it's text (preserves encoding)");
    tracing::info!("   - Always verify round-trip for critical data");
    tracing::info!("   - MIME types affect how Portal displays items");
}
