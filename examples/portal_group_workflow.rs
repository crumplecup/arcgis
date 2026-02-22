//! 👥 Portal Group Management - Complete Workflow
//!
//! Demonstrates the complete lifecycle of group management in ArcGIS Online/Enterprise,
//! including creation, item management, updates, and deletion.
//!
//! # What You'll Learn
//!
//! - **create_group**: Create new groups for organizing content
//! - **get_group**: Retrieve group details and membership
//! - **update_group**: Modify group metadata (title, description, access)
//! - **add_to_group**: Add items to a group
//! - **remove_from_group**: Remove items from a group
//! - **delete_group**: Clean up groups
//!
//! # Prerequisites
//!
//! - ArcGIS API Key with content management privileges
//! - Set ARCGIS_CONTENT_KEY in `.env` file
//!
//! # Running
//!
//! ```bash
//! cargo run --example portal_group_workflow
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example portal_group_workflow
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Content organization**: Group related items by project, theme, or department
//! - **Sharing workflows**: Share multiple items with a group instead of individually
//! - **Collaboration**: Create project teams with shared content access
//! - **Content discovery**: Make items discoverable through group membership
//! - **Permissions**: Use groups for fine-grained access control

use arcgis::{
    AddItemParams, ApiKeyAuth, ApiKeyTier, ArcGISClient, CreateGroupParams, DeleteItemResult,
    PortalClient, Result, ShareItemResult, UnshareItemResult, UpdateGroupParams,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("👥 Portal Group Management Workflow");
    tracing::info!("");

    // Authenticate
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Content)?;
    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &client);

    tracing::info!("✅ Authenticated with API key (ARCGIS_CONTENT_KEY)");

    // Execute the complete workflow
    let result = run_group_workflow(&portal).await;

    match result {
        Ok(_) => {
            tracing::info!("");
            tracing::info!("✅ All group workflow operations completed successfully!");
            print_best_practices();
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ Group workflow failed: {}", e);
            Err(e)
        }
    }
}

