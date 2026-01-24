# arcgis

[![Crates.io](https://img.shields.io/crates/v/arcgis.svg)](https://crates.io/crates/arcgis)
[![Documentation](https://docs.rs/arcgis/badge.svg)](https://docs.rs/arcgis)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](README.md#license)
[![Build Status](https://github.com/crumplecup/arcgis/workflows/CI/badge.svg)](https://github.com/crumplecup/arcgis/actions)

A type-safe Rust SDK for the [ArcGIS REST API](https://developers.arcgis.com/rest/) with compile-time guarantees.

**Current Status**: 113 operations across 12 services (65% API coverage, 80% use case coverage)

## Features

### Core Capabilities

- 🔒 **Type-safe**: Strong typing with enums instead of strings - invalid states are unrepresentable
- 🌍 **GeoRust integration**: Native support for `geo-types` and the GeoRust ecosystem
- 🔐 **Authentication**: API Key and OAuth 2.0 Client Credentials (fully automated, no browser required)
- ⚡ **Async/await**: Built on `tokio` and `reqwest` for async operations
- 🔄 **Auto-pagination**: Transparent handling of large result sets
- 📦 **Zero unsafe code**: Memory-safe by default
- 🧪 **Well-tested**: Comprehensive test coverage with integration tests

### Supported Operations

**Feature Management** (Feature Service):
- Query with advanced parameters (spatial, attribute, statistics, grouping)
- CRUD operations (add, update, delete, batch edits)
- Attachment management (upload, download, delete)
- Relationship queries, top features, field calculations
- Domain queries, truncate operations

**Mapping** (Map Service):
- Export maps with custom extent, layers, format
- Legend retrieval, identify operations
- Text search (find), KML generation
- Dynamic renderer generation

**Geocoding** (Geocode Service):
- Forward geocoding (single and batch)
- Reverse geocoding with filtering
- Autocomplete suggestions with categories
- Spatial reference customization

**Geometry Operations** (Geometry Service):
- Coordinate projection and transformations
- Buffer, union, simplify operations
- Distance and area calculations
- Datum transformation discovery

**Network Analysis** (Routing Service):
- Route solving (optimal paths between stops)
- Service area generation (drive-time polygons)
- Closest facility analysis
- Origin-destination cost matrices

**Geoprocessing**:
- Synchronous and asynchronous execution
- Job status polling with exponential backoff
- Result retrieval and message handling

**Portal & Content Management**:
- User and item CRUD operations
- Search with advanced queries
- Sharing and group management
- Service publishing and updates

**Imagery** (Image Service):
- Image export with rendering rules
- Pixel value identification and sampling
- Histogram computation
- Raster metadata retrieval

**3D & Tiles**:
- Vector tile retrieval (MVT format)
- Style and font management
- Sprite sheet access

**Versioned Workflows** (Version Management):
- Edit and read sessions
- Version creation and management
- Reconciliation and posting
- Conflict detection and resolution

**Location Services**:
- Elevation profiles and terrain analysis
- Viewshed computation
- POI search with categories

## Quick Start

### 1. Add Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
arcgis = "0.1"
tokio = { version = "1", features = ["full"] }
```

### 2. Set Up Credentials

Create a `.env` file in your project root (automatically loaded by the SDK):

```bash
cp .env.example .env
```

Add your credentials to `.env`:

```env
ARCGIS_API_KEY=your_api_key_here
```

**Important**: Add `.env` to your `.gitignore` to keep credentials out of version control!

Get your API key from the [ArcGIS Developer Dashboard](https://developers.arcgis.com/).

### 3. Query Features

```rust
use arcgis::{ApiKeyAuth, ArcGISClient, FeatureServiceClient, LayerId};

#[tokio::main]
async fn main() -> arcgis::Result<()> {
    // Load API key from environment (.env is automatically loaded)
    let api_key = std::env::var("ARCGIS_API_KEY")
        .expect("ARCGIS_API_KEY must be set in .env");

    // Create authenticated client
    let auth = ApiKeyAuth::new(api_key);
    let client = ArcGISClient::new(auth);

    // Connect to a feature service
    let service = FeatureServiceClient::new(
        "https://services.arcgis.com/org/arcgis/rest/services/Dataset/FeatureServer",
        &client,
    );

    // Query features with type-safe builder API
    let features = service
        .query(LayerId::new(0))
        .where_clause("POPULATION > 100000")
        .out_fields(&["NAME", "POPULATION", "STATE"])
        .return_geometry(true)
        .execute()
        .await?;

    println!("Retrieved {} features", features.features().len());

    // Geometries are returned as geo-types
    for feature in features.features() {
        if let Some(geometry) = feature.geometry() {
            println!("Feature geometry: {:?}", geometry);
        }
    }

    Ok(())
}
```

### Auto-Pagination

For large datasets, use `execute_all()` to automatically paginate:

```rust
// Retrieve all features matching the query (may make multiple requests)
let all_features = service
    .query(LayerId::new(0))
    .where_clause("STATE = 'CA'")
    .execute_all()  // Automatically handles pagination
    .await?;

println!("Retrieved {} total features", all_features.features().len());
```

### Type Safety Example

Instead of error-prone strings:

```rust
// ❌ Runtime errors waiting to happen
params.geometry_type = "esriGeometryPolyline";  // Typo? No compile error!
params.spatial_rel = "esriSpatialRelIntersect";  // Missing 's'? No compile error!
```

Use strongly-typed enums:

```rust
// ✅ Compile-time guarantees
use arcgis::{GeometryType, SpatialRel};

params.geometry_type = GeometryType::Polyline;  // Autocomplete works!
params.spatial_rel = SpatialRel::Intersects;    // Typos = compile errors!
```

## Authentication

The SDK automatically loads credentials from a `.env` file when you create a client. This keeps your API keys and secrets out of your code and version control.

### API Key (Simplest)

Best for development, testing, and simple applications:

```rust
use arcgis::{ApiKeyAuth, ArcGISClient};

// Load API key from environment (.env automatically loaded)
let api_key = std::env::var("ARCGIS_API_KEY")
    .expect("ARCGIS_API_KEY must be set in .env");

let auth = ApiKeyAuth::new(api_key);
let client = ArcGISClient::new(auth);
```

### OAuth 2.0 Client Credentials (Recommended for Production)

Fully automated server-to-server authentication - no browser or user interaction required:

```rust
use arcgis::{ClientCredentialsAuth, ArcGISClient, AuthProvider};

#[tokio::main]
async fn main() -> arcgis::Result<()> {
    // Create authenticator with client credentials
    let auth = ClientCredentialsAuth::new(
        std::env::var("ARCGIS_CLIENT_ID")?,
        std::env::var("ARCGIS_CLIENT_SECRET")?,
    )?;

    // Token is fetched automatically on first use
    let client = ArcGISClient::new(auth);

    // All subsequent requests automatically use refreshed tokens
    // No manual token management required!

    Ok(())
}
```

**Key features:**
- Fully automated - no browser interaction
- Automatic token refresh before expiration
- Perfect for servers, CLI tools, and CI/CD
- Short-lived tokens (2 hours) for better security

See [`examples/client_credentials_flow.rs`](examples/client_credentials_flow.rs) for a complete example.

## Examples

See the [`examples/`](examples/) directory for complete examples:

- [`basic_client.rs`](examples/basic_client.rs) - Basic client setup and usage
- [`client_credentials_flow.rs`](examples/client_credentials_flow.rs) - OAuth 2.0 automated authentication

Run an example:

```bash
cargo run --example client_credentials_flow
```

## Testing

### Unit Tests

Run unit tests (no credentials required):

```bash
cargo test
```

### Integration Tests

Integration tests require ArcGIS credentials and the `api` feature flag.

1. **Set up credentials**:
   ```bash
   cp .env.example .env
   # Edit .env and add your ARCGIS_API_KEY or ARCGIS_CLIENT_ID/ARCGIS_CLIENT_SECRET
   ```

2. **Run integration tests**:
   ```bash
   # Run all API tests (be patient, includes rate limiting)
   cargo test --features api

   # Or use the justfile recipe
   just test-api

   # Run specific test
   cargo test --features api test_public_feature_service_accessible
   ```

See [`tests/README.md`](tests/README.md) for more details.

## GeoRust Integration

All geometries use the GeoRust ecosystem via `geo-types`. ArcGIS geometries are automatically converted when querying features:

```rust
use geo_types::{Point, Polygon};

// Query features - geometries are returned as geo-types
let features = service
    .query(LayerId::new(0))
    .return_geometry(true)
    .execute()
    .await?;

// Work with native geo-types
for feature in features.features() {
    if let Some(geometry) = feature.geometry() {
        match geometry {
            geo_types::Geometry::Point(pt) => println!("Point at {}, {}", pt.x(), pt.y()),
            geo_types::Geometry::Polygon(poly) => println!("Polygon with {} points", poly.exterior().points().count()),
            _ => {}
        }
    }
}
```

## Design Philosophy

This SDK prioritizes **type safety** and **correctness**:

1. **No stringly-typed APIs**: Every enumerated value in the ArcGIS API is represented as a Rust enum
2. **Newtype pattern**: IDs are wrapped in newtypes (e.g., `LayerId`, `ObjectId`) to prevent mixing
3. **Validated construction**: Invalid states are prevented at compile time
4. **Leverage existing crates**: Uses `oauth2`, GeoRust, `reqwest`, and `tokio` instead of reinventing

See [ARCGIS_REST_API_RESEARCH.md](ARCGIS_REST_API_RESEARCH.md) for the full design rationale.

## Minimum Supported Rust Version (MSRV)

This crate requires Rust 1.75 or later.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/crumplecup/arcgis.git
cd arcgis

# Build the project
cargo build

# Run tests
cargo test

# Run tests with all features
cargo test --all-features

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-targets --all-features
```

## Implemented Services

**12 Services, 113 Operations** (Baseline - 65% coverage):

| Service | Operations | Status |
|---------|-----------|--------|
| **Elevation** | 3 | ✅ Complete |
| **Feature** | 17 | ✅ Complete (queries, edits, attachments, admin) |
| **Geocode** | 9 | ✅ Complete (forward, reverse, batch, suggest) |
| **Geometry** | 8 | ✅ Core Complete (project, buffer, union, measure) |
| **Geoprocessing** | 7 | ✅ Complete (sync/async execution, polling) |
| **Image** | 6 | ✅ Core Complete (export, identify, samples, histograms) |
| **Map** | 10 | ✅ Complete (export, legend, identify, KML, renderer) |
| **Places** | 3 | ✅ Complete (search, details, categories) |
| **Portal** | 24 | ✅ Core Complete (users, items, sharing, publishing, groups) |
| **Routing** | 4 | ✅ Complete (route, service area, closest facility, OD matrix) |
| **Vector Tile** | 6 | ✅ Core Complete (tiles, style, fonts, sprites) |
| **Version Management** | 16 | ✅ Complete (sessions, CRUD, reconcile, conflicts) |

See [COVERAGE_ROADMAP.md](COVERAGE_ROADMAP.md) for the path to full coverage.

## Roadmap to Full Coverage

**Bronze** (Phase 1 - 70% coverage):
- Complete existing services (+20 ops)
- Image/Portal/Feature extensions

**Silver** (Phase 2 - 75% coverage):
- Scene Service (3D visualization)
- Stream Service (real-time WebSocket)
- Utility Network Service
- GeoEnrichment Service

**Gold** (Phase 3 - 80% coverage):
- Network Diagram
- Parcel Fabric
- Printing Service

**Platinum** (Full Coverage - 80-85% coverage):
- All Tier 1-3 services complete
- 183-200 total operations
- 95%+ use case coverage

## Documentation

- [API Documentation](https://docs.rs/arcgis)
- [Coverage Roadmap](COVERAGE_ROADMAP.md) - Path to full API coverage
- [ArcGIS REST API Reference](https://developers.arcgis.com/rest/)
- [Research Document](ARCGIS_REST_API_RESEARCH.md)

## Related Projects

- [GeoRust](https://github.com/georust) - Geospatial primitives and algorithms for Rust
- [arcgis-rest-js](https://github.com/Esri/arcgis-rest-js) - Official JavaScript wrapper (reference implementation)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Disclaimer

This is an unofficial community-driven project and is not officially supported by Esri. For official Esri SDKs, see [ArcGIS Developer Documentation](https://developers.arcgis.com/).
