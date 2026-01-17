# ArcGIS Rust SDK Examples

This directory contains comprehensive examples demonstrating the ArcGIS Rust SDK's capabilities.

All examples follow consistent patterns:
- ✅ `main() -> anyhow::Result<()>` for proper error handling
- ✅ Structured logging with `tracing` (use `RUST_LOG=debug` for verbose output)
- ✅ Secure credential management via `.env` files
- ✅ Real-world use cases with best practices

## Running Examples

```bash
cargo run --example <example_name>
```

Set log level for detailed output:
```bash
RUST_LOG=debug cargo run --example query_features
```

## Authentication Setup

Most examples require credentials. Create a `.env` file in the project root:

```bash
cp .env.example .env
# Edit .env and add your credentials
```

`.env` file contents:
```env
ARCGIS_API_KEY=your_api_key_here
CLIENT_ID=your_client_id           # For OAuth examples
CLIENT_SECRET=your_client_secret   # For OAuth examples
```

**Get credentials:** [ArcGIS Developers Dashboard](https://developers.arcgis.com/)

## Available Examples

### 🌟 Getting Started

#### `basic_client.rs`
**Authentication:** API Key required
**Demonstrates:**
- Creating an ArcGIS client
- API key authentication
- Environment variable loading

```bash
cargo run --example basic_client
```

#### `client_credentials_flow.rs`
**Authentication:** OAuth Client Credentials required
**Demonstrates:**
- Fully automated OAuth 2.0 flow (no browser needed)
- Token caching and automatic refresh
- Server-to-server authentication

```bash
cargo run --example client_credentials_flow
```

### 📍 Feature Service (Queries & Data)

#### `query_features.rs`
**Authentication:** None (uses public service)
**Demonstrates:**
- Basic WHERE clause queries
- Field filtering (`out_fields`)
- Geometry control (`return_geometry`)
- Count-only queries
- Object ID queries
- Manual pagination (offset/limit)
- Auto-pagination (`execute_all`)
- Response formats (JSON, GeoJSON, PBF)

```bash
cargo run --example query_features
```

**Key Features:**
- No authentication required (public data)
- Comprehensive query examples
- Performance optimization techniques

#### `spatial_query.rs`
**Authentication:** None (uses public service)
**Demonstrates:**
- Bounding box queries (envelope)
- Polygon queries (custom shapes)
- Spatial relationships (Intersects, Contains, Within)
- Combined spatial + attribute filters
- Large area queries with pagination

```bash
cargo run --example spatial_query
```

**Key Features:**
- Advanced spatial queries
- Multiple spatial relationship types
- No authentication required

#### `edit_session.rs`
**Authentication:** OAuth Client Credentials required
**Demonstrates:**
- Versioned editing workflows
- Edit sessions for transaction semantics
- Adding features with session IDs
- Saving vs. discarding changes
- Branch-versioned geodatabases

```bash
cargo run --example edit_session
```

**Requirements:**
- ArcGIS Enterprise 11.2+
- Branch-versioned feature service
- Version Management Server

### 🗺️ Geocoding Service

#### `geocode_addresses.rs`
**Authentication:** API Key required
**Demonstrates:**
- Forward geocoding (address → coordinates)
- Reverse geocoding (coordinates → address)
- Autocomplete suggestions
- Multiple address processing
- Score-based filtering for quality

```bash
cargo run --example geocode_addresses
```

**Key Features:**
- Complete geocoding workflows
- Quality filtering examples
- Rate limiting best practices

### 📐 Geometry Service

#### `geometry_operations.rs`
**Authentication:** API Key required
**Demonstrates:**
- Coordinate projection (WGS84 ↔ Web Mercator)
- Buffer creation with geodesic calculations
- Distance calculation between points
- Batch geometry processing
- Builder pattern for parameters

```bash
cargo run --example geometry_operations
```

**Key Features:**
- GeoRust integration
- Type-safe units (LinearUnit enum)
- Multiple projection examples

## Example Categories

| Category | Examples | Auth Required |
|----------|----------|---------------|
| **Getting Started** | `basic_client`, `client_credentials_flow` | API Key / OAuth |
| **Feature Queries** | `query_features`, `spatial_query` | None (public) |
| **Editing** | `edit_session` | OAuth |
| **Geocoding** | `geocode_addresses` | API Key |
| **Geometry** | `geometry_operations` | API Key |

## Tips

### Logging

Control output verbosity with `RUST_LOG`:

```bash
# Show all logs
RUST_LOG=debug cargo run --example query_features

# Show only warnings and errors
RUST_LOG=warn cargo run --example geocode_addresses

# Show specific module logs
RUST_LOG=arcgis=debug cargo run --example geometry_operations
```

### Public vs Authenticated Services

Some examples use **public services** (no auth required):
- `query_features.rs` - World Cities service
- `spatial_query.rs` - World Cities service

Others require **authentication**:
- `geocode_addresses.rs` - Requires API key (credits consumed)
- `geometry_operations.rs` - Requires API key
- `edit_session.rs` - Requires OAuth + enterprise setup

### Best Practices

All examples demonstrate:
- ✅ Secure credential management (`.env` files, not hardcoded)
- ✅ Proper error handling (`anyhow::Result`, `?` operator)
- ✅ Structured logging (use fields for better debugging)
- ✅ Builder patterns where appropriate
- ✅ Type-safe enums (no magic strings)
- ✅ Real-world use cases

## Further Reading

- [Main README](../README.md) - SDK overview and features
- [API Documentation](https://docs.rs/arcgis) - Full API reference
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Development guidelines
- [ArcGIS REST API](https://developers.arcgis.com/rest/) - Official API docs
