# Contributing to ArcGIS Rust SDK

Welcome! Thank you for your interest in the ArcGIS Rust SDK. This library is in active early development, and **we'd love your feedback**.

## Why Publish Early?

We're publishing during early development specifically to get the library into your hands and hear what you think. Your experience using the SDK—what works, what doesn't, what's missing—directly shapes how we build it.

## We Want to Hear From You

**Have you tried the library?** We want to know:

- ✅ What worked well for you
- ❌ What didn't work or was confusing
- 💡 What features you need that we don't have yet
- 📖 What documentation would help
- 🐛 Any bugs you've encountered
- 🎯 What you're building with it

**[Share your feedback on GitHub Discussions →](https://github.com/crumplecup/arcgis/discussions)**

No issue is too small, no question is too basic. We genuinely want to hear from you.

## Quick Start

Get up and running in 60 seconds:

### 1. Add to your project

```bash
cargo add arcgis
```

### 2. Try a public query (no API key needed)

```rust
use arcgis::{ArcGISClient, FeatureServiceClient, LayerId, NoAuth};

#[tokio::main]
async fn main() -> Result<(), arcgis::Error> {
    // Use NoAuth for public ArcGIS services
    let client = ArcGISClient::new(NoAuth);

    // Query ESRI's public World Cities dataset
    let service = FeatureServiceClient::new(
        "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/World_Cities/FeatureServer",
        &client
    );

    // Find cities with population > 5 million
    let result = service
        .query(LayerId::new(0))
        .where_clause("POP > 5000000")
        .out_fields(&["CITY_NAME", "POP", "CNTRY_NAME"])
        .limit(10)
        .execute()
        .await?;

    println!("Found {} major cities", result.features().len());

    for feature in result.features() {
        let city = feature.attributes().get("CITY_NAME").unwrap();
        let pop = feature.attributes().get("POP").unwrap();
        let country = feature.attributes().get("CNTRY_NAME").unwrap();
        println!("  {} ({}) - population: {}", city, country, pop);
    }

    Ok(())
}
```

### 3. With an API key (for authenticated services)

```rust
use arcgis::{ArcGISClient, ApiKeyAuth};

#[tokio::main]
async fn main() -> Result<(), arcgis::Error> {
    let auth = ApiKeyAuth::new("YOUR_API_KEY");
    let client = ArcGISClient::new(auth);

    // Now you can access authenticated services:
    // - Geocoding (forward, reverse, batch)
    // - Routing (routes, service areas, closest facility)
    // - Places (search, categories, details)
    // - Feature services (query, edit, attachments)
    // - Map services (export, identify, find)
    // - And more...

    Ok(())
}
```

Get your free API key at [developers.arcgis.com](https://developers.arcgis.com).

## What's Working Now

Check out the [README](README.md) for the current feature list. We have 113 operations across 12 services implemented (65% API coverage).

**Popular features already working:**

- ✅ Geocoding (forward, reverse, batch, suggestions)
- ✅ Routing (routes, service areas, closest facility, OD cost matrix)
- ✅ Places (search, categories, details)
- ✅ Feature queries (spatial, attribute, related records)
- ✅ Feature editing (add, update, delete, attachments)
- ✅ Map tile services (vector tiles, raster tiles)
- ✅ Elevation services (profile, viewshed, summarize)
- ✅ Geometry operations (buffer, project, union, intersect)

## Development Roadmap

See [COVERAGE_ROADMAP.md](COVERAGE_ROADMAP.md) for our path to 100% API coverage. We're tracking progress through Bronze/Silver/Gold/Platinum milestones.

## Found a Bug?

**[Open an issue →](https://github.com/crumplecup/arcgis/issues/new)**

Please include:
- What you were trying to do
- What you expected to happen
- What actually happened
- Minimal code example (if possible)
- Rust version: `rustc --version`
- Crate version: `cargo tree | grep arcgis`

## Questions?

**[Ask on GitHub Discussions →](https://github.com/crumplecup/arcgis/discussions)**

We're here to help:
- ❓ Questions about using the library
- 💬 General discussion about ArcGIS and Rust
- 🎓 Learning how to contribute
- 🚀 Sharing what you're building

---

# Developer Contributions

Want to contribute code? Awesome! Here's everything you need to know.

## Getting Started

### 1. Set up your development environment

```bash
# Fork and clone the repo
git clone https://github.com/YOUR_USERNAME/arcgis.git
cd arcgis

# Install development tools
just setup

# Build and test
just build
just test
```

### 2. Read the project guidelines

See [CLAUDE.md](CLAUDE.md) for comprehensive guidelines on:
- Code style and patterns
- Testing requirements (tests go in `tests/`, not inline)
- Module organization (crate-level imports only)
- Error handling (use `derive_more` for all errors)
- Type construction (always use builders, never literals)
- Documentation standards

**Key rules:**
- All public functions must have `#[instrument]` for tracing
- Use `derive_more::Display` and `derive_more::Error` for errors
- Import as `use crate::{Type}`, never `use crate::module::Type`
- Tests go in `tests/` directory, never `#[cfg(test)]` inline
- Never use `#[allow]` - fix the root cause instead

## Development Workflow

### Create a feature branch

```bash
git checkout -b feature/your-feature-name
```

### Make your changes

Follow the patterns in [CLAUDE.md](CLAUDE.md) and existing code.

### Run all checks before committing

```bash
# Run comprehensive checks
just check-all

# Or individually:
just check           # Compilation
just clippy          # Linting
just fmt             # Format code
just test            # Unit tests
just test-package    # Package-specific tests
```

### Commit and push

```bash
git add .
git commit -m "feat(scope): description"
git push origin feature/your-feature-name
```

Follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Tests
- `refactor`: Refactoring
- `perf`: Performance
- `chore`: Maintenance

### Open a pull request

**[Create a pull request →](https://github.com/crumplecup/arcgis/compare)**

## Type Safety Requirements

This project enforces strict type safety:

### ✅ DO

- Use enums for all enumerated string values from the API
- Use newtypes for all ID types (LayerId, ObjectId, etc.)
- Use `chrono` types for temporal values
- Use `geo-types` for spatial primitives
- Implement `serde::Serialize`/`Deserialize` with proper field renames
- Add comprehensive documentation to all public APIs
- Always use builders for struct construction

### ❌ DON'T

- Use `String` for enumerated values
- Use `String` for temporal values
- Use bare integers for ID types
- Use tuples for compound values
- Add `unsafe` code (forbidden via `#![deny(unsafe_code)]`)
- Use struct literals (always use builders)
- Use `#[allow]` directives (fix root cause instead)

## Adding a New Service

When adding a new ArcGIS service:

1. **Read the ArcGIS REST API documentation** thoroughly
2. **Extract all enumerated values** and create enums in `src/services/{service}/types.rs`
3. **Identify all ID types** and create newtypes in `src/types/`
4. **Create request/response types** in `src/services/{service}/types.rs`
5. **Implement the client** in `src/services/{service}/client/mod.rs`
   - Split large clients into submodules (see portal, feature, version_management)
   - Keep files under 500-1000 lines
6. **Write integration tests** in `tests/{service}_test.rs`
7. **Update exports** in `src/lib.rs`
8. **Update README.md** with the new service
9. **Update COVERAGE_ROADMAP.md** with progress

Example structure for large services:
```
src/services/myservice/
├── mod.rs           # Module declarations + re-exports
├── types.rs         # Type definitions
└── client/
    ├── mod.rs       # Client struct + constructor
    ├── queries.rs   # Query methods
    ├── editing.rs   # Edit methods
    └── admin.rs     # Admin methods
```

## Testing Guidelines

### Tier 1: Public Tests (Free, runs in CI)

```rust
#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_public_service() {
    let client = ArcGISClient::new(NoAuth);
    // Test using public ESRI services
}
```

Run with: `cargo test --features test-public`

### Tier 2: Location Services (Manual)

Requires API key with location privileges:

```rust
#[tokio::test]
#[cfg(feature = "test-location")]
async fn test_geocoding() {
    let auth = ApiKeyAuth::new(env::var("ARCGIS_LOCATION_KEY").unwrap());
    let client = ArcGISClient::new(auth);
    // Test location services
}
```

Run with: `cargo test --features test-location`

### Tier 3: Portal/Publishing (Manual)

Requires API key with portal/publishing privileges. See [API_KEY_TESTING_STRATEGY.md](API_KEY_TESTING_STRATEGY.md) for complete details.

## Pre-Commit Checklist

Before committing, ensure:

```bash
# Format code
just fmt

# Run all checks
just check-all

# Run clippy (zero warnings)
just clippy

# Check all feature combinations
just check-features
```

## Pre-Merge Checklist

Before merging to main:

```bash
# Run comprehensive validation
just pre-merge

# This runs:
# - check-all (clippy, fmt, tests)
# - check-features (all feature combinations)
# - audit (security vulnerabilities)
# - test-public (public integration tests)
```

## Documentation Standards

All public APIs must be documented with:

- **What** it does (concise first line)
- **Why** it's useful (when non-obvious)
- **Parameters** (when not obvious from types)
- **Returns** (what you get back)
- **Errors** (what can go wrong)
- **Example** (for complex APIs)

```rust
/// Queries features from a feature layer.
///
/// # Arguments
///
/// * `layer_id` - The ID of the layer to query
///
/// # Returns
///
/// Returns a `FeatureSet` containing the matching features.
///
/// # Errors
///
/// Returns an error if the layer doesn't exist or the query is invalid.
///
/// # Example
///
/// ```no_run
/// use arcgis::{FeatureServiceClient, LayerId};
///
/// # async fn example(client: &FeatureServiceClient) -> Result<(), arcgis::Error> {
/// let features = client
///     .query(LayerId::new(0))
///     .where_clause("POPULATION > 100000")
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(self))]
pub async fn query(&self, layer_id: LayerId) -> QueryBuilder {
    // ...
}
```

## Development Commands

```bash
just setup           # Install development tools (cargo-dist, omnibor, etc.)
just build           # Build the project
just build-release   # Build with optimizations
just test            # Run unit tests
just test-package    # Run tests for specific package
just test-all        # Run all tests including doc tests
just check           # Basic compilation check
just check-all       # Clippy + fmt + tests
just check-features  # Check all feature combinations
just clippy          # Run clippy linter
just fmt             # Format code
just fmt-check       # Check formatting
just doc             # Build documentation
just doc-open        # Build and open documentation
just audit           # Security vulnerability scan
just omnibor         # Generate SBOM
just security        # Run all security checks (audit + omnibor)
just pre-commit      # Pre-commit validation
just pre-merge       # Pre-merge validation
just pre-publish     # Pre-publish validation (full checks)
just watch           # Watch for changes and run tests
```

See `just --list` for all available commands.

## Code of Conduct

Be respectful, be kind, be constructive. We're all here to build something useful together.

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for being part of this project!** Every bit of feedback, every question, every contribution makes this library better. We're excited to see what you build with it.
