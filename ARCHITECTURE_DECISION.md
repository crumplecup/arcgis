# Architecture Decision: Monorepo vs Workspace

## Decision: Monorepo with Feature Flags

**Status**: Decided
**Date**: 2025-12-21
**Context**: Determining whether to implement ArcGIS Rust SDK as a single crate (monorepo) or multiple crates (workspace)

## TL;DR

Start with a **single crate** using **Cargo feature flags** to make services optional, but structure the code to be **workspace-compatible** from day one. Split into a workspace only if the crate exceeds ~100k LOC or if there's a clear need for independent service versioning.

## Analysis

### Option 1: Monorepo (Single Crate)

**Structure**:
```
arcgis/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── client.rs
    ├── error.rs
    ├── auth/
    ├── types/
    ├── geometry/
    └── services/
        ├── feature/
        ├── map/
        ├── geocode/
        └── ...
```

**Cargo.toml**:
```toml
[package]
name = "arcgis"
version = "0.1.0"

[features]
default = ["feature-service"]

# Core (always included)
# - client, auth, types, geometry conversion

# Services (optional via features)
feature-service = []
map-service = []
geocoding = []
geometry-service = ["geo/use-serde"]  # May have extra deps
routing = []
geoprocessing = []
stream-service = ["tokio/sync"]  # WebSocket deps
places = []

# Format support
pbf = ["arcpbf", "prost"]

# All services
full = [
    "feature-service",
    "map-service",
    "geocoding",
    "geometry-service",
    "routing",
    "geoprocessing",
    "stream-service",
    "places",
    "pbf",
]
```

**Pros**:
✅ Single dependency for users: `arcgis = "0.1"`
✅ Single version number - simple semver
✅ Shared types without duplication
✅ No circular dependency issues
✅ Single CHANGELOG, single release
✅ Faster iteration during development
✅ Feature flags allow optional compilation
✅ Similar to other Rust SDKs (aws-sdk-rust uses this pattern)

