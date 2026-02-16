//! 🗺️ Vector Tiles - Modern Web Mapping with MVT
//!
//! Demonstrates working with vector tile services for efficient, scalable web mapping.
//! Shows how to fetch tiles, styles, fonts, and sprites for building modern map applications.
//!
//! # What You'll Learn
//!
//! - **Vector tile fetching**: Download MVT (Mapbox Vector Tile) format tiles
//! - **Batch operations**: Fetch multiple tiles efficiently
//! - **Style retrieval**: Get Mapbox GL style JSON for rendering
//! - **Font glyphs**: Download font data for text labels
//! - **Sprite sheets**: Retrieve icon/symbol metadata and images
//! - **Practical patterns**: Build tile caches, inspect styles
//!
//! # Vector Tiles vs Raster Tiles
//!
//! **Vector Tiles (MVT):**
//! - Contains geometric data (points, lines, polygons)
//! - Styled dynamically on client
//! - Smaller file sizes
//! - Supports rotation, dynamic styling
//! - Modern standard for web mapping
//!
//! **Raster Tiles:**
//! - Pre-rendered images (PNG/JPG)
//! - Fixed style, larger files
//! - Simple to display
//! - Better for imagery/hillshades
//!
//! # Prerequisites
//!
//! - No authentication required (uses public basemap service)
//! - Internet connection for service access
//!
//! # Running
//!
//! ```bash
//! cargo run --example vector_tiles
//!
//! # With debug logging to see all requests:
//! RUST_LOG=debug cargo run --example vector_tiles
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Web mapping applications**: Leaflet, MapLibre, OpenLayers integration
//! - **Offline mapping**: Cache tiles for offline use
//! - **Custom basemaps**: Style vector tiles for branding
//! - **Mobile apps**: Efficient data transfer for mobile mapping
//! - **Multi-resolution displays**: Retina/HiDPI support
//! - **Dynamic theming**: Switch between light/dark modes
//!
//! # MVT Format
//!
//! Mapbox Vector Tiles (MVT) use Protocol Buffers for efficient encoding.
//! Each tile contains:
//! - **Layers**: Separate data layers (roads, buildings, water, etc.)
//! - **Features**: Geometric features with attributes
//! - **Metadata**: Tile extent, version, coordinate system
//!
//! To decode MVT tiles in Rust, use the `mapbox-vector-tile` crate.

use anyhow::Result;
use arcgis::{
    ArcGISClient, FontStack, GlyphRange, NoAuth, TileCoordinate, VectorTileServiceClient,
};
use std::collections::HashMap;

