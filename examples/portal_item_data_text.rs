//! 📝 Portal Item Data - Text Uploads Example
//!
//! Demonstrates uploading JSON-based item data using the `Text` variant of `ItemDataUpload`.
//! This example shows how to work with Web Maps, GeoJSON text, and other JSON-structured
//! portal items.
//!
//! # What You'll Learn
//!
//! - **Text-based uploads**: Use `ItemDataUpload::Text` for JSON content
//! - **GeoJSON items**: Create and populate GeoJSON items with feature data
//! - **Web Map items**: Upload Web Map JSON definitions
//! - **Data verification**: Download and validate uploaded content
//! - **Round-trip integrity**: Ensure data matches after upload/download cycle
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
//! - **GeoJSON**: Geographic feature collections
//! - **Web Map**: Map configuration and layer definitions
//! - **JSON**: Generic JSON data items

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

    tracing::info!("📝 Portal Item Data - Text Uploads Example");
    tracing::info!("");

    // Load API key from environment
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Content)
        .expect("ARCGIS_CONTENT_KEY environment variable required");

    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new(PORTAL_URL, &client);

    // Demonstrate different text-based upload scenarios
    test_geojson_text_upload(&portal).await?;
    test_webmap_text_upload(&portal).await?;

    tracing::info!("\n✅ All text upload examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates uploading GeoJSON as text data.
async fn test_geojson_text_upload(portal: &PortalClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: GeoJSON Text Upload ===");
    tracing::info!("Create GeoJSON item and upload feature data as text");
    tracing::info!("");

    // Create a sample GeoJSON FeatureCollection
    let geojson = r#"{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [-122.4194, 37.7749]
      },
      "properties": {
        "name": "San Francisco",
        "population": 883305,
        "state": "California"
      }
    },
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [-118.2437, 34.0522]
      },
      "properties": {
        "name": "Los Angeles",
        "population": 3979576,
        "state": "California"
      }
    },
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [-121.8863, 37.3382]
      },
      "properties": {
        "name": "San Jose",
        "population": 1026908,
        "state": "California"
      }
    }
  ]
}"#;

    let original_size = geojson.len();
    tracing::info!("   GeoJSON size: {} bytes", original_size);
    tracing::info!("   Features: 3 cities");
    tracing::info!("");

    // STEP 1: Create GeoJSON item (metadata only)
    tracing::info!("📋 STEP 1: Creating GeoJSON item");
    let item_params = AddItemParams::new("California Cities (Text Upload Demo)", "GeoJson")
        .with_description("Demonstrates text-based GeoJSON upload using ItemDataUpload::Text")
        .with_tags(vec![
            "demo".to_string(),
            "geojson".to_string(),
            "text-upload".to_string(),
        ]);

    let add_result = portal.add_item(item_params).await?;
    let item_id = add_result.id().to_string();
    tracing::info!("✅ Created item: {}", item_id);
    tracing::info!("");

    // STEP 2: Upload GeoJSON data as text
    tracing::info!("📤 STEP 2: Uploading GeoJSON as text");
    tracing::info!("   Method: update_item_data_v2()");
    tracing::info!("   Upload Type: ItemDataUpload::Text");
    tracing::info!("");

    let upload = ItemDataUpload::Text(geojson.to_string());
    let update_result = portal.update_item_data_v2(&item_id, upload).await?;

    assert!(
        update_result.success(),
        "Failed to upload GeoJSON text: {:?}",
        update_result
    );
    tracing::info!("✅ Uploaded {} bytes as text", original_size);
    tracing::info!("");

    // STEP 3: Download and verify
    tracing::info!("📥 STEP 3: Downloading and verifying data");
    let retrieved_data = portal.get_item_data(&item_id).await?;
    let retrieved_size = retrieved_data.len();

    tracing::info!("   Retrieved size: {} bytes", retrieved_size);
    tracing::info!("   Original size:  {} bytes", original_size);

    // Verify data integrity
    assert!(retrieved_size > 0, "Retrieved data is empty!");

    // Parse to verify it's valid GeoJSON
    let retrieved_string = String::from_utf8(retrieved_data.to_vec())?;
    let parsed: geojson::FeatureCollection = serde_json::from_str(&retrieved_string)?;

    assert_eq!(
        parsed.features.len(),
        3,
        "Expected 3 features, got {}",
        parsed.features.len()
    );

    tracing::info!("✅ Valid GeoJSON with {} features", parsed.features.len());
    tracing::info!("");

    // STEP 4: Cleanup
    tracing::info!("🧹 STEP 4: Cleaning up");
    let delete_result = portal.delete_item(&item_id).await?;
    assert!(delete_result.success(), "Failed to delete item");
    tracing::info!("✅ Deleted item");
    tracing::info!("");

    tracing::info!("📊 GeoJSON Text Upload Summary:");
    tracing::info!("   ✓ Created GeoJSON item");
    tracing::info!(
        "   ✓ Uploaded {} bytes via ItemDataUpload::Text",
        original_size
    );
    tracing::info!(
        "   ✓ Retrieved and verified {} features",
        parsed.features.len()
    );
    tracing::info!("   ✓ Cleaned up resources");

    Ok(())
}

