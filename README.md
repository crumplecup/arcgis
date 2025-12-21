# arcgis

[![Crates.io](https://img.shields.io/crates/v/arcgis.svg)](https://crates.io/crates/arcgis)
[![Documentation](https://docs.rs/arcgis/badge.svg)](https://docs.rs/arcgis)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](README.md#license)
[![Build Status](https://github.com/crumplecup/arcgis/workflows/CI/badge.svg)](https://github.com/crumplecup/arcgis/actions)

A type-safe Rust SDK for the [ArcGIS REST API](https://developers.arcgis.com/rest/) with compile-time guarantees.

## Features

- 🔒 **Type-safe**: Strong typing with enums instead of strings - invalid states are unrepresentable
- 🌍 **GeoRust integration**: Native support for `geo-types` and the GeoRust ecosystem
- 🔐 **Authentication**: API Key, OAuth 2.0 (authorization code & client credentials)
- ⚡ **Async/await**: Built on `tokio` and `reqwest` for async operations
- 🎯 **Focused features**: Optional services via Cargo features - only compile what you need
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
use arcgis::{ArcGISClient, auth::ApiKeyAuth, feature::FeatureServiceClient};

#[tokio::main]
async fn main() -> Result<(), arcgis::Error> {
    // Create authenticated client
    let auth = ApiKeyAuth::new("YOUR_API_KEY");
    let client = ArcGISClient::new(auth);

    // Connect to a feature service
    let feature_client = FeatureServiceClient::new(
        "https://services.arcgis.com/org/arcgis/rest/services/Dataset/FeatureServer",
        &client,
    );

    // Query features with type-safe API
    let features = feature_client
        .query()
        .layer(arcgis::types::LayerId(0))
        .where_clause("POPULATION > 100000")
        .out_fields(&["NAME", "POPULATION", "STATE"])
        .return_geometry(true)
        .execute()
        .await?;

    // Geometries are returned as geo-types
    for feature in features.features {
        if let Some(geometry) = feature.geometry {
            println!("Feature: {:?}", geometry);
        }
    }

    Ok(())
}
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
use arcgis::types::{GeometryType, SpatialRel};

params.geometry_type = GeometryType::Polyline;  // Autocomplete works!
params.spatial_rel = SpatialRel::Intersects;    // Typos = compile errors!
```

## Services Support

Enable the services you need via Cargo features:

```toml
[dependencies]
arcgis = { version = "0.1", features = ["feature-service", "geocoding", "map-service"] }
```

### Available Features

| Feature | Service | Status |
|---------|---------|--------|
| `feature-service` | Feature Service (default) | ✅ v0.1.0 |
| `map-service` | Map Service | 🚧 Planned |
| `geocoding` | Geocoding Service | 🚧 Planned |
| `geometry-service` | Geometry Service | 🚧 Planned |
| `routing` | Routing Service | 🚧 Planned |
| `geoprocessing` | Geoprocessing Service | 🚧 Planned |
| `stream-service` | Stream Service | 🚧 Planned |
| `places` | Places Service | 🚧 Planned |
| `pbf` | Protocol Buffer support | 🚧 Planned |
| `full` | All services | 🚧 Planned |

## Examples

See the [`examples/`](examples/) directory for complete examples:

- [`query_features.rs`](examples/query_features.rs) - Basic feature querying
- [`spatial_query.rs`](examples/spatial_query.rs) - Spatial relationship queries
- [`edit_features.rs`](examples/edit_features.rs) - CRUD operations (coming soon)
- [`oauth_flow.rs`](examples/oauth_flow.rs) - OAuth authentication (coming soon)

Run an example:

```bash
cargo run --example query_features
```

## Authentication

### API Key

```rust
use arcgis::auth::ApiKeyAuth;

let auth = ApiKeyAuth::new("YOUR_API_KEY");
let client = ArcGISClient::new(auth);
```

### OAuth 2.0 (Coming Soon)

```rust
use arcgis::auth::OAuthProvider;

let oauth = OAuthProvider::new(
    "client_id",
    "client_secret",
    "https://your-app.com/callback"
);

// Authorization code flow
let auth_url = oauth.authorize_url().await?;
// ... redirect user to auth_url ...
oauth.exchange_code(&authorization_code).await?;

let client = ArcGISClient::new(oauth);
```

## GeoRust Integration

Geometries are seamlessly converted to/from `geo-types`:

```rust
use geo_types::{Point, Polygon};
use arcgis::geometry::convert;

// ArcGIS JSON -> geo-types
let point: Point = convert::from_arcgis_point(&arcgis_point)?;

// geo-types -> ArcGIS JSON
let arcgis_polygon = convert::to_arcgis_polygon(&polygon)?;
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

- [x] Project structure and research
- [ ] v0.1.0: Feature Service (query only) + API Key auth
- [ ] v0.2.0: Feature Service (full CRUD) + OAuth
- [ ] v0.3.0: Map Service + Geocoding Service
- [ ] v0.4.0: Geometry + Routing + Geoprocessing Services
- [ ] v1.0.0: Production-ready with all services

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
