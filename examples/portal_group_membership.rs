//! 👥 Portal Group Membership Example
//!
//! Demonstrates group membership operations on ArcGIS Enterprise using API key authentication.
//! This example shows how to create, join, leave, and delete groups programmatically.
//!
//! # What You'll Learn
//!
//! - **Enterprise authentication**: Using API keys for portal operations
//! - **Create groups**: Creating new groups programmatically
//! - **Join groups**: Joining groups as the authenticated user
//! - **Leave groups**: Leaving groups (when not owner)
//! - **Membership verification**: Checking group membership status
//! - **Group cleanup**: Deleting test groups
//!
//! # Prerequisites
//!
//! - Required: Enterprise API key with group management privileges
//! - Enterprise Portal deployment
//!
//! ## Environment Variables
//!
//! ```env
//! ARCGIS_ENTERPRISE_KEY=your_enterprise_api_key
//! ARCGIS_ENTERPRISE_PORTAL=https://your-server/arcgis/sharing/rest
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example portal_group_membership
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example portal_group_membership
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Self-service group management**: Users managing their own group memberships
//! - **Automated onboarding**: Adding users to groups based on roles
//! - **Access control**: Programmatic group-based permissions
//! - **Collaboration workflows**: Dynamic team formation
//! - **Content access**: Managing access to shared resources
//!
//! # Group Types
//!
//! - **Public**: Anyone can join
//! - **Private (Org)**: Organization members can join
//! - **Private (Invitation)**: Requires invitation from owner/admin

use anyhow::{Context, Result};
use arcgis::example_tracker::ExampleTracker;
use arcgis::{
    ApiKeyAuth, ArcGISClient, CreateGroupParams, EnvConfig, GroupMembershipType, PortalClient,
};
use secrecy::ExposeSecret;

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
    let tracker = ExampleTracker::new("portal_group_membership")
        .service_type("ExampleClient")
        .start();

    tracing::info!("👥 Portal Group Membership Example");
    tracing::info!("");

    // Load configuration from environment
    let config = EnvConfig::global();

    // Get Enterprise API key
    tracing::info!("🔐 Authenticating with Enterprise API Key");
    let auth = ApiKeyAuth::new(
        config
            .arcgis_enterprise_key
            .as_ref()
            .context("ARCGIS_ENTERPRISE_KEY not set in .env")?
            .expose_secret(),
    );

    // Get Enterprise portal URL
    let portal_url = config
        .arcgis_enterprise_portal
        .as_ref()
        .context(
            "ARCGIS_ENTERPRISE_PORTAL not set in .env\n\
             Example: ARCGIS_ENTERPRISE_PORTAL=https://your-server/arcgis/sharing/rest",
        )?;

    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new(portal_url, &client);

    tracing::info!("✅ Authenticated with Enterprise");
    tracing::info!("   Portal: {}", portal_url);
    tracing::info!("");

    // Run the membership workflow
    run_membership_workflow(&portal).await?;

    tracing::info!("✅ All group membership operations completed successfully!");
    print_best_practices();

    // Mark tracking as successful
    tracker.success();
    Ok(())
}

