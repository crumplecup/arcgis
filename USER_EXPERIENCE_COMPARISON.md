# User Experience Comparison: Monorepo vs Workspace

## Installation & Dependencies

### Monorepo Approach (Chosen)

**Minimal install** (just Feature Service):
```toml
[dependencies]
arcgis = "0.1"  # Uses default features = ["feature-service"]
```

**Custom features**:
```toml
[dependencies]
arcgis = { version = "0.1", features = ["feature-service", "geocoding", "map-service"] }
```

**Everything**:
```toml
[dependencies]
arcgis = { version = "0.1", features = ["full"] }
```

**No default features** (opt-in to what you need):
```toml
[dependencies]
arcgis = { version = "0.1", default-features = false, features = ["geocoding"] }
```

---

### Workspace Approach (Alternative)

**Minimal install**:
```toml
[dependencies]
arcgis-core = "0.1"
arcgis-types = "0.1"
arcgis-geometry = "0.1"
arcgis-feature = "0.1"
```

**Multiple services**:
```toml
[dependencies]
arcgis-core = "0.1.0"
arcgis-types = "0.1.0"
arcgis-geometry = "0.1.0"
arcgis-feature = "0.1.0"
arcgis-geocoding = "0.1.3"  # Different patch version!
arcgis-map = "0.2.0"        # Breaking change in map service
```

**Everything** (meta-crate):
```toml
[dependencies]
arcgis = { version = "0.1", features = ["full"] }  # Still need a meta-crate

# OR list them all
arcgis-core = "0.1"
arcgis-types = "0.1"
arcgis-geometry = "0.1"
arcgis-feature = "0.1"
arcgis-map = "0.2"
arcgis-geocoding = "0.1"
arcgis-routing = "0.1"
arcgis-geoprocessing = "0.1"
```

---

## Code Usage

### Monorepo Approach (Chosen)

```rust
use arcgis::{
    ArcGISClient,
    auth::ApiKeyAuth,
    feature::FeatureServiceClient,
    geocoding::GeocodeServiceClient,
    types::{GeometryType, SpatialRel, LayerId},
};

#[tokio::main]
async fn main() -> Result<(), arcgis::Error> {
    let auth = ApiKeyAuth::new("YOUR_API_KEY");
    let client = ArcGISClient::new(auth);

    // Feature Service
    let feature_client = FeatureServiceClient::new(
        "https://services.arcgis.com/.../FeatureServer",
        &client,
    );

    let features = feature_client
        .query()
        .layer(LayerId(0))
        .where_clause("POPULATION > 100000")
        .return_geometry(true)
        .execute()
        .await?;

    // Geocoding
    let geocode_client = GeocodeServiceClient::new(
        "https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer",
        &client,
    );

    let candidates = geocode_client
        .find_address_candidates("380 New York St, Redlands, CA")
        .await?;

    Ok(())
}
```

**Single import path**, all types from one crate.

---

### Workspace Approach (Alternative)

```rust
// Must import from multiple crates
use arcgis_core::ArcGISClient;
use arcgis_auth::ApiKeyAuth;
use arcgis_feature::FeatureServiceClient;
use arcgis_geocoding::GeocodeServiceClient;
use arcgis_types::{GeometryType, SpatialRel, LayerId};

#[tokio::main]
async fn main() -> Result<(), arcgis_core::Error> {
    let auth = ApiKeyAuth::new("YOUR_API_KEY");
    let client = ArcGISClient::new(auth);

    // Feature Service
    let feature_client = FeatureServiceClient::new(
        "https://services.arcgis.com/.../FeatureServer",
        &client,
    );

    let features = feature_client
        .query()
        .layer(LayerId(0))
        .where_clause("POPULATION > 100000")
        .return_geometry(true)
        .execute()
        .await?;

    // Geocoding
    let geocode_client = GeocodeServiceClient::new(
        "https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer",
        &client,
    );

    let candidates = geocode_client
        .find_address_candidates("380 New York St, Redlands, CA")
        .await?;

    Ok(())
}
```

**Multiple import paths**, error types from different crates.

---

## Version Management

### Monorepo Approach (Chosen)

**Single version to track**:
```
v0.1.0 - Initial release with Feature Service
v0.2.0 - Add OAuth + editing (breaking auth change)
v0.3.0 - Add Map and Geocoding services
v0.4.0 - Add Routing and Geometry services
v1.0.0 - Stability guarantee
```