**Cons**:
❌ Larger compile if user enables `full`
❌ Single version for all services (can't version independently)
❌ All services share same release cycle
❌ Harder to extract individual services later

### Option 2: Workspace (Multiple Crates)

**Structure**:
```
arcgis/
├── Cargo.toml          # Workspace root
├── crates/
│   ├── arcgis-core/    # HTTP client, auth, errors
│   ├── arcgis-types/   # Shared types (enums, IDs)
│   ├── arcgis-geometry/ # Geometry conversions
│   ├── arcgis-feature/ # Feature Service
│   ├── arcgis-map/     # Map Service
│   ├── arcgis-geocoding/
│   ├── arcgis-routing/
│   └── arcgis/         # Meta-crate (re-exports all)
└── examples/
```

**Pros**:
✅ Independent versioning per service
✅ Smaller compile units
✅ Clear boundaries between services
✅ Can publish services independently
✅ Easier to maintain long-term (if very large)

**Cons**:
❌ Dependency explosion for users
❌ Version management nightmare (core@0.1, feature@0.2, map@0.1.3)
❌ Shared types create circular dependencies
❌ Multiple CHANGELOGs to maintain
❌ Harder to ensure cross-service compatibility
❌ More complex release process
❌ Breaking change in core breaks all services

### Option 3: Hybrid (Recommended)

**Start**: Monorepo with feature flags
**Migrate**: To workspace if crate exceeds ~100k LOC or clear need emerges

**Structure** (workspace-ready but single crate):
```
arcgis/
├── Cargo.toml
└── src/
    ├── lib.rs
    │
    ├── core/              # Could become arcgis-core
    │   ├── mod.rs
    │   ├── client.rs
    │   ├── error.rs
    │   └── auth/
    │
    ├── types/             # Could become arcgis-types
    │   ├── mod.rs
    │   ├── geometry.rs
    │   ├── spatial.rs
    │   ├── field.rs
    │   └── ids.rs
    │
    ├── geometry/          # Could become arcgis-geometry
    │   ├── mod.rs
    │   └── convert.rs
    │
    └── services/          # Each could become a crate
        ├── mod.rs
        ├── feature/       # Could become arcgis-feature
        ├── map/           # Could become arcgis-map
        └── geocode/       # Could become arcgis-geocoding
```

**Migration Path**:
1. When crate becomes large, extract `core/` → `arcgis-core` crate
2. Extract `types/` → `arcgis-types` crate
3. Extract each service to its own crate
4. Create meta-crate `arcgis` that re-exports everything

**Benefits**:
✅ Start simple, grow complex as needed
✅ Code is already organized for splitting
✅ No premature optimization
✅ Can gather user feedback before committing to structure

## Decision

**Use Option 3 (Hybrid)**: Monorepo with feature flags, structured for future workspace migration.

## Rationale

### 1. Tight Coupling of Core Components

All services depend on:
- Same HTTP client instance (connection pooling)
- Same authentication token (shared across services)
- Same type definitions (GeometryType, SpatialRel, LayerId)
- Same error types
- Same geometry conversion logic

Splitting these would create artificial boundaries and force duplication or complex dependency graphs.

### 2. User Experience Priority

```toml
# ✅ Monorepo - Clean and simple
[dependencies]
arcgis = { version = "0.1", features = ["feature-service", "geocoding"] }

# ❌ Workspace - Dependency management burden
[dependencies]
arcgis-core = "0.1.0"
arcgis-types = "0.1.0"
arcgis-auth = "0.1.0"
arcgis-geometry = "0.1.0"
arcgis-feature = "0.2.0"  # Different version!
arcgis-geocoding = "0.1.5"  # Different version!
```

Users want a single import:
```rust
use arcgis::feature::FeatureServiceClient;
use arcgis::geocoding::GeocodeServiceClient;
```

Not:
```rust
use arcgis_feature::FeatureServiceClient;
use arcgis_geocoding::GeocodeServiceClient;
```

### 3. Semver Simplicity

With a monorepo:
- v0.1.0: Initial release with Feature Service
- v0.2.0: Add OAuth (breaking change to auth API)
- v0.3.0: Add Map and Geocoding services (minor bump)
- v1.0.0: Stability guarantee

With a workspace:
- arcgis-core v0.1.0
- arcgis-feature v0.1.0 (depends on core 0.1)
- arcgis-core v0.2.0 (breaking change)
- arcgis-feature v0.2.0 (must update for new core)
- arcgis-map v0.1.0 (depends on core 0.2)
- arcgis-geocoding v0.1.0 (depends on core 0.2)

Now users must track 4+ version numbers.

### 4. Reference Implementations

**aws-sdk-rust**: Uses a workspace but with generated code for 300+ services. They need independent versioning because AWS services version independently.

**google-cloud-rust**: Single crate with features for each GCP service.

**azure-sdk-for-rust**: Workspace, but Azure has 100+ services.

**stripe-rust**: Single crate with feature flags.

**ArcGIS has ~10-15 core services**. This is closer to Stripe than AWS. Single crate is appropriate.

### 5. Type Sharing Problem

Many types are shared:
- `GeometryType` - used in Feature, Map, Geometry services
- `SpatialReference` - used in all services
- `LayerId` - used in Feature, Map services
- `ObjectId` - used in Feature service

In a workspace:
- Put in `arcgis-types` crate
- All services depend on `arcgis-types`
- Changes to types require republishing all services
- Defeats the purpose of splitting

### 6. Future-Proofing

The hybrid approach allows us to:
1. **Start simple**: Single crate, fast iteration
2. **Gather feedback**: Learn what users actually need
3. **Split if needed**: Code structure supports it
4. **No premature optimization**: Don't solve problems we don't have yet

## Implementation Guidelines

### Feature Flag Strategy

```toml
[features]
default = ["feature-service"]

# Services
feature-service = []
map-service = []
geocoding = []
geometry-service = []
routing = []
geoprocessing = []
stream-service = ["tokio/sync"]
places = []

# Formats
pbf = ["arcpbf", "prost"]
geojson-output = []  # Already have geojson, this is for enhanced output

# Convenience
full = [
    "feature-service",
    "map-service",
    "geocoding",
    "geometry-service",
    "routing",
    "geoprocessing",
    "stream-service",
    "places",
    "pbf",
]
```

### Conditional Compilation

```rust
// src/lib.rs
pub mod client;
pub mod error;
pub mod auth;
pub mod types;
pub mod geometry;

#[cfg(feature = "feature-service")]
pub mod feature {
    pub use crate::services::feature::*;
}

#[cfg(feature = "map-service")]
pub mod map {
    pub use crate::services::map::*;
}

#[cfg(feature = "geocoding")]
pub mod geocoding {
    pub use crate::services::geocode::*;
}

// Internal modules always compiled (for re-export)
mod services {
    #[cfg(feature = "feature-service")]
    pub mod feature;

    #[cfg(feature = "map-service")]
    pub mod map;

    #[cfg(feature = "geocoding")]
    pub mod geocode;
}
```

### Module Organization (Workspace-Ready)

```
src/
├── lib.rs                 # Public API surface
│
├── core/                  # Could become arcgis-core
│   ├── mod.rs
│   ├── client.rs          # HTTP client
│   ├── error.rs           # Error types
│   └── auth/              # Authentication
│       ├── mod.rs
│       ├── api_key.rs
│       └── oauth.rs
│
├── types/                 # Could become arcgis-types
│   ├── mod.rs
│   ├── geometry.rs        # GeometryType, SpatialRel
│   ├── spatial.rs         # SpatialReference
│   ├── field.rs           # FieldType
│   └── ids.rs             # LayerId, ObjectId, etc.
│
├── geometry/              # Could become arcgis-geometry
│   ├── mod.rs
│   ├── convert.rs         # geo-types conversions
│   └── serde.rs           # Serde implementations
│
└── services/              # Internal module
    ├── mod.rs
    ├── feature/           # Could become arcgis-feature
    │   ├── mod.rs
    │   ├── types.rs
    │   ├── enums.rs
    │   ├── client.rs
    │   └── query.rs
    ├── map/               # Could become arcgis-map
    └── geocode/           # Could become arcgis-geocoding
```

## When to Split into Workspace

Trigger points for migration:
1. **Size**: Crate exceeds 100,000 lines of code
2. **Compile time**: Clean build takes >5 minutes
3. **Independence**: Clear need to version services separately
4. **Team structure**: Different teams own different services
5. **User demand**: Users consistently request smaller dependency footprint

## Exceptions

If we decide to build **separate** crates:
- `arcgis-cli` - Command-line tool (separate binary)
- `arcgis-mock` - Mock server for testing
- `arcgis-derive` - Derive macros (if we build custom ones)

These have different purposes and release cycles.

## Action Items

1. ✅ Set up monorepo structure with feature flags
2. ✅ Organize code in workspace-ready modules
3. ✅ Document feature flags in README
4. ✅ Add examples for different feature combinations
5. ⏳ Monitor crate size and compile times
6. ⏳ Gather user feedback on structure
7. ⏳ Revisit decision at v0.5.0 milestone

## References

- [Cargo Book - Features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Cargo Book - Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [AWS SDK Rust Structure](https://github.com/awslabs/aws-sdk-rust)
- [Stripe Rust Structure](https://github.com/arlyon/async-stripe)
- [Discussion: Monorepo vs Workspace](https://www.reddit.com/r/rust/comments/pxmxqg/when_to_use_cargo_workspaces/)

---

**Decision Made By**: Implementation Team
**Review Date**: v0.5.0 release (re-evaluate structure)
**Status**: Active