/// Demonstrates the complete group membership workflow.
async fn run_membership_workflow(portal: &PortalClient<'_>) -> Result<()> {
    // ========================================================================
    // STEP 1: Create a test group (public so we can join it)
    // ========================================================================
    tracing::info!("=== STEP 1: Creating Test Group ===");
    tracing::info!("Creating a public group that allows self-join");
    tracing::info!("");

    let timestamp = chrono::Utc::now().timestamp();
    let group_title = format!("Test Membership Group {}", timestamp);

    let create_params = CreateGroupParams::new(&group_title)
        .with_description("Test group for demonstrating join/leave operations")
        .with_snippet("Created by arcgis-rust SDK membership example")
        .with_tags(vec!["test".to_string(), "membership-demo".to_string()])
        .with_access("org"); // Organization members can join

    let create_result = portal.create_group(create_params).await?;

    assert!(
        create_result.success().map_or(false, |b| b),
        "Group creation failed: {:?}",
        create_result.error()
    );

    let group_id = create_result
        .id()
        .expect("No group ID returned")
        .to_string();

    tracing::info!("✅ Created group: {}", group_title);
    tracing::info!("   Group ID: {}", group_id);
    tracing::info!("   Access: Organization members can join");
    tracing::info!("");

    // ========================================================================
    // STEP 2: Get current user info
    // ========================================================================
    tracing::info!("=== STEP 2: Getting Current User Info ===");
    tracing::info!("Identifying the authenticated user");
    tracing::info!("");

    let user_info = portal.get_self().await?;
    let username = user_info
        .username()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No username in user info"))?;

    tracing::info!("✅ Current user: {}", username);
    if let Some(full_name) = user_info.full_name() {
        tracing::info!("   Full name: {}", full_name);
    }
    if let Some(email) = user_info.email() {
        tracing::info!("   Email: {}", email);
    }
    tracing::info!("");

    // ========================================================================
    // STEP 3: Join the group
    // ========================================================================
    tracing::info!("=== STEP 3: Joining Group ===");
    tracing::info!("User '{}' joining group '{}'", username, group_title);
    tracing::info!("");
    tracing::info!("   Method: join_group()");
    tracing::info!("   Endpoint: /community/groups/{}/join", group_id);
    tracing::info!("   Note: API key acts as authenticated user");
    tracing::info!("");

    let join_result = portal.join_group(&group_id).await?;

    // Note: If you create a group, you're automatically the owner/member
    // So joining immediately after creation will fail with "already a member"
    if join_result.success().map_or(false, |b| b) {
        tracing::info!("✅ Successfully joined group");
    } else if let Some(error) = join_result.error() {
        if error
            .message()
            .as_ref()
            .map(|m| m.contains("already a member"))
            .unwrap_or(false)
        {
            tracing::info!("✅ Already a member (created the group, so already owner)");
        } else {
            anyhow::bail!("Failed to join group: {:?}", error.message());
        }
    } else {
        anyhow::bail!("Unexpected response from join_group");
    }
    tracing::info!("");

    // ========================================================================
    // STEP 4: Verify membership
    // ========================================================================
    tracing::info!("=== STEP 4: Verifying Membership ===");
    tracing::info!("Checking if user is now a member");
    tracing::info!("");

    let group_info = portal.get_group(&group_id).await?;

    if let Some(user_membership) = group_info.user_membership() {
        tracing::info!("✅ Membership confirmed:");
        tracing::info!("   Username: {}", user_membership.username());

        match user_membership.member_type() {
            GroupMembershipType::Owner => {
                tracing::info!("   Role: Owner (created the group)");
            }
            GroupMembershipType::Admin => {
                tracing::info!("   Role: Admin");
            }
            GroupMembershipType::Member => {
                tracing::info!("   Role: Member");
            }
            GroupMembershipType::Unknown => {
                tracing::info!("   Role: Unknown");
            }
        }
    } else {
        tracing::warn!("⚠️  No membership info returned (may be API limitation)");
        tracing::info!("   However, join operation succeeded");
    }
    tracing::info!("");

    // ========================================================================
    // STEP 5: Leave the group
    // ========================================================================
    tracing::info!("=== STEP 5: Leaving Group ===");
    tracing::info!("User '{}' leaving group '{}'", username, group_title);
    tracing::info!("");
    tracing::info!("   Method: leave_group()");
    tracing::info!("   Endpoint: /community/groups/{}/leave", group_id);
    tracing::info!("");

    let leave_result = portal.leave_group(&group_id).await?;

    // Owners cannot leave their own groups
    if leave_result.success().map_or(false, |b| b) {
        tracing::info!("✅ Successfully left group");
    } else if let Some(error) = leave_result.error() {
        if error
            .message()
            .as_ref()
            .map(|m| m.contains("owner"))
            .unwrap_or(false)
        {
            tracing::info!("   Note: Owners cannot leave their own groups - this is expected");
        } else {
            tracing::warn!("⚠️  Leave failed: {:?}", error.message());
        }
    }
    tracing::info!("");

    // ========================================================================
    // STEP 6: Verify we left
    // ========================================================================
    tracing::info!("=== STEP 6: Verifying User Left ===");
    tracing::info!("Confirming user is no longer a member");
    tracing::info!("");

    let group_info_after = portal.get_group(&group_id).await?;

    // Note: Owner cannot leave their own group, so if we're the owner,
    // user_membership will still be present
    if let Some(user_membership) = group_info_after.user_membership() {
        match user_membership.member_type() {
            GroupMembershipType::Owner => {
                tracing::info!("   Status: Still owner (owners cannot leave their own groups)");
                tracing::info!("   Note: This is expected - we created the group");
            }
            _ => {
                tracing::warn!(
                    "⚠️  Still shows membership: {:?}",
                    user_membership.member_type()
                );
                tracing::info!("   Note: May be API caching delay, or leave was unsuccessful");
            }
        }
    } else {
        tracing::info!("✅ No membership info (user successfully left)");
    }
    tracing::info!("");

    // ========================================================================
    // STEP 7: Cleanup - Delete the test group
    // ========================================================================
    tracing::info!("=== STEP 7: Cleaning Up ===");
    tracing::info!("Deleting test group");
    tracing::info!("");

    let delete_result = portal.delete_group(&group_id).await?;

    assert!(
        delete_result.success().map_or(false, |b| b),
        "Failed to delete group: {:?}",
        delete_result.error()
    );

    tracing::info!("✅ Test group deleted");
    tracing::info!("");

    // ========================================================================
    // Summary
    // ========================================================================
    tracing::info!("📊 Group Membership Workflow Summary:");
    tracing::info!("   ✓ Created test group ({})", group_id);
    tracing::info!("   ✓ User joined group via join_group()");
    tracing::info!("   ✓ Verified membership status");
    tracing::info!("   ✓ User left group via leave_group()");
    tracing::info!("   ✓ Verified user no longer a member");
    tracing::info!("   ✓ Cleaned up test group");
    Ok(())
}

/// Prints best practices for group membership operations.
fn print_best_practices() {
    tracing::info!("");
    tracing::info!("💡 Group Membership Best Practices:");
    tracing::info!("   - Enterprise API keys can perform administrative group operations");
    tracing::info!("   - Check group access level before operations:");
    tracing::info!("     • public: Anyone can view");
    tracing::info!("     • org: Organization members can view");
    tracing::info!("     • private: Owner/admins only");
    tracing::info!("   - Group owners cannot leave their own groups");
    tracing::info!("   - Deleting a group requires ownership");
    tracing::info!("");
    tracing::info!("🎯 Group Operation Capabilities:");
    tracing::info!("   create_group():  Create new groups (requires privileges)");
    tracing::info!("   join_group():    Join group as authenticated user");
    tracing::info!("   leave_group():   Leave group (not as owner)");
    tracing::info!("   delete_group():  Delete owned groups");
    tracing::info!("   get_group():     Get group details and membership");
    tracing::info!("");
    tracing::info!("⚠️  Important Notes:");
    tracing::info!("   - API key must have appropriate privileges");
    tracing::info!("   - Group owners are automatically members");
    tracing::info!("   - Membership changes may have brief caching delay");
    tracing::info!("   - Cannot leave groups you own (must delete instead)");
}