**Cargo.lock** (simplified):
```toml
[[package]]
name = "arcgis"
version = "0.3.0"
```

**Upgrading**:
```bash
cargo update arcgis
# One dependency, one version, done
```

---

### Workspace Approach (Alternative)

**Multiple versions to track**:
```
arcgis-core v0.1.0 - Initial release
arcgis-types v0.1.0 - Initial types
arcgis-feature v0.1.0 - Feature Service

arcgis-core v0.2.0 - OAuth support (BREAKING)
arcgis-types v0.1.1 - Add new enum variant
arcgis-feature v0.2.0 - Update for new core (BREAKING)
arcgis-map v0.1.0 - Initial release (depends on core 0.2)

arcgis-core v0.2.1 - Bug fix
arcgis-feature v0.2.1 - Update for core fix

arcgis-types v0.2.0 - Rename enum variant (BREAKING)
arcgis-feature v0.3.0 - Update for new types (BREAKING)
arcgis-map v0.2.0 - Update for new types (BREAKING)
```

**Cargo.lock** (complex):
```toml
[[package]]
name = "arcgis-core"
version = "0.2.1"

[[package]]
name = "arcgis-types"
version = "0.2.0"

[[package]]
name = "arcgis-feature"
version = "0.3.0"
dependencies = [
    "arcgis-core 0.2.1",
    "arcgis-types 0.2.0",
]

[[package]]
name = "arcgis-map"
version = "0.2.0"
dependencies = [
    "arcgis-core 0.2.1",
    "arcgis-types 0.2.0",
]
```

**Upgrading**:
```bash
cargo update arcgis-core
cargo update arcgis-types
cargo update arcgis-feature
cargo update arcgis-map
# Hope they're all compatible!
```

---

## Documentation

### Monorepo Approach (Chosen)

**Single docs site**: https://docs.rs/arcgis

**Structure**:
- arcgis::client - HTTP client
- arcgis::auth - Authentication
- arcgis::types - Shared types
- arcgis::feature - Feature Service
- arcgis::map - Map Service
- arcgis::geocoding - Geocoding

**Search**: All types searchable in one place

**Examples**: All in one crate's `examples/` directory

---

### Workspace Approach (Alternative)

**Multiple docs sites**:
- https://docs.rs/arcgis-core
- https://docs.rs/arcgis-types
- https://docs.rs/arcgis-feature
- https://docs.rs/arcgis-map
- https://docs.rs/arcgis-geocoding

**Navigation**: Must switch between docs sites to understand how types interact

**Search**: Can't search across all crates

**Examples**: Scattered across multiple crates

---

## Error Handling

### Monorepo Approach (Chosen)

```rust
use arcgis::Error;

fn my_function() -> Result<(), arcgis::Error> {
    // All errors from the same type
    let client = ArcGISClient::new(auth)?;
    let features = feature_client.query().execute().await?;
    let addresses = geocode_client.find_address_candidates("...").await?;
    Ok(())
}
```

**Single error type**, easy error propagation.

---

### Workspace Approach (Alternative)

```rust
use arcgis_core::Error as CoreError;
use arcgis_feature::Error as FeatureError;
use arcgis_geocoding::Error as GeocodingError;

// OR define your own error type that wraps all of them
#[derive(Debug, thiserror::Error)]
enum MyError {
    #[error("Core error: {0}")]
    Core(#[from] CoreError),

    #[error("Feature error: {0}")]
    Feature(#[from] FeatureError),

    #[error("Geocoding error: {0}")]
    Geocoding(#[from] GeocodingError),
}

fn my_function() -> Result<(), MyError> {
    let client = ArcGISClient::new(auth)?;
    let features = feature_client.query().execute().await?;
    let addresses = geocode_client.find_address_candidates("...").await?;
    Ok(())
}
```

**Multiple error types**, user must unify them.

---

## Breaking Changes

### Monorepo Approach (Chosen)

**Scenario**: Need to add a new variant to `GeometryType` enum

**Impact**:
- Bump to v0.4.0 (minor bump, adding variant is not breaking in Rust with `#[non_exhaustive]`)
- All services automatically get new version
- Users update once: `arcgis = "0.4"`

**CHANGELOG**:
```markdown
# v0.4.0

## Added
- New `GeometryType::MultiPatch` variant
- Support for 3D geometries in Feature Service

## Changed
- Feature Service query builder now supports 3D geometries

## Fixed
- Geometry conversion edge case
```

---

### Workspace Approach (Alternative)

