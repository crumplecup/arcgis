//! 📝 Portal Item Data - Web Map Text Uploads Example
//!
//! Demonstrates uploading Web Map JSON using the `text` parameter during item creation.
//! Web Maps are one of the few item types that support inline text data (as opposed to file uploads).
//!
//! # What You'll Learn
//!
//! - **Text-based uploads**: Use `AddItemParams::with_text()` for Web Map JSON
//! - **Web Map items**: Upload Web Map JSON definitions inline
//! - **Data verification**: Download and validate uploaded content
//! - **Round-trip integrity**: Ensure data matches after upload/download cycle
//!
//! # Important Note
//!
//! Most item types (GeoJSON, CSV, shapefiles, etc.) require **file uploads**, not text.
//! The `text` parameter is specifically for:
//! - Web Maps (map configuration JSON)
//! - Web Mapping Applications (app configuration JSON)
//! - Other configuration-based item types
//!
//! For GeoJSON and other file-based items, see the `portal_item_data_files` example.
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
//! cargo run --example portal_item_data_text
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example portal_item_data_text
//! ```
//!
//! # Real-World Use Cases
//!
//! - **GeoJSON publishing**: Upload GeoJSON files to Portal catalog
//! - **Web Map creation**: Programmatically create Web Map items
//! - **Configuration management**: Store JSON configuration as Portal items
//! - **Data sharing**: Share structured JSON data within organization
//! - **Metadata management**: Upload and manage item metadata
//!
//! # Item Types Covered
//!
//! - **Web Map**: Map configuration and layer definitions (uses `text` parameter)
//!
//! # Why Not GeoJSON?
//!
//! GeoJSON and most other file-based item types require the `file` parameter, not `text`.
//! See the `portal_item_data_files` example for file-based uploads.

use anyhow::Result;
use arcgis::{AddItemParams, ApiKeyAuth, ApiKeyTier, ArcGISClient, PortalClient};
use arcgis::example_tracker::ExampleTracker;

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

    // Start accountability tracking
    let tracker = ExampleTracker::new("portal_item_data_text")
        .service_type("ExampleClient")
        .start();

    tracing::info!("📝 Portal Item Data - Text Uploads Example");
    tracing::info!("");

    // Load API key from environment
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Content)
        .expect("ARCGIS_CONTENT_KEY environment variable required");

    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new(PORTAL_URL, &client);

    // Demonstrate Web Map text upload (one of the few item types that supports inline text data)
    test_webmap_text_upload(&portal).await?;

    tracing::info!("\n✅ Web Map text upload completed successfully!");
    print_best_practices();

    // Mark tracking as successful
    tracker.success();
    Ok(())
}

