# arcgis

[![Crates.io](https://img.shields.io/crates/v/arcgis.svg)](https://crates.io/crates/arcgis)
[![Documentation](https://docs.rs/arcgis/badge.svg)](https://docs.rs/arcgis)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](README.md#license)
[![Build Status](https://github.com/crumplecup/arcgis/workflows/CI/badge.svg)](https://github.com/crumplecup/arcgis/actions)

A type-safe Rust SDK for the [ArcGIS REST API](https://developers.arcgis.com/rest/) with compile-time guarantees.

## Features

- 🔒 **Type-safe**: Strong typing with enums instead of strings - invalid states are unrepresentable
- 🌍 **GeoRust integration**: Native support for `geo-types` and the GeoRust ecosystem
- 🔐 **Authentication**: API Key and OAuth 2.0 Client Credentials (fully automated, no browser required)
- ⚡ **Async/await**: Built on `tokio` and `reqwest` for async operations
- 🔄 **Auto-pagination**: Transparent handling of large result sets
- 📦 **Zero unsafe code**: Memory-safe by default
- 🧪 **Well-tested**: Comprehensive test coverage with integration tests

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
arcgis = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Query Features

```rust
use arcgis::{ApiKeyAuth, ArcGISClient, FeatureServiceClient, LayerId};

#[tokio::main]
async fn main() -> arcgis::Result<()> {
    // Create authenticated client
    let auth = ApiKeyAuth::new("YOUR_API_KEY");
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

### API Key (Simplest)

Best for development, testing, and simple applications:

```rust
use arcgis::{ApiKeyAuth, ArcGISClient};

let auth = ApiKeyAuth::new("YOUR_API_KEY");
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
        std::env::var("CLIENT_ID")?,
        std::env::var("CLIENT_SECRET")?,
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
   # Edit .env and add your ARCGIS_API_KEY or CLIENT_ID/CLIENT_SECRET
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

## Roadmap

See [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) for the detailed roadmap.

**Current Status**: End of Phase 2 (v0.2.0-ready)

- [x] **Phase 1**: OAuth 2.0 Client Credentials authentication
- [x] **Phase 2**: Feature Service query API with auto-pagination
- [ ] **Phase 3**: Feature Service editing (add, update, delete, batch operations)
- [ ] **Phase 4**: Additional services (Map, Geocoding, Geometry, Routing)
- [ ] **Phase 5**: Production hardening (retry logic, caching, circuit breaker)

**Version milestones:**
- [x] v0.1.0-alpha: Authentication infrastructure
- [ ] v0.2.0: Feature queries + OAuth (nearly complete)
- [ ] v0.3.0: Feature editing (CRUD operations)
- [ ] v0.4.0: Multi-service support
- [ ] v1.0.0: Production-ready

## Documentation

- [API Documentation](https://docs.rs/arcgis)
- [ArcGIS REST API Reference](https://developers.arcgis.com/rest/)
- [Research Document](ARCGIS_REST_API_RESEARCH.md)
- [Implementation Plan](IMPLEMENTATION_PLAN.md)
- [Architecture Decision Record](ARCHITECTURE_DECISION.md)

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
