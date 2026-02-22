//! 📦 Portal Content Discovery - Search and Explore ArcGIS Online
//!
//! Demonstrates searching and exploring public content in ArcGIS Online/Portal.
//! Learn how to discover datasets, retrieve metadata, search groups, and build
//! powerful Lucene queries to find exactly what you need!
//!
//! # What You'll Learn
//!
//! - **Content search**: Discover items using Lucene query syntax
//! - **Advanced filters**: Search by type, tags, owner, and more
//! - **Pagination**: Handle large result sets efficiently
//! - **Item metadata**: Retrieve detailed information about items
//! - **Group discovery**: Find public and organizational groups
//! - **Builder patterns**: Construct complex search parameters
//!
//! # Prerequisites
//!
//! - OAuth2 client credentials (for org content) OR API key (for public content)
//! - Set `ARCGIS_CLIENT_ID` + `ARCGIS_CLIENT_SECRET` OR `ARCGIS_API_KEY` in `.env`
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
//! # Real-World Use Cases
//!
//! - **Data discovery**: Find public datasets for analysis or integration
//! - **Content inventory**: Catalog organizational assets and resources
//! - **Group management**: Discover and join collaborative workspaces
//! - **Metadata exploration**: Understand dataset characteristics before use
//! - **Search interfaces**: Build custom portals with advanced filtering
//!
//! # Note on Content Creation
//!
//! Creating, updating, and managing content requires **user authentication**
//! via OAuth2 PKCE flow (browser-based login), not client credentials or API keys.
//! This example focuses on discovery and read-only operations that work with
//! all authentication methods.

use anyhow::{Context, Result};
use arcgis::{
    ArcGISClient, ClientCredentialsAuth, GroupSearchParameters, PortalClient, SearchParameters,
    SortOrder,
};

/// ArcGIS Online Portal URL (SaaS)
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

    tracing::info!("📦 ArcGIS Portal Content Discovery Examples");
    tracing::info!("Demonstrating search and exploration workflows");

    // Create authenticated client (automatically loads .env)
    tracing::debug!("Creating authenticated client");
    let auth = ClientCredentialsAuth::from_env()
        .context("Failed to load OAuth credentials from environment")?;
    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new(PORTAL_URL, &client);

    // Demonstrate portal content discovery operations
    demonstrate_basic_search(&portal).await?;
    demonstrate_advanced_query(&portal).await?;
    demonstrate_item_details(&portal).await?;
    demonstrate_group_discovery(&portal).await?;
    demonstrate_pagination(&portal).await?;

    tracing::info!("\n✅ Portal content discovery examples completed!");
    print_best_practices();

    Ok(())
}

/// Demonstrates basic content search using Lucene query syntax.
async fn demonstrate_basic_search(portal: &PortalClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Basic Content Search ===");
    tracing::info!("Find feature services related to 'parks'");

    let search_params =
        SearchParameters::new("type:\"Feature Service\" AND tags:parks").with_pagination(1, 10); // Start at 1, get 10 results

    tracing::debug!("Sending search request");
    let search_results = portal
        .search(search_params)
        .await
        .context("Failed to search portal items")?;

    // Verify search returned results
    assert!(
        *search_results.total() > 0,
        "Search should find feature services tagged with 'parks'"
    );
    assert!(
        !search_results.results().is_empty(),
        "Search results should not be empty"
    );

    tracing::info!(
        total_found = search_results.total(),
        returned = search_results.results().len(),
        "✅ Search completed"
    );

    tracing::info!("📊 Top parks-related feature services:");
    for (i, item) in search_results.results().iter().take(5).enumerate() {
        tracing::info!("   {}. {} ({})", i + 1, item.title(), item.owner());
        tracing::debug!(item_id = %item.id(), item_type = %item.item_type(), "Item details");
    }

    Ok(())
}

/// Demonstrates advanced search with sorting by modification date.
async fn demonstrate_advanced_query(portal: &PortalClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Advanced Query with Sorting ===");
    tracing::info!("Find recent web maps, sorted by modification date");

    let recent_maps = SearchParameters::new("type:\"Web Map\"")
        .with_pagination(1, 5)
        .with_sort("modified", SortOrder::Desc); // Most recently modified first

    let maps_result = portal
        .search(recent_maps)
        .await
        .context("Failed to search web maps")?;

    // Verify web map search returned results
    assert!(*maps_result.total() > 0, "Search should find web maps");
    assert!(
        !maps_result.results().is_empty(),
        "Web map results should not be empty"
    );

    tracing::info!(
        total_found = maps_result.total(),
        "✅ Found {} total web maps",
        maps_result.total()
    );

    tracing::info!("📍 Recently modified web maps:");
    for (i, item) in maps_result.results().iter().enumerate() {
        tracing::info!("   {}. {}", i + 1, item.title());
        tracing::debug!(
            modified = %item.modified(),
            "Modified timestamp"
        );
    }

    Ok(())
}