/// Demonstrates uploading GeoJSON as text data.
async fn test_webmap_text_upload(portal: &PortalClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Web Map Text Upload ===");
    tracing::info!("Create Web Map item and upload map definition as text");
    tracing::info!("");

    // Create a sample Web Map JSON definition
    let webmap_json = r#"{
  "operationalLayers": [
    {
      "id": "world_imagery",
      "title": "World Imagery",
      "url": "https://services.arcgisonline.com/arcgis/rest/services/World_Imagery/MapServer",
      "layerType": "ArcGISTiledMapServiceLayer",
      "opacity": 1,
      "visibility": true
    }
  ],
  "baseMap": {
    "baseMapLayers": [
      {
        "id": "defaultBasemap",
        "url": "https://services.arcgisonline.com/arcgis/rest/services/World_Topo_Map/MapServer",
        "layerType": "ArcGISTiledMapServiceLayer"
      }
    ],
    "title": "Topographic"
  },
  "spatialReference": {
    "wkid": 102100,
    "latestWkid": 3857
  },
  "initialExtent": {
    "xmin": -13692297.398272788,
    "ymin": 4470035.373838849,
    "xmax": -13175564.849020682,
    "ymax": 4812498.190346919,
    "spatialReference": {
      "wkid": 102100
    }
  },
  "version": "2.0"
}"#;

    let original_size = webmap_json.len();
    tracing::info!("   Web Map JSON size: {} bytes", original_size);
    tracing::info!("   Layers: 1 operational + 1 basemap");
    tracing::info!("");

    // STEP 1: Create Web Map item WITH data (text parameter)
    tracing::info!("📋 STEP 1: Creating Web Map item with data");
    tracing::info!("   Method: add_item() with .with_text()");
    tracing::info!("   Data size: {} bytes", original_size);
    tracing::info!("");

    let item_params = AddItemParams::new("Sample Web Map (Text Upload Demo)", "Web Map")
        .with_description("Demonstrates text-based Web Map upload using add_item with text parameter")
        .with_tags(vec![
            "demo".to_string(),
            "webmap".to_string(),
            "text-upload".to_string(),
        ])
        .with_text(webmap_json.to_string());

    let add_result = portal.add_item(item_params).await?;
    let item_id = add_result.id().to_string();
    tracing::info!("✅ Created item with data: {}", item_id);
    tracing::info!("");

    // STEP 2: Download and verify
    tracing::info!("📥 STEP 2: Downloading and verifying data");
    let retrieved_data = portal.get_item_data(&item_id).await?;
    let retrieved_size = retrieved_data.len();

    tracing::info!("   Retrieved size: {} bytes", retrieved_size);
    tracing::info!("   Original size:  {} bytes", original_size);

    assert!(retrieved_size > 0, "Retrieved data is empty!");

    // Parse to verify it's valid JSON
    let retrieved_string = String::from_utf8(retrieved_data.to_vec())?;
    let parsed: serde_json::Value = serde_json::from_str(&retrieved_string)?;

    // Verify structure
    assert!(
        parsed.get("operationalLayers").is_some(),
        "Missing operationalLayers"
    );
    assert!(parsed.get("baseMap").is_some(), "Missing baseMap");

    let layers = parsed["operationalLayers"].as_array().unwrap();
    assert_eq!(layers.len(), 1, "Expected 1 operational layer");

    tracing::info!(
        "✅ Valid Web Map JSON with {} operational layers",
        layers.len()
    );
    tracing::info!("");

    // STEP 3: Cleanup
    tracing::info!("🧹 STEP 3: Cleaning up");
    let delete_result = portal.delete_item(&item_id).await?;
    assert!(delete_result.success(), "Failed to delete item");
    tracing::info!("✅ Deleted item");
    tracing::info!("");

    tracing::info!("📊 Web Map Text Upload Summary:");
    tracing::info!("   ✓ Created Web Map item with {} bytes of data", original_size);
    tracing::info!("   ✓ Retrieved and verified map definition");
    tracing::info!("   ✓ Cleaned up resources");

    Ok(())
}

/// Prints best practices for text-based uploads.
fn print_best_practices() {
    tracing::info!("\n💡 Text Upload Best Practices:");
    tracing::info!("   - Use AddItemParams::with_text() for configuration JSON items");
    tracing::info!("   - Suitable for: Web Maps, Web Mapping Applications, configuration JSON");
    tracing::info!("   - Content is sent directly as 'text' parameter during add_item()");
    tracing::info!("   - No file encoding overhead vs File uploads");
    tracing::info!("");
    tracing::info!("🎯 When to Use Text vs File:");
    tracing::info!("   Text:  Web Maps, Web Mapping Applications, configuration JSON");
    tracing::info!("   File:  GeoJSON, CSV, shapefiles, images, PDFs, packages, most data files");
    tracing::info!("   Url:   External service references");
    tracing::info!("");
    tracing::info!("⚠️  Important Notes:");
    tracing::info!("   - Text data must be provided during item creation (add_item)");
    tracing::info!("   - Most item types (including GeoJSON) require file uploads, not text");
    tracing::info!("   - The 'text' parameter is specifically for configuration-based items");
    tracing::info!("   - Choose based on item type and data format");
    tracing::info!("   - Both methods support full round-trip integrity");
}
