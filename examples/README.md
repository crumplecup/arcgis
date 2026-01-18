# ArcGIS Rust SDK Examples

This directory contains comprehensive examples demonstrating the ArcGIS Rust SDK's capabilities, organized into two categories:

- **[public/](./public/)** - Examples using public services (no authentication required)
- **[enterprise/](./enterprise/)** - Examples requiring API keys or OAuth (may consume credits)

All examples follow consistent patterns:
- ✅ `main() -> anyhow::Result<()>` for proper error handling
- ✅ Structured logging with `tracing` (use `RUST_LOG=debug` for verbose output)
- ✅ Secure credential management via `.env` files
- ✅ Real-world use cases with best practices

## Quick Start

### Public Examples (No Auth Required)

Run immediately without any setup:

```bash
cargo run --example query_features
cargo run --example spatial_query
```

See [public/README.md](./public/README.md) for details.

### Enterprise Examples (Auth Required)

1. Create a `.env` file in the project root:

```env
ARCGIS_API_KEY=your_api_key_here
CLIENT_ID=your_client_id           # For OAuth examples
CLIENT_SECRET=your_client_secret   # For OAuth examples
```

2. Run examples:

```bash
cargo run --example geocode_addresses
cargo run --example routing_navigation
```

See [enterprise/README.md](./enterprise/README.md) for credit estimates and full details.

**Get credentials:** [ArcGIS Developers Dashboard](https://developers.arcgis.com/)

## Example Categories

### 🆓 Public Examples (Tier 1 - No Auth)

**Location:** [public/](./public/)

These examples use public services and can be run immediately without any API key or credentials.

| Example | Description | Runtime |
|---------|-------------|---------|
| `query_features` | Comprehensive Feature Service queries | ~5-10s |
| `spatial_query` | Advanced spatial relationship queries | ~5-10s |

**Credit Cost:** None

See [public/README.md](./public/README.md) for detailed documentation.

---

### 💼 Enterprise Examples (Tier 2+ - Auth Required)

**Location:** [enterprise/](./enterprise/)

These examples require authentication and may consume credits from your ArcGIS subscription. For enterprise users, the marginal cost is typically near zero.

#### Authentication & Setup

| Example | Tier | Credits | Description |
|---------|------|---------|-------------|
| `basic_client` | N/A | 0 | Creating an ArcGIS client with API key auth |
| `client_credentials_flow` | N/A | 0 | OAuth 2.0 client credentials flow |

#### Location Services (Tier 2)

| Example | Est. Credits | Runtime | Description |
|---------|--------------|---------|-------------|
| `geocode_addresses` | 0.04 | 5-10s | Forward/reverse geocoding, batch operations |
| `routing_navigation` | 0.50 | 10-15s | Route finding, turn-by-turn directions |
| `geometry_operations` | 0.10 | 5-10s | Buffer, union, intersection operations |

#### Portal Operations (Tier 3)

| Example | Est. Credits | Runtime | Description |
|---------|--------------|---------|-------------|
| `edit_session` | 0.01 | 5-10s | Feature editing with transactions |
| `feature_attachments` | 0.02 | 5-10s | Attachment upload/download/management |
| `portal_content_management` | 0.01 | 10-15s | Portal search, groups, metadata |

**Total for all enterprise examples:** ~0.68 credits (~$0.07)

See [enterprise/README.md](./enterprise/README.md) for detailed credit estimates and full documentation.

## Running Examples

All examples follow the same pattern:

```bash
cargo run --example <example_name>
```

Control log output with `RUST_LOG`:

```bash
# Show all logs (verbose)
RUST_LOG=debug cargo run --example query_features

# Show only warnings and errors (quiet)
RUST_LOG=warn cargo run --example geocode_addresses

# Show specific module logs
RUST_LOG=arcgis=debug cargo run --example geometry_operations
```

## Best Practices

All examples demonstrate:
- ✅ Secure credential management (`.env` files, not hardcoded)
- ✅ Proper error handling (`anyhow::Result`, `?` operator)
- ✅ Structured logging (use fields for better debugging)
- ✅ Builder patterns where appropriate
- ✅ Type-safe enums (no magic strings)
- ✅ Real-world use cases

## Credit Management

Enterprise examples may consume credits from your ArcGIS subscription. Key points:

- **Free tier:** 2M basemap tiles, 20k geocodes, 5k routes per month
- **Enterprise pools:** Shared credits across organization
- **Marginal cost:** Near zero for testing/learning
- **Monitor usage:** Check your [ArcGIS Dashboard](https://developers.arcgis.com/dashboard)

See [enterprise/README.md](./enterprise/README.md) for detailed credit estimates per example.

## Further Reading

- [Main README](../README.md) - SDK overview and features
- [API Documentation](https://docs.rs/arcgis) - Full API reference
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Development guidelines
- [ArcGIS REST API](https://developers.arcgis.com/rest/) - Official API docs
