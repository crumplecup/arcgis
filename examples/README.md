# ArcGIS Rust SDK Examples

This directory contains examples demonstrating how to use the ArcGIS Rust SDK.

## Running Examples

```bash
cargo run --example <example_name>
```

## Available Examples

### Basic Client (`basic_client.rs`)

Demonstrates how to create an ArcGIS client with API Key authentication.

```bash
cargo run --example basic_client
```

## Coming Soon

The following examples will be added as features are implemented:

- `query_features.rs` - Query features from a Feature Service (v0.1.0)
- `spatial_query.rs` - Spatial relationship queries (v0.1.0)
- `edit_features.rs` - CRUD operations (v0.2.0)
- `oauth_flow.rs` - OAuth authentication (v0.2.0)
- `geocode_addresses.rs` - Geocoding Service (v0.3.0)
- `export_map.rs` - Map Service (v0.3.0)

## Authentication

Examples that require API keys or OAuth credentials should load them from environment variables:

```bash
export ARCGIS_API_KEY="your_api_key_here"
cargo run --example query_features
```

See the [main README](../README.md#authentication) for more details on authentication.