/// Demonstrates retrieving detailed item metadata.
async fn demonstrate_item_details(portal: &PortalClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Retrieve Item Metadata ===");
    tracing::info!("Get detailed information about a specific item");

    // First, find an item to get details for
    let search_params =
        SearchParameters::new("type:\"Feature Service\" AND tags:parks").with_pagination(1, 1);

    let search_results = portal
        .search(search_params)
        .await
        .context("Failed to search portal items")?;

    if let Some(first_item) = search_results.results().first() {
        let item_id = first_item.id();
        tracing::debug!(item_id = %item_id, "Fetching item details");

        let item_details = portal
            .get_item(item_id)
            .await
            .context("Failed to retrieve item details")?;

        // Verify item details were retrieved
        assert!(!item_details.title().is_empty(), "Item should have a title");
        assert!(
            !item_details.item_type().is_empty(),
            "Item should have a type"
        );
        assert!(
            !item_details.owner().is_empty(),
            "Item should have an owner"
        );

        tracing::info!("📋 Item Details:");
        tracing::info!("   Title: {}", item_details.title());
        tracing::info!("   Type: {}", item_details.item_type());
        tracing::info!("   Owner: {}", item_details.owner());

        if let Some(desc) = item_details.description() {
            let preview = desc.chars().take(100).collect::<String>();
            tracing::info!("   Description: {}...", preview);
        }

        if !item_details.tags().is_empty() {
            tracing::info!("   Tags: {}", item_details.tags().join(", "));
        }

        if let Some(url) = item_details.url() {
            tracing::info!("   URL: {}", url);
        }

        tracing::debug!(
            id = %item_details.id(),
            access = %item_details.access(),
            "Full item metadata retrieved"
        );
    }

    Ok(())
}

/// Demonstrates finding and exploring public groups.
async fn demonstrate_group_discovery(portal: &PortalClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 4: Group Discovery ===");
    tracing::info!("Find public groups related to 'open data'");

    let group_search =
        GroupSearchParameters::new("title:\"open data\" AND access:public").with_pagination(1, 5);

    let group_results = portal
        .search_groups(group_search)
        .await
        .context("Failed to search groups")?;

    // Verify group search returned results
    assert!(
        *group_results.total() > 0,
        "Search should find public groups related to 'open data'"
    );
    assert!(
        !group_results.results().is_empty(),
        "Group search results should not be empty"
    );

    tracing::info!(
        total_found = group_results.total(),
        returned = group_results.results().len(),
        "✅ Found {} total groups",
        group_results.total()
    );

    tracing::info!("👥 Open data groups:");
    for (i, group) in group_results.results().iter().take(5).enumerate() {
        tracing::info!("   {}. {}", i + 1, group.title());
        tracing::info!("      Owner: {}", group.owner());
        if let Some(desc) = group.description() {
            let preview = desc.chars().take(60).collect::<String>();
            tracing::info!("      Description: {}...", preview);
        }
        tracing::debug!(group_id = %group.id(), access = %group.access(), "Group details");
    }

    Ok(())
}

/// Demonstrates pagination for handling large result sets.
async fn demonstrate_pagination(portal: &PortalClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 5: Pagination Pattern ===");
    tracing::info!("Retrieve multiple pages of search results");

    let page_size = 5;
    let mut start = 1;
    let max_pages = 3;

    tracing::info!("Fetching {} pages of imagery results:", max_pages);

    for page_num in 1..=max_pages {
        let page_params =
            SearchParameters::new("type:\"Image Service\"").with_pagination(start, page_size);

        let page_results = portal
            .search(page_params)
            .await
            .context("Failed to search page")?;

        // Verify pagination returns results
        assert!(
            !page_results.results().is_empty(),
            "Pagination should return results on page {}",
            page_num
        );

        tracing::info!(
            "📄 Page {} (items {}-{}):",
            page_num,
            start,
            start + page_size - 1
        );
        for (i, item) in page_results.results().iter().enumerate() {
            tracing::info!("   {}. {}", i + 1, item.title());
        }

        start += page_size;

        if page_results.results().len() < page_size as usize {
            tracing::info!("   (Last page)");
            break;
        }
    }

    Ok(())
}

/// Prints best practices for portal content search and discovery.
fn print_best_practices() {
    tracing::info!("\n💡 Search Best Practices:");
    tracing::info!("   - Use Lucene query syntax for powerful filtering");
    tracing::info!("   - Combine type filters: type:\"Feature Service\" AND tags:parks");
    tracing::info!("   - Sort by 'modified', 'created', 'title', 'owner', or 'numviews'");
    tracing::info!("   - Paginate large result sets (max 100 per page)");
    tracing::info!("   - Cache item metadata to reduce API calls");
    tracing::info!("   - Search is case-insensitive by default");
    tracing::info!("");
    tracing::info!("📚 Common Search Patterns:");
    tracing::info!("   - By owner: owner:username");
    tracing::info!("   - By org: orgid:abc123");
    tracing::info!("   - Public only: access:public");
    tracing::info!("   - Date range: modified:[NOW-7DAYS TO NOW]");
    tracing::info!("   - Multiple tags: tags:(transportation AND roads)");
    tracing::info!("");
    tracing::info!("🎯 Item Types:");
    tracing::info!("   - Feature Service: Editable vector data layers");
    tracing::info!("   - Map Service: Cached map tiles (fast display)");
    tracing::info!("   - Image Service: Raster imagery and elevation");
    tracing::info!("   - Web Map: Configured map with layers and symbology");
    tracing::info!("   - Web Scene: 3D map configurations");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Request only needed fields to reduce payload size");
    tracing::info!("   - Use pagination (don't fetch all results at once)");
    tracing::info!("   - Implement client-side caching for frequently accessed items");
    tracing::info!("   - Narrow searches with specific type and tag filters");
    tracing::info!("   - Consider rate limits when building batch operations");
}
