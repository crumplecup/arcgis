# Contributing to arcgis-rust

Thank you for your interest in contributing to the ArcGIS Rust SDK! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in all interactions.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/arcgis-rust.git
   cd arcgis-rust
   ```
3. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/my-new-feature
   ```

## Development Workflow

### Building

```bash
# Build with default features
cargo build

# Build with all features
cargo build --all-features

# Build specific feature
cargo build --features geocoding
```

### Testing

```bash
# Run unit tests
cargo test

# Run all tests with all features
cargo test --all-features

# Run integration tests
cargo test --test integration

# Run a specific test
cargo test test_feature_query
```

### Code Quality

Before submitting a PR, ensure:

```bash
# Format code
cargo fmt

# Check formatting (CI will verify this)
cargo fmt --check

# Run clippy (no warnings allowed)
cargo clippy --all-targets --all-features -- -D warnings

# Check for common issues
cargo check --all-features
```

### Documentation

```bash
# Build documentation
cargo doc --no-deps --all-features

# Build and open documentation
cargo doc --no-deps --all-features --open

# Check for broken links
cargo doc --no-deps --all-features 2>&1 | grep warning
```

## Type Safety Requirements

This project enforces strict type safety:

### ✅ DO

- Use enums for all enumerated string values from the API
- Use newtypes for all ID types (LayerId, ObjectId, etc.)
- Use `chrono` types for temporal values
- Use `geo-types` for spatial primitives
- Implement `serde::Serialize`/`Deserialize` with proper field renames
- Add comprehensive documentation to all public APIs

### ❌ DON'T

- Use `String` for enumerated values
- Use `String` for temporal values
- Use bare integers for ID types
- Use tuples for compound values
- Add `unsafe` code without explicit justification and review

See [Type Safety as a Design Requirement](ARCGIS_REST_API_RESEARCH.md#why-rust-type-safety-as-a-design-requirement) for details.

## Adding a New Service

When adding a new ArcGIS service (e.g., Geocoding Service):

1. **Read the ArcGIS REST API documentation** thoroughly
2. **Extract all enumerated values** and create enums in `src/types/` or `src/services/{service}/enums.rs`
3. **Identify all ID types** and create newtypes in `src/types/ids.rs`
4. **Create request/response types** in `src/services/{service}/types.rs`
5. **Implement the client** in `src/services/{service}/client.rs`
6. **Write unit tests** for all serde serialization/deserialization
7. **Write integration tests** against live ArcGIS services (read-only)
8. **Add examples** in `examples/{service}_example.rs`
9. **Add Cargo feature** in `Cargo.toml`
10. **Update README.md** with the new service

Example structure:
```
src/services/geocode/
├── mod.rs         # Public exports
├── enums.rs       # Service-specific enums
├── types.rs       # Request/response types
├── client.rs      # GeocodeServiceClient
└── README.md      # Service-specific docs (optional)
```

## Commit Message Guidelines

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

Examples:
```
feat(feature): add spatial query support

Implement spatial relationship queries with all esriSpatialRel types.
Includes integration tests against public feature services.

Closes #42

fix(auth): correct OAuth token refresh timing

Token was being refreshed too early, causing unnecessary requests.
Now refreshes when 90% of expiry time has elapsed.

docs(readme): add geocoding service example

test(geometry): add round-trip conversion tests for all geometry types
```

## Pull Request Process

1. **Update documentation** for any changed functionality
2. **Add/update tests** to maintain or improve coverage
3. **Update CHANGELOG.md** under "Unreleased" section
4. **Ensure CI passes**:
   - All tests pass
   - Code is formatted (`cargo fmt`)
   - No clippy warnings
   - Documentation builds
5. **Fill out the PR template** completely
6. **Request review** from maintainers

### PR Title Format

```
<type>(<scope>): <description>
```

Example: `feat(geocoding): implement reverse geocoding`

## Testing Guidelines

### Unit Tests

- Test all serialization/deserialization paths
- Test enum string conversions
- Test builder patterns
- Test validation logic

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_type_serialization() {
        let geom_type = GeometryType::Point;
        let json = serde_json::to_string(&geom_type).unwrap();
        assert_eq!(json, r#""esriGeometryPoint""#);
    }

    #[test]
    fn test_geometry_type_deserialization() {
        let json = r#""esriGeometryPolyline""#;
        let geom_type: GeometryType = serde_json::from_str(json).unwrap();
        assert_eq!(geom_type, GeometryType::Polyline);
    }
}
```

### Integration Tests

- Use public ArcGIS services for read-only tests
- Use test organization for write operations (requires setup)
- Mark expensive tests with `#[ignore]` - run with `cargo test -- --ignored`

```rust
#[tokio::test]
async fn test_query_public_service() {
    let auth = ApiKeyAuth::new(env::var("ARCGIS_API_KEY").unwrap());
    let client = ArcGISClient::new(auth);

    // Test against a stable public service
    let result = client.query_features(/* ... */).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore] // Expensive test, run with --ignored
async fn test_large_dataset_pagination() {
    // ...
}
```

## Documentation Standards

All public APIs must be documented:

```rust
/// Queries features from a feature layer.
///
/// # Arguments
///
/// * `layer_id` - The ID of the layer to query
/// * `params` - Query parameters including WHERE clause, fields, etc.
///
/// # Returns
///
/// Returns a `FeatureSet` containing the matching features.
///
/// # Errors
///
/// Returns an error if:
/// - The layer doesn't exist
/// - The WHERE clause is invalid SQL
/// - Network request fails
///
/// # Example
///
/// ```no_run
/// use arcgis::{ArcGISClient, feature::FeatureServiceClient};
/// use arcgis::types::LayerId;
///
/// # async fn example() -> Result<(), arcgis::Error> {
/// let client = FeatureServiceClient::new("https://...", &arcgis_client);
/// let features = client
///     .query()
///     .layer(LayerId(0))
///     .where_clause("POPULATION > 100000")
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub async fn query(&self, layer_id: LayerId, params: QueryParams) -> Result<FeatureSet> {
    // ...
}
```

## Performance Considerations

- Use `&str` instead of `String` where possible
- Avoid unnecessary clones
- Use iterators instead of collecting when possible
- Profile before optimizing
- Benchmark performance-critical paths

## Security

- Never commit API keys or secrets
- Use `secrecy` crate for sensitive data
- Validate all user input
- Be aware of SQL injection in WHERE clauses
- Use `reqwest` redirect policy to prevent SSRF

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions
- Join the GeoRust community for general Rust GIS discussion

## License

By contributing, you agree that your contributions will be licensed under both the MIT and Apache-2.0 licenses.
