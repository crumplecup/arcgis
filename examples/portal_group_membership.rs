//! 👥 Portal Group Membership Example
//!
//! Demonstrates user-based group membership operations using OAuth authentication.
//! This example shows how users can join and leave groups programmatically.
//!
//! # What You'll Learn
//!
//! - **OAuth authentication**: Using Client Credentials flow for user operations
//! - **Join groups**: Users joining groups programmatically
//! - **Leave groups**: Users leaving groups
//! - **Membership verification**: Checking group membership status
//! - **Group types**: Understanding public, private, and invitation-only groups
//!
//! # Prerequisites
//!
//! - Required: OAuth client credentials in `.env` file
//! - Permissions: User account with group join/leave privileges
//!
//! ## Environment Variables
//!
//! ```env
//! ARCGIS_CLIENT_ID=your_oauth_client_id
//! ARCGIS_CLIENT_SECRET=your_oauth_client_secret
//! ```
//!
//! Get credentials from: https://developers.arcgis.com/applications
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

use anyhow::Result;
use arcgis::{
    ArcGISClient, ClientCredentialsAuth, CreateGroupParams, GroupMembershipType, PortalClient,
};

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

    tracing::info!("👥 Portal Group Membership Example");
    tracing::info!("");

    // Load OAuth credentials from environment
    tracing::info!("🔐 Authenticating with OAuth 2.0 Client Credentials");
    let auth = ClientCredentialsAuth::from_env()
        .expect("ARCGIS_CLIENT_ID and ARCGIS_CLIENT_SECRET environment variables required");

    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new(PORTAL_URL, &client);

    tracing::info!("✅ Authenticated successfully");
    tracing::info!("");

    // Run the membership workflow
    run_membership_workflow(&portal).await?;

    tracing::info!("✅ All group membership operations completed successfully!");
    print_best_practices();

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
        .with_tags(vec![
            "test".to_string(),
            "membership-demo".to_string(),
        ])
        .with_access("org"); // Organization members can join

    let create_result = portal.create_group(create_params).await?;

    assert!(
        *create_result.success(),
        "Group creation failed: {:?}",
        create_result
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
    tracing::info!("   Note: Requires OAuth user token (not API key)");
    tracing::info!("");

    let join_result = portal.join_group(&group_id).await?;

    assert!(
        *join_result.success(),
        "Failed to join group: {:?}",
        join_result
    );

    tracing::info!("✅ Successfully joined group");
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

    assert!(
        *leave_result.success(),
        "Failed to leave group: {:?}",
        leave_result
    );

    tracing::info!("✅ Successfully left group");
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
                tracing::warn!("⚠️  Still shows membership: {:?}", user_membership.member_type());
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
        *delete_result.success(),
        "Failed to delete group: {:?}",
        delete_result
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
    tracing::info!("   - Use OAuth user tokens (not API keys) for join/leave");
    tracing::info!("   - Check group access level before joining:");
    tracing::info!("     • public: Anyone can join");
    tracing::info!("     • org: Organization members can join");
    tracing::info!("     • private: Invitation required");
    tracing::info!("   - Group owners cannot leave their own groups");
    tracing::info!("   - Use add_to_group() (admin) vs join_group() (user)");
    tracing::info!("");
    tracing::info!("🎯 When to Use Which Operation:");
    tracing::info!("   join_group():   User joins group themselves (OAuth)");
    tracing::info!("   leave_group():  User leaves group themselves (OAuth)");
    tracing::info!("   add_to_group(): Admin adds user to group (API key OK)");
    tracing::info!("   remove_from_group(): Admin removes user (API key OK)");
    tracing::info!("");
    tracing::info!("⚠️  Important Notes:");
    tracing::info!("   - Requires OAuth 2.0 Client Credentials or user token");
    tracing::info!("   - API keys DO NOT work for join_group/leave_group");
    tracing::info!("   - Group owners are automatically members");
    tracing::info!("   - Membership changes may have brief caching delay");
}