/// Public vector basemap service (no authentication required)
const WORLD_BASEMAP: &str =
    "https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🗺️  Vector Tile Service Examples");
    tracing::info!("Using ArcGIS World Basemap (public, no auth required)");
    tracing::info!("");

    // Create client with no authentication (public service)
    let client = ArcGISClient::new(NoAuth);
    let vt_service = VectorTileServiceClient::new(WORLD_BASEMAP, &client);

    // Demonstrate vector tile operations
    demonstrate_style_retrieval(&vt_service).await?;
    demonstrate_single_tile(&vt_service).await?;
    demonstrate_batch_tiles(&vt_service).await?;
    demonstrate_font_glyphs(&vt_service).await?;
    demonstrate_sprite_resources(&vt_service).await?;

    tracing::info!("\n✅ All vector tile examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates retrieving and inspecting the Mapbox GL style document.
async fn demonstrate_style_retrieval(vt_service: &VectorTileServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Style Document Retrieval ===");
    tracing::info!("Get Mapbox GL style JSON for rendering configuration");
    tracing::info!("");

    let style = vt_service.get_style().await?;

    tracing::info!("✅ Retrieved vector tile style");
    tracing::info!("   Mapbox GL version: {}", style.version());
    tracing::info!("   Layer count: {}", style.layers().len());

    if let Some(name) = style.name() {
        tracing::info!("   Style name: {}", name);
    }

    if let Some(zoom) = style.zoom() {
        tracing::info!("   Default zoom: {}", zoom);
    }

    if let Some(center) = style.center() {
        if center.len() >= 2 {
            tracing::info!("   Default center: [{}, {}]", center[0], center[1]);
        }
    }

    // Show layer types present in the style
    tracing::info!("");
    tracing::info!("   Sample layers:");
    for (i, layer) in style.layers().iter().take(5).enumerate() {
        if let Some(id) = layer.get("id").and_then(|v| v.as_str()) {
            let layer_type = layer
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            tracing::info!("     {}. {} (type: {})", i + 1, id, layer_type);
        }
    }

    tracing::info!("");
    tracing::info!("💡 Style documents define:");
    tracing::info!("   • Layer rendering order and visibility");
    tracing::info!("   • Colors, widths, and symbols");
    tracing::info!("   • Zoom level visibility ranges");
    tracing::info!("   • Data source URLs");

    Ok(())
}

/// Demonstrates fetching a single vector tile.
async fn demonstrate_single_tile(vt_service: &VectorTileServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Single Tile Fetch ===");
    tracing::info!("Download one MVT tile in Protocol Buffer format");
    tracing::info!("");

    // Tile for downtown San Francisco at zoom level 10
    // Tile coordinates: zoom=10, row=396, col=163
    let tile = TileCoordinate::new(10, 396, 163);

    tracing::info!("   Tile coordinates:");
    tracing::info!("     Zoom level: {}", tile.level());
    tracing::info!("     Row: {}", tile.row());
    tracing::info!("     Column: {}", tile.col());
    tracing::info!("");

    let tile_data = vt_service.get_tile(&tile).await?;

    tracing::info!("✅ Retrieved vector tile");
    tracing::info!("   Size: {} bytes", tile_data.len());
    tracing::info!("   Format: MVT (Mapbox Vector Tile / Protocol Buffer)");
    tracing::info!("");
    tracing::info!("💡 To decode MVT tiles:");
    tracing::info!("   • Use mapbox-vector-tile crate");
    tracing::info!("   • Extract layers and features");
    tracing::info!("   • Render with MapLibre GL or similar");

    Ok(())
}

/// Demonstrates batch fetching multiple tiles efficiently.
async fn demonstrate_batch_tiles(vt_service: &VectorTileServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Batch Tile Fetch ===");
    tracing::info!("Download multiple tiles in a single operation");
    tracing::info!("");

    // Fetch a 2x2 grid of tiles around San Francisco
    let tiles = vec![
        TileCoordinate::new(10, 396, 163), // Downtown SF
        TileCoordinate::new(10, 396, 164), // East
        TileCoordinate::new(10, 397, 163), // South
        TileCoordinate::new(10, 397, 164), // Southeast
    ];

    tracing::info!("   Fetching 2x2 tile grid:");
    tracing::info!("     Zoom level: 10");
    tracing::info!("     Rows: 396-397");
    tracing::info!("     Columns: 163-164");
    tracing::info!("");

    let tile_data = vt_service.get_tiles(&tiles).await?;

    tracing::info!("✅ Retrieved {} tiles", tile_data.len());

    let total_size: usize = tile_data.iter().map(|t| t.len()).sum();
    let avg_size = total_size / tile_data.len();

    tracing::info!("   Total size: {} bytes", total_size);
    tracing::info!("   Average tile size: {} bytes", avg_size);
    tracing::info!("");
    tracing::info!("💡 Batch fetching:");
    tracing::info!("   • More efficient than individual requests");
    tracing::info!("   • Ideal for tile caching systems");
    tracing::info!("   • Reduces HTTP overhead");
    tracing::info!("   • Perfect for offline map generation");

    Ok(())
}

/// Demonstrates downloading font glyphs for text rendering.
async fn demonstrate_font_glyphs(vt_service: &VectorTileServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 4: Font Glyph Retrieval ===");
    tracing::info!("Download font data for rendering text labels");
    tracing::info!("");

    // Common fonts used in basemaps
    let fonts = vec![
        ("Arial Unicode MS Regular", GlyphRange::ascii()),
        ("Noto Sans Regular", GlyphRange::basic_latin()),
    ];

    let mut glyph_sizes = HashMap::new();

    for (font_name, range) in &fonts {
        tracing::debug!(font = %font_name, range = %range, "Fetching glyphs");

        let font_stack = FontStack::new(*font_name);
        let glyphs = vt_service.get_fonts(&font_stack, range).await?;

        glyph_sizes.insert(*font_name, glyphs.len());

        tracing::info!("✅ Retrieved glyphs: {}", font_name);
        tracing::info!("   Range: {}", range);
        tracing::info!("   Size: {} bytes", glyphs.len());
        tracing::info!("");
    }

    tracing::info!("💡 Font glyphs:");
    tracing::info!("   • PBF format (Protocol Buffer)");
    tracing::info!("   • Contains vector outlines for text rendering");
    tracing::info!("   • Downloaded per Unicode range (0-255, 256-511, etc.)");
    tracing::info!("   • Cached by web mapping libraries");
    tracing::info!("");
    tracing::info!("   Common ranges:");
    tracing::info!("     • 0-255: ASCII and Latin-1");
    tracing::info!("     • 256-511: Latin Extended");
    tracing::info!("     • 1024-1279: Cyrillic");
    tracing::info!("     • 19968-20479: CJK Unified Ideographs");

    Ok(())
}

/// Demonstrates retrieving sprite resources (icons and symbols).
async fn demonstrate_sprite_resources(vt_service: &VectorTileServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 5: Sprite Sheet Resources ===");
    tracing::info!("Download icon/symbol sprite metadata and images");
    tracing::info!("");

    // Get sprite metadata (JSON describing icon positions)
    tracing::debug!("Fetching sprite metadata");
    let sprite_meta = vt_service.get_sprite_metadata().await?;

    let sprite_count = if sprite_meta.is_object() {
        sprite_meta.as_object().map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    tracing::info!("✅ Retrieved sprite metadata");
    tracing::info!("   Format: JSON");
    tracing::info!("   Sprite count: {}", sprite_count);

    // Show a few sprite examples
    if let Some(obj) = sprite_meta.as_object() {
        tracing::info!("   Sample sprites:");
        for (i, (name, _data)) in obj.iter().take(5).enumerate() {
            tracing::info!("     {}. {}", i + 1, name);
        }
    }

    tracing::info!("");

    // Get sprite image (PNG sprite sheet)
    tracing::debug!("Fetching sprite image");
    let sprite_image = vt_service.get_sprite_image().await?;

    tracing::info!("✅ Retrieved sprite image");
    tracing::info!("   Format: PNG");
    tracing::info!("   Size: {} bytes", sprite_image.len());
    tracing::info!("");
    tracing::info!("💡 Sprite sheets:");
    tracing::info!("   • PNG image containing all icons");
    tracing::info!("   • JSON metadata describes each icon's position and size");
    tracing::info!("   • Used for point symbols (markers, icons)");
    tracing::info!("   • Web renderers extract individual icons from sheet");
    tracing::info!("");
    tracing::info!("   Common sprites:");
    tracing::info!("     • Place markers (restaurant, hotel, etc.)");
    tracing::info!("     • Transportation symbols (airport, parking)");
    tracing::info!("     • Arrows and directional indicators");
    tracing::info!("     • Custom brand icons");

    Ok(())
}

/// Prints best practices for vector tile usage.
fn print_best_practices() {
    tracing::info!("\n💡 Vector Tile Best Practices:");
    tracing::info!("   - Cache tiles locally for better performance");
    tracing::info!("   - Use batch operations when downloading multiple tiles");
    tracing::info!("   - Implement tile pyramid (multiple zoom levels)");
    tracing::info!("   - Respect service rate limits and terms of use");
    tracing::info!("   - Use HTTP/2 for efficient tile downloads");
    tracing::info!("");
    tracing::info!("🎯 Tile Caching Strategy:");
    tracing::info!("   - File system: tiles/{{z}}/{{x}}/{{y}}.pbf");
    tracing::info!("   - Database: SQLite with MBTiles format");
    tracing::info!("   - Memory: LRU cache for frequently accessed tiles");
    tracing::info!("   - CDN: CloudFront/Cloudflare for public maps");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Pre-download tiles for expected viewport");
    tracing::info!("   - Use web workers for MVT decoding");
    tracing::info!("   - Enable GZIP compression (often default)");
    tracing::info!("   - Load higher zoom tiles progressively");
    tracing::info!("   - Implement tile request coalescing");
    tracing::info!("");
    tracing::info!("🌐 Web Integration:");
    tracing::info!("   - MapLibre GL JS: Open-source Mapbox GL alternative");
    tracing::info!("   - Leaflet: Via leaflet.vectorgrid plugin");
    tracing::info!("   - OpenLayers: Built-in MVT support");
    tracing::info!("   - Deck.gl: High-performance WebGL rendering");
    tracing::info!("");
    tracing::info!("📐 Coordinate Systems:");
    tracing::info!("   - MVT uses local tile coordinates (0-4096)");
    tracing::info!("   - Convert to/from geographic coordinates");
    tracing::info!("   - Web Mercator (EPSG:3857) is standard");
    tracing::info!("   - Tile coordinates follow XYZ scheme (zoom, x, y)");
}