**Scenario**: Need to add a new variant to `GeometryType` enum in `arcgis-types`

**Impact**:
- Bump `arcgis-types` to v0.4.0
- Bump `arcgis-feature` to v0.5.0 (depends on types 0.4)
- Bump `arcgis-map` to v0.3.0 (depends on types 0.4)
- Bump `arcgis-geometry` to v0.2.0 (depends on types 0.4)

Users must update ALL of them simultaneously.

**CHANGELOGs** (across 4 crates):

`arcgis-types/CHANGELOG.md`:
```markdown
# v0.4.0

## Added
- New `GeometryType::MultiPatch` variant
```

`arcgis-feature/CHANGELOG.md`:
```markdown
# v0.5.0

## Changed
- Updated to arcgis-types v0.4.0 for MultiPatch support
```

`arcgis-map/CHANGELOG.md`:
```markdown
# v0.3.0

## Changed
- Updated to arcgis-types v0.4.0 for MultiPatch support
```

`arcgis-geometry/CHANGELOG.md`:
```markdown
# v0.2.0

## Changed
- Updated to arcgis-types v0.4.0 for MultiPatch support
- Added conversion support for MultiPatch geometries
```

---

## Compile Times

### Monorepo Approach (Chosen)

**Default features** (Feature Service only):
```bash
$ cargo build
   Compiling arcgis v0.1.0
   # ~60 seconds (estimate)
```

**Full features**:
```bash
$ cargo build --features full
   Compiling arcgis v0.1.0
   # ~120 seconds (estimate)
```

**Custom features** (just what you need):
```bash
$ cargo build --features "feature-service,geocoding"
   Compiling arcgis v0.1.0
   # ~80 seconds (estimate)
```

---

### Workspace Approach (Alternative)

**Minimal**:
```bash
$ cargo build
   Compiling arcgis-core v0.1.0
   Compiling arcgis-types v0.1.0
   Compiling arcgis-geometry v0.1.0
   Compiling arcgis-feature v0.1.0
   # ~60 seconds (estimate, similar to monorepo)
```

**Multiple services**:
```bash
$ cargo build
   Compiling arcgis-core v0.1.0
   Compiling arcgis-types v0.1.0
   Compiling arcgis-geometry v0.1.0
   Compiling arcgis-feature v0.1.0
   Compiling arcgis-map v0.1.0
   Compiling arcgis-geocoding v0.1.0
   # ~120 seconds (estimate, similar to monorepo with full features)
```

**Benefit**: Can compile crates in parallel (but Cargo already does this for modules)

---

## Testing

### Monorepo Approach (Chosen)

```bash
# Test everything
$ cargo test

# Test specific service
$ cargo test --features feature-service

# Test with all features
$ cargo test --features full
```

**Test organization**:
```
tests/
├── feature_service.rs
├── map_service.rs
├── geocoding.rs
└── integration.rs
```

---

### Workspace Approach (Alternative)

```bash
# Test everything (from workspace root)
$ cargo test --all

# Test specific crate
$ cargo test -p arcgis-feature

# Must ensure inter-crate compatibility
$ cargo test --all --all-features
```

**Test organization** (fragmented):
```
crates/arcgis-feature/tests/
crates/arcgis-map/tests/
crates/arcgis-geocoding/tests/
# Integration tests must live somewhere...
```

---

## Summary Table

| Aspect | Monorepo (Chosen) | Workspace (Alternative) |
|--------|-------------------|------------------------|
| **User dependencies** | 1 | 4-8+ |
| **Version tracking** | 1 version | 4-8+ versions |
| **Breaking changes** | 1 bump | Cascade across crates |
| **Import paths** | `arcgis::*` | `arcgis_*::*` |
| **Error handling** | Single type | Multiple types |
| **Documentation** | Single site | Multiple sites |
| **Compile time (minimal)** | ~60s | ~60s |
| **Compile time (full)** | ~120s | ~120s |
| **Maintenance burden** | Low | High |
| **Release process** | Simple | Complex |
| **User experience** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Flexibility** | Medium | High |
| **Independent versioning** | No | Yes |

## Conclusion

**Monorepo wins** for user experience, simplicity, and maintenance.

**Workspace** only makes sense if:
- You have 50+ services (we have ~10)
- Services version independently (ArcGIS versions as a platform)
- You have multiple teams owning different services (we're one team/one person)
- Users need ultra-minimal dependencies (feature flags solve this)

**Decision**: Start with monorepo, structured to split later if needed.
