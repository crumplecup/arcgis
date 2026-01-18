//! 📦 Portal Content Management - Complete Content Lifecycle
//!
//! Demonstrates the full lifecycle of managing content in ArcGIS Online/Portal:
//! Search for existing datasets, create new items with rich metadata, update content,
//! create collaboration groups, and share with your team!
//!
//! # What You'll Learn
//!
//! - **Content search**: Discover items using Lucene query syntax
//! - **Item creation**: Upload GeoJSON and create feature layers
//! - **Metadata management**: Add rich descriptions, tags, and thumbnails
//! - **Group collaboration**: Create project groups for team work
//! - **Sharing workflows**: Control access (private, org, groups, public)
//! - **Builder patterns**: Construct complex parameters elegantly
//!
//! # Prerequisites
//!
//! - API key with content creation permissions OR
//! - OAuth2 credentials (user account with publish privileges)
//! - Set `ARCGIS_API_KEY` in `.env` file
//!
//! # Running
//!
//! ```bash
//! cargo run --example portal_content_management
//!
//! # With debug logging to see all API interactions:
//! RUST_LOG=debug cargo run --example portal_content_management
//! ```
//!
//! # Cost Awareness
//!
//! ⚠️ This example creates actual items in your ArcGIS Online account.
//! Items count toward your storage quota. Clean up test items when done!

use arcgis::{
    AddItemParams, ArcGISClient, ApiKeyAuth, CreateGroupParams, PortalClient, SearchParameters,
    SharingParameters, UpdateItemParams,
};
use anyhow::Context;

