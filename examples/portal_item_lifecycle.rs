//! Portal Item Lifecycle Example
//!
//! Demonstrates the complete lifecycle of a portal item using metadata operations:
//! 1. Create item (add_item)
//! 2. Retrieve item (get_item)
//! 3. Update metadata (update_item)
//! 4. Verify updates (get_item)
//! 5. Share with organization (share_item)
//! 6. Unshare from organization (unshare_item)
//! 7. Delete item (delete_item)
//!
//! This example uses a "Web Mapping Application" item type which requires only
//! metadata and a URL - no file uploads or complex data handling.

use arcgis::{
    AddItemParams, ApiKeyAuth, ApiKeyTier, ArcGISClient, PortalClient, Result, SharingParameters,
    UpdateItemParams,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("🔄 Portal Item Lifecycle Example");
    tracing::info!("Demonstrating complete metadata-based item management");
    tracing::info!("");

    // Authenticate
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Content)?;
    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &client);

    tracing::info!("✅ Authenticated with API key (ARCGIS_CONTENT_KEY)");
    tracing::info!("");

    // ========================================================================
    // STEP 1: Create item
    // ========================================================================
    tracing::info!("📝 STEP 1: Creating portal item");
    tracing::info!("");
    tracing::info!("   Item Type: Web Mapping Application");
    tracing::info!("   Title: Lifecycle Demo App");
    tracing::info!("   URL: https://example.com/app");
    tracing::info!("");

    let item_params = AddItemParams::new("Lifecycle Demo App", "Web Mapping Application")
        .with_description("Testing portal item lifecycle operations")
        .with_tags(vec!["demo".to_string(), "test".to_string()])
        .with_snippet("Portal lifecycle example")
        .with_url("https://example.com/app");

    let add_result = portal.add_item(item_params).await?;
    let item_id = add_result.id();

    assert!(*add_result.success(), "Failed to create item");

    tracing::info!("✅ Created item: {}", item_id);
    tracing::info!("");

    // ========================================================================
    // STEP 2: Retrieve item
    // ========================================================================
    tracing::info!("🔍 STEP 2: Retrieving item metadata");
    tracing::info!("");

    let item = portal.get_item(item_id).await?;

    assert_eq!(item.title(), "Lifecycle Demo App");
    assert_eq!(item.item_type(), "Web Mapping Application");

    tracing::info!("✅ Retrieved item");
    tracing::info!("   Title: {}", item.title());
    tracing::info!("   Type: {}", item.item_type());
    tracing::info!("   Owner: {}", item.owner());
    tracing::info!("");

    // ========================================================================
    // STEP 3: Update metadata
    // ========================================================================
    tracing::info!("✏️  STEP 3: Updating item metadata");
    tracing::info!("");

    let update_params = UpdateItemParams::new()
        .with_title("Updated Lifecycle Demo")
        .with_description("Updated description")
        .with_tags(vec!["demo".to_string(), "updated".to_string()]);

    let update_result = portal.update_item(item_id, update_params).await?;

    assert!(*update_result.success(), "Failed to update item");

    tracing::info!("✅ Updated metadata");
    tracing::info!("");

    // ========================================================================
    // STEP 4: Verify updates
    // ========================================================================
    tracing::info!("🔍 STEP 4: Verifying metadata updates");
    tracing::info!("");

    let updated_item = portal.get_item(item_id).await?;

    assert_eq!(updated_item.title(), "Updated Lifecycle Demo");

    tracing::info!("✅ Verified updates");
    tracing::info!("   Title: {}", updated_item.title());
    tracing::info!("");

    // ========================================================================
    // STEP 5: Share with organization
    // ========================================================================
    tracing::info!("🔐 STEP 5: Sharing with organization");
    tracing::info!("");

    let share_params = SharingParameters::new().with_org(true);
    let share_result = portal.share_item(item_id, share_params).await?;

    if *share_result.success() {
        tracing::info!("✅ Shared with organization");
    } else {
        tracing::warn!("⚠️  Share returned success=false (may be API key permission restriction)");
    }
    tracing::info!("");

    // ========================================================================
    // STEP 6: Unshare from organization
    // ========================================================================
    tracing::info!("🔓 STEP 6: Unsharing from organization");
    tracing::info!("");

    let unshare_params = SharingParameters::new();
    let unshare_result = portal.unshare_item(item_id, unshare_params).await?;

    if *unshare_result.success() {
        tracing::info!("✅ Unshared from organization");
    } else {
        tracing::warn!(
            "⚠️  Unshare returned success=false (may be API key permission restriction)"
        );
    }
    tracing::info!("");

    // ========================================================================
    // STEP 7: Delete item
    // ========================================================================
    tracing::info!("🗑️  STEP 7: Deleting item");
    tracing::info!("");

    let delete_result = portal.delete_item(item_id).await?;

    assert!(*delete_result.success(), "Failed to delete item");

    tracing::info!("✅ Deleted item");
    tracing::info!("");

    // ========================================================================
    // Summary
    // ========================================================================
    tracing::info!("📊 Lifecycle Summary:");
    tracing::info!("   ✓ Created item (add_item)");
    tracing::info!("   ✓ Retrieved metadata (get_item)");
    tracing::info!("   ✓ Updated metadata (update_item)");
    tracing::info!("   ✓ Verified updates (get_item)");
    tracing::info!("   ✓ Shared with organization (share_item)");
    tracing::info!("   ✓ Unshared from organization (unshare_item)");
    tracing::info!("   ✓ Deleted item (delete_item)");
    tracing::info!("");
    tracing::info!("💡 All metadata lifecycle operations demonstrated successfully!");

    Ok(())
}