/// Executes the complete group lifecycle workflow.
async fn run_group_workflow(portal: &PortalClient<'_>) -> Result<()> {
    // Step 1: Create a test item (we need something to add to the group)
    tracing::info!("=== Step 1: Creating Test Item ===");
    tracing::info!("Creating a simple web map item to add to group");

    let item_params = AddItemParams::new("Test Web Map for Group Demo", "Web Map")
        .with_description("This item will be added to a group for testing")
        .with_tags(vec!["test".to_string(), "group-demo".to_string()])
        .with_text(r#"{"version":"2.0","baseMap":{"title":"Basemap"}}"#);

    let item_result = portal.add_item(item_params).await?;
    assert!(item_result.success(), "Item creation must succeed");

    let item_id = item_result.id();
    tracing::info!("✓ Created test item: {}", item_id);

    // Step 2: Create a test group
    tracing::info!("");
    tracing::info!("=== Step 2: Creating Group ===");
    tracing::info!("Creating a new group for organizing content");

    let timestamp = chrono::Utc::now().timestamp();
    let group_title = format!("Test Group {}", timestamp);

    let create_params = CreateGroupParams::new(&group_title)
        .with_description("A test group for demonstrating group management workflow")
        .with_snippet("Test group created by arcgis-rust SDK")
        .with_tags(vec!["test".to_string(), "sdk-demo".to_string()])
        .with_access("private"); // Private group

    let create_result = portal.create_group(create_params).await?;
    assert!(*create_result.success(), "Group creation must succeed");

    tracing::debug!(
        "Create result: success={}, id={:?}",
        create_result.success(),
        create_result.id()
    );

    let group_id = create_result
        .id()
        .expect("No group ID returned")
        .to_string();

    tracing::info!("✓ Created group: {} ({})", group_title, group_id);

    // Step 3: Get group details (verify creation)
    tracing::info!("");
    tracing::info!("=== Step 3: Getting Group Details ===");
    tracing::info!("Retrieving group metadata to verify creation");

    let group_info = portal.get_group(&group_id).await?;

    tracing::info!("✓ Retrieved group details:");
    tracing::info!("   ID: {}", group_info.id());
    tracing::info!("   Title: {}", group_info.title());
    tracing::info!("   Owner: {}", group_info.owner());
    tracing::info!(
        "   Description: {}",
        group_info.description().as_deref().unwrap_or("None")
    );
    tracing::info!("   Tags: {:?}", group_info.tags());

    assert!(
        group_info.title() == &group_title,
        "Group title must match: expected '{}', got '{}'",
        group_title,
        group_info.title()
    );

    // Step 4: Add item to group
    tracing::info!("");
    tracing::info!("=== Step 4: Adding Item to Group ===");
    tracing::info!("Adding the test item to the group");

    let add_result: ShareItemResult = portal.add_to_group(&group_id, item_id).await?;
    assert!(add_result.success(), "Adding item to group must succeed");

    tracing::info!("✓ Added item {} to group {}", item_id, group_id);

    // Step 5: Verify item was added
    tracing::info!("");
    tracing::info!("=== Step 5: Verifying Item in Group ===");
    tracing::info!("Getting group details to confirm item membership");

    // Note: GroupInfo doesn't include the list of items in the response
    // In a real application, you would search for items in the group
    // or use the Portal REST API's /community/groups/{groupId}/content endpoint
    let group_after_add = portal.get_group(&group_id).await?;
    tracing::info!("✓ Group retrieved after adding item");
    tracing::info!("   Group ID: {}", group_after_add.id());
    tracing::info!("   Note: Item membership verified through successful add operation");

    // Step 6: Update group metadata
    tracing::info!("");
    tracing::info!("=== Step 6: Updating Group ===");
    tracing::info!("Modifying group title and description");

    let updated_title = format!("Updated Test Group {}", timestamp);
    let update_params = UpdateGroupParams::new()
        .with_title(&updated_title)
        .with_description("This group description has been updated")
        .with_snippet("Updated snippet for test group");

    let update_result = portal.update_group(&group_id, update_params).await?;
    assert!(update_result.success(), "Group update must succeed");

    tracing::info!("✓ Updated group metadata");

    // Step 7: Verify update
    tracing::info!("");
    tracing::info!("=== Step 7: Verifying Update ===");
    tracing::info!("Getting group details to confirm changes");

    let group_after_update = portal.get_group(&group_id).await?;

    tracing::info!("✓ Retrieved updated group details:");
    tracing::info!("   New Title: {}", group_after_update.title());
    tracing::info!(
        "   New Description: {}",
        group_after_update
            .description()
            .as_deref()
            .unwrap_or("None")
    );

    assert!(
        group_after_update.title() == &updated_title,
        "Group title must be updated: expected '{}', got '{}'",
        updated_title,
        group_after_update.title()
    );

    // Step 8: Remove item from group
    tracing::info!("");
    tracing::info!("=== Step 8: Removing Item from Group ===");
    tracing::info!("Removing the test item from the group");

    let remove_result: UnshareItemResult = portal.remove_from_group(&group_id, item_id).await?;
    assert!(
        remove_result.success(),
        "Removing item from group must succeed"
    );

    tracing::info!("✓ Removed item {} from group {}", item_id, group_id);

    // Step 9: Verify removal
    tracing::info!("");
    tracing::info!("=== Step 9: Verifying Item Removal ===");

    let _group_after_remove = portal.get_group(&group_id).await?;
    tracing::info!("✓ Group retrieved after removing item");
    tracing::info!("   Note: Item removal verified through successful remove operation");

    // Step 10: Clean up - Delete item
    tracing::info!("");
    tracing::info!("=== Step 10: Cleaning Up - Deleting Item ===");

    let delete_item_result: DeleteItemResult = portal.delete_item(item_id).await?;
    assert!(delete_item_result.success(), "Item deletion must succeed");

    tracing::info!("✓ Deleted test item: {}", item_id);

    // Step 11: Clean up - Delete group
    tracing::info!("");
    tracing::info!("=== Step 11: Cleaning Up - Deleting Group ===");

    let delete_group_result = portal.delete_group(&group_id).await?;
    assert!(delete_group_result.success(), "Group deletion must succeed");

    tracing::info!("✓ Deleted test group: {}", group_id);

    Ok(())
}

/// Prints best practices for group management.
fn print_best_practices() {
    tracing::info!("");
    tracing::info!("💡 Group Management Best Practices:");
    tracing::info!("   - Use descriptive group names and descriptions");
    tracing::info!("   - Tag groups appropriately for discoverability");
    tracing::info!("   - Set appropriate access levels (private, org, public)");
    tracing::info!("   - Use groups to organize related content");
    tracing::info!("   - Clean up test groups to avoid clutter");
    tracing::info!("");
    tracing::info!("🔐 Access Levels:");
    tracing::info!("   • private: Only group members can see/access");
    tracing::info!("   • org: Organization members can see/request to join");
    tracing::info!("   • public: Anyone can see, members can access content");
    tracing::info!("");
    tracing::info!("📊 Group Sharing:");
    tracing::info!("   • Items can be shared with groups using share_item()");
    tracing::info!("   • Group membership controls access to shared items");
    tracing::info!("   • Use groups for project-based content organization");
    tracing::info!("");
    tracing::info!("⚠️ Note on User Operations:");
    tracing::info!("   • join_group/leave_group require OAuth user authentication");
    tracing::info!("   • API key auth is for content/group management only");
    tracing::info!("   • Use OAuth for user-based group membership operations");
}