/// Demonstrates uploading Web Map JSON as text data.
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

    // STEP 1: Create Web Map item
    tracing::info!("📋 STEP 1: Creating Web Map item");
    let item_params = AddItemParams::new("Sample Web Map (Text Upload Demo)", "Web Map")
        .with_description("Demonstrates text-based Web Map upload using ItemDataUpload::Text")
        .with_tags(vec![
            "demo".to_string(),
            "webmap".to_string(),
            "text-upload".to_string(),
        ]);

    let add_result = portal.add_item(item_params).await?;
    let item_id = add_result.id().to_string();
    tracing::info!("✅ Created item: {}", item_id);
    tracing::info!("");

    // STEP 2: Upload Web Map JSON as text
    tracing::info!("📤 STEP 2: Uploading Web Map JSON as text");
    tracing::info!("   Method: update_item_data_v2()");
    tracing::info!("   Upload Type: ItemDataUpload::Text");
    tracing::info!("");

    let upload = ItemDataUpload::Text(webmap_json.to_string());
    let update_result = portal.update_item_data_v2(&item_id, upload).await?;

    assert!(
        update_result.success(),
        "Failed to upload Web Map JSON: {:?}",
        update_result
    );
    tracing::info!("✅ Uploaded {} bytes as text", original_size);
    tracing::info!("");

    // STEP 3: Download and verify
    tracing::info!("📥 STEP 3: Downloading and verifying data");
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

    // STEP 4: Cleanup
    tracing::info!("🧹 STEP 4: Cleaning up");
    let delete_result = portal.delete_item(&item_id).await?;
    assert!(delete_result.success(), "Failed to delete item");
    tracing::info!("✅ Deleted item");
    tracing::info!("");

    tracing::info!("📊 Web Map Text Upload Summary:");
    tracing::info!("   ✓ Created Web Map item");
    tracing::info!(
        "   ✓ Uploaded {} bytes via ItemDataUpload::Text",
        original_size
    );
    tracing::info!("   ✓ Retrieved and verified map definition");
    tracing::info!("   ✓ Cleaned up resources");

    Ok(())
}

/// Prints best practices for text-based uploads.
fn print_best_practices() {
    tracing::info!("\n💡 Text Upload Best Practices:");
    tracing::info!("   - Use ItemDataUpload::Text for JSON-structured items");
    tracing::info!("   - Suitable for: GeoJSON, Web Maps, JSON config");
    tracing::info!("   - Content is sent directly as 'text' parameter");
    tracing::info!("   - No file encoding overhead vs File uploads");
    tracing::info!("");
    tracing::info!("🎯 When to Use Text vs File:");
    tracing::info!("   Text:  JSON content, Web Maps, GeoJSON text");
    tracing::info!("   File:  Binary files, CSVs, images, PDFs, packages");
    tracing::info!("   Url:   External service references");
    tracing::info!("");
    tracing::info!("⚠️  Important Notes:");
    tracing::info!("   - Text uploads send JSON as string parameter");
    tracing::info!("   - File uploads use multipart encoding");
    tracing::info!("   - Choose based on item type and data format");
    tracing::info!("   - Both methods support full round-trip integrity");
}