/// ArcGIS Online Portal URL (SaaS)
const PORTAL_URL: &str = "https://www.arcgis.com/sharing/rest";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("📦 ArcGIS Portal Content Management Examples");
    tracing::info!("Demonstrating complete content lifecycle workflows");

    // Create authenticated client
    let auth = ApiKeyAuth::from_env().context("Failed to load API key from environment")?;
    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new(PORTAL_URL, &client);

    // Example 1: Search for Existing Content
    tracing::info!("\n=== Example 1: Searching for Content ===");
    tracing::info!("Find feature services related to 'parks'");

    let search_params = SearchParameters::new("type:\"Feature Service\" AND tags:parks")
        .with_pagination(1, 10); // Start at 1, get 10 results

    tracing::debug!("Sending search request");
    let search_results = portal
        .search(search_params)
        .await
        .context("Failed to search portal items")?;

    tracing::info!(
        total_found = search_results.total(),
        returned = search_results.results().len(),
        "✅ Search completed"
    );

    tracing::info!("📊 Top results:");
    for (i, item) in search_results.results().iter().take(5).enumerate() {
        tracing::info!(
            "   {}. {} ({})",
            i + 1,
            item.title(),
            item.item_type()
        );
        tracing::debug!(item_id = %item.id(), owner = %item.owner(), "Item details");
    }

    // Example 2: Create a New Item
    tracing::info!("\n=== Example 2: Creating a New Item ===");
    tracing::info!("Upload a GeoJSON file as a new item");

    // Sample GeoJSON with a point feature
    let geojson_data = serde_json::json!({
        "type": "FeatureCollection",
        "features": [{
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [-122.4194, 37.7749] // San Francisco
            },
            "properties": {
                "name": "Sample Location",
                "description": "A test point in San Francisco"
            }
        }]
    });

    let add_params = AddItemParams::new("My Sample GeoJSON", "GeoJson")
        .with_description("A sample GeoJSON dataset created via the ArcGIS Rust SDK")
        .with_snippet("Demonstrates item creation with rich metadata")
        .with_tags(vec![
            "sample".to_string(),
            "demo".to_string(),
            "rust-sdk".to_string(),
        ])
        .with_type_keywords(vec!["Data".to_string(), "geojson".to_string()])
        .with_access("private".to_string()); // Private by default

    tracing::debug!(
        title = %add_params.title(),
        item_type = %add_params.item_type(),
        "Creating item"
    );

    let add_result = portal
        .add_item(add_params)
        .await
        .context("Failed to create item")?;

    if *add_result.success() {
        tracing::info!(
            item_id = %add_result.id(),
            "✅ Item created successfully!"
        );

        let created_item_id = add_result.id().to_string();

        // Upload the actual GeoJSON data
        tracing::debug!("Uploading GeoJSON data");
        let data_bytes = serde_json::to_vec(&geojson_data)
            .context("Failed to serialize GeoJSON")?;

        let update_result = portal
            .update_item_data(&created_item_id, data_bytes)
            .await
            .context("Failed to upload item data")?;

        if *update_result.success() {
            tracing::info!("✅ GeoJSON data uploaded successfully");
        }

        // Example 3: Update Item Metadata
        tracing::info!("\n=== Example 3: Updating Item Metadata ===");
        tracing::info!("Add more detailed information to the item");

        let update_params = UpdateItemParams::new()
            .with_description(
                "Updated description: This GeoJSON dataset demonstrates the complete \
                content management workflow using the ArcGIS Rust SDK. It includes \
                sample point features and rich metadata."
            )
            .with_snippet("Enhanced with detailed workflow documentation")
            .with_tags(vec![
                "sample".to_string(),
                "demo".to_string(),
                "rust-sdk".to_string(),
                "updated".to_string(),
            ]);

        tracing::debug!(item_id = %created_item_id, "Updating metadata");
        let update_meta_result = portal
            .update_item(&created_item_id, update_params)
            .await
            .context("Failed to update item metadata")?;

        if *update_meta_result.success() {
            tracing::info!("✅ Metadata updated successfully");
        }

        // Verify the update by fetching the item
        tracing::debug!("Verifying changes");
        let updated_item = portal
            .get_item(&created_item_id)
            .await
            .context("Failed to retrieve updated item")?;

        tracing::info!("📋 Item details:");
        tracing::info!("   Title: {}", updated_item.title());
        tracing::info!("   Type: {}", updated_item.item_type());
        tracing::info!("   Owner: {}", updated_item.owner());
        if let Some(desc) = updated_item.description() {
            tracing::info!("   Description: {}...", desc.chars().take(80).collect::<String>());
        }
        if !updated_item.tags().is_empty() {
            tracing::info!("   Tags: {}", updated_item.tags().join(", "));
        }

        // Example 4: Create a Collaboration Group
        tracing::info!("\n=== Example 4: Creating a Project Group ===");
        tracing::info!("Set up a group for team collaboration");

        let group_params = CreateGroupParams::new("Rust SDK Sample Project")
            .with_description(
                "A collaboration group for testing ArcGIS Rust SDK portal operations"
            )
            .with_snippet("Demonstrates group creation and sharing workflows")
            .with_tags(vec![
                "rust-sdk".to_string(),
                "sample".to_string(),
                "collaboration".to_string(),
            ])
            .with_access("org".to_string()); // Visible to organization

        tracing::debug!(title = %group_params.title(), "Creating group");
        let group_result = portal
            .create_group(group_params)
            .await
            .context("Failed to create group")?;

        if *group_result.success() {
            if let Some(group_id) = group_result.group_id() {
                tracing::info!(
                    group_id = %group_id,
                    "✅ Group created successfully!"
                );

                // Example 5: Share Item with Group
                tracing::info!("\n=== Example 5: Sharing Content ===");
                tracing::info!("Share the item with the project group");

                let share_params = SharingParameters::new()
                    .with_groups(vec![group_id.to_string()])
                    .with_org(true); // Also share with organization

                tracing::debug!(
                    item_id = %created_item_id,
                    group_id = %group_id,
                    "Sharing item"
                );

                let share_result = portal
                    .share_item(&created_item_id, share_params)
                    .await
                    .context("Failed to share item")?;

                if *share_result.success() {
                    tracing::info!("✅ Item shared with group and organization");
                    tracing::info!("   Group members can now access this item");
                }

                // Clean up: Delete the group (cleanup test data)
                tracing::info!("\n💡 Cleanup tip:");
                tracing::info!("   To delete test group: portal.delete_group(\"{}\").await", group_id);
            } else {
                tracing::warn!("⚠️  Group created but no ID returned");
            }
        } else {
            tracing::warn!("⚠️  Group creation failed");
        }

        // Clean up: Delete the test item (uncomment to actually delete)
        tracing::info!("\n💡 Cleanup tip:");
        tracing::info!("   To delete test item: portal.delete_item(\"{}\").await", created_item_id);
        tracing::info!("   Item URL: https://www.arcgis.com/home/item.html?id={}", created_item_id);
    } else {
        tracing::error!("❌ Item creation failed");
    }

    // Summary and Best Practices
    tracing::info!("\n✅ Portal content management examples completed!");
    tracing::info!("💡 Content Management Best Practices:");
    tracing::info!("   - Use rich metadata: tags, descriptions, snippets help discoverability");
    tracing::info!("   - Start private: Create items as private, then share deliberately");
    tracing::info!("   - Organize with groups: Use groups for project-based collaboration");
    tracing::info!("   - Clean up test data: Delete experimental items to manage quota");
    tracing::info!("   - Version control metadata: Track item updates in your workflow");
    tracing::info!("   - Use type keywords: Help Portal categorize and filter content");
    tracing::info!("⚠️  Note: Created items persist in your account - remember to clean up!");

    Ok(())
}
