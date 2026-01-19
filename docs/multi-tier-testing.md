# Multi-Tier Testing Configuration

This document explains the multi-tier API key configuration system used for testing the ArcGIS Rust SDK.

## Overview

The SDK supports multiple privilege tiers for API keys, each providing access to different service levels. Instead of managing multiple `.env` files, the SDK uses a **single `.env` file with multiple API keys**, automatically selecting the correct key based on which test tier you're running.

## Architecture

```
┌─────────────────┐
│  Test Feature   │ (compile-time)
│  test-public    │
│  test-location  │
│  test-portal    │
│  test-publishing│
└────────┬────────┘
         │
         ▼
┌─────────────────────┐
│ config/test-tiers   │ (configuration)
│ .toml               │
│                     │
│ Tier → Env Var Name │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│     .env file       │ (runtime)
│                     │
│ ARCGIS_PUBLIC_KEY   │
│ ARCGIS_LOCATION_KEY │
│ ARCGIS_PORTAL_KEY   │
│ ARCGIS_PUBLISH_KEY  │
└─────────────────────┘
```

## Configuration Files

### config/test-tiers.toml

Maps test tiers to environment variable names:

```toml
[tiers.public]
env_var = "ARCGIS_PUBLIC_KEY"
credits_per_request = 0.0
description = "Public services - no authentication required"

[tiers.location]
env_var = "ARCGIS_LOCATION_KEY"
credits_per_request = 0.004
description = "Location services - geocoding, routing, geometry"

# ... additional tiers
```

This file is:
- **Git-tracked** - Part of the repository
- **Declarative** - Configuration as data, not code
- **Extensible** - Easy to add new tiers

### .env (Single File)

Contains all your API keys:

```bash
# Single .env file with multiple keys
ARCGIS_PUBLIC_KEY=your_public_key_here
ARCGIS_LOCATION_KEY=your_location_key_here
ARCGIS_PORTAL_KEY=your_portal_key_here
ARCGIS_PUBLISH_KEY=your_publishing_key_here
```

This file is:
- **Git-ignored** - Never committed
- **User-specific** - Each developer has their own
- **Single source** - All keys in one place

## Test Tiers

### Tier 1: Public

**Feature flag:** `test-public`
**Environment variable:** `ARCGIS_PUBLIC_KEY`
**Services:** World Cities, public feature services
**Credit cost:** 0 (no auth required)

```bash
cargo test --features test-public
```

### Tier 2: Location

**Feature flag:** `test-location`
**Environment variable:** `ARCGIS_LOCATION_KEY`
**Services:** Geocoding, routing, geometry operations
**Credit cost:** ~0.004 per geocode, ~0.5 per route

```bash
cargo test --features test-location
```

### Tier 3: Portal

**Feature flag:** `test-portal`
**Environment variable:** `ARCGIS_PORTAL_KEY`
**Services:** Portal content management, groups, sharing
**Credit cost:** ~0.001 per operation (storage costs)

```bash
cargo test --features test-portal
```

### Tier 3: Publishing

**Feature flag:** `test-publishing`
**Environment variable:** `ARCGIS_PUBLISH_KEY`
**Services:** Feature service creation, versioning, edit sessions
**Credit cost:** ~0.001 per operation (storage costs)

```bash
cargo test --features test-publishing
```

## Usage in Tests

### Common Helper

All tests use the `common::api_key()` helper which automatically selects the correct environment variable:

```rust
// tests/integration_geocoding.rs
#![cfg(feature = "test-location")]

fn create_client() -> anyhow::Result<ArcGISClient> {
    // Automatically reads ARCGIS_LOCATION_KEY
    let key = common::api_key()?;
    Ok(ArcGISClient::new(ApiKeyAuth::new(key)))
}
```

### How It Works

1. **Compile time:** Feature flag determines active tier (`test-location`)
2. **Configuration:** `config/test-tiers.toml` maps tier → env var name
3. **Runtime:** `common::api_key()` reads the mapped environment variable
4. **Automatic:** No code changes needed per tier

## Setup for Developers

### 1. Copy Template

```bash
cp .env.example .env
```

### 2. Add Your Keys

Edit `.env` and fill in the keys you have:

```bash
# You don't need all keys - only add the ones you have
ARCGIS_PUBLIC_KEY=       # Optional (public services work without auth)
ARCGIS_LOCATION_KEY=     # For geocoding/routing tests
ARCGIS_PORTAL_KEY=       # For portal tests
ARCGIS_PUBLISH_KEY=      # For publishing tests
```

### 3. Create API Keys with Correct Permissions

Go to [developers.arcgis.com/api-keys](https://developers.arcgis.com/api-keys/) and create keys with the following permissions:

#### Tier 1: Public Key (ARCGIS_PUBLIC_KEY) - Optional

Public services work without authentication, but providing a key may increase rate limits.

**Creating the key:**
1. Sign in to [ArcGIS Location Platform](https://location.arcgis.com/) or [ArcGIS Online](https://arcgis.com)
2. Go to **Content > My content > New item > Developer credentials > API key credentials**
3. Name: `SDK Testing - Public`
4. Set expiration date (up to 1 year)
5. **Privileges:** None required (leave all unchecked)
   - _Optional:_ Check "Basemap styles service" for enhanced rate limits

**Note:** This key is optional - public services work without authentication.

#### Tier 2: Location Key (ARCGIS_LOCATION_KEY) - Required for test-location

**Creating the key:**
1. Sign in to [ArcGIS Location Platform](https://location.arcgis.com/) or [ArcGIS Online](https://arcgis.com)
2. Go to **Content > My content > New item > Developer credentials > API key credentials**
3. Name: `SDK Testing - Location Services`
4. Set expiration date (up to 1 year)
5. **Configure privileges - Check these:**

**Location Services** (check all that apply):
- ☑ Basemap styles service
- ☑ Static basemap tiles
- ☑ Geocode (stored)
- ☑ Geocode (not stored)
- ☑ Elevation
- ☑ GeoEnrichment
- ☑ Place finding
- ☑ Routing
- ☑ Service area
- ☑ Closest facility
- ☑ Origin/destination cost matrix
- ☑ Optimized routing
- ☑ Multi-vehicle routing
- ☑ Location allocation
- ☑ Last mile

**Spatial Analysis:**
- ☑ Spatial analysis service

**What this enables:**
- Geocoding (address → coordinates, coordinates → address)
- Routing (routes, directions, travel times)
- Geometry operations (buffer, project, simplify)
- Service area analysis (drive-time polygons)
- Network analysis (closest facility, vehicle routing)

**Credit consumption:**
- Geocoding: ~0.004 credits per geocode
- Simple route: ~0.5 credits per route
- Service area: ~0.5 credits per analysis
- Geometry operations: Varies by complexity

#### Tier 3: Portal Key (ARCGIS_PORTAL_KEY) - Required for test-portal

**Creating the key:**
1. Sign in to [ArcGIS Online](https://arcgis.com) (requires organizational account)
2. Go to **Content > My content > New item > Developer credentials > API key credentials**
3. Name: `SDK Testing - Portal Operations`
4. Set expiration date (up to 1 year)
5. **Configure privileges - Check these:**

**Content:**
- ☑ Create, update, and delete
- ☑ Publish hosted feature layers

**Sharing:**
- ☑ Share with groups
- ☑ Share with organization
- ☑ Share with public (optional)

**Groups:**
- ☑ View organization members
- ☑ View organization groups
- ☑ Join organization groups

**Additional Requirements:**
Your ArcGIS Online account must have these user privileges (configured by your org administrator):

1. Log in to [arcgis.com](https://arcgis.com)
2. Go to **Organization → Members → Your Profile → Privileges**
3. Verify you have:
   - Content: Create, update, delete
   - Publishing: Publish hosted feature layers
   - Groups: Create, update, delete groups
   - Sharing: Share content with groups and organization

**What this enables:**
- Portal content search and discovery
- Item creation/metadata management
- Group creation and administration
- Content sharing and permissions
- Portal item operations

**Credit consumption:**
- Portal operations: ~0.001 credits per operation
- Storage costs apply (check your organization's quota)

#### Tier 3: Publishing Key (ARCGIS_PUBLISH_KEY) - Required for test-publishing

**Creating the key:**
1. Sign in to ArcGIS Enterprise portal (11.2+) or [ArcGIS Online](https://arcgis.com)
2. Go to **Content > My content > New item > Developer credentials > API key credentials**
3. Name: `SDK Testing - Publishing`
4. Set expiration date (up to 1 year)
5. **Configure privileges - Check these:**

**Content:**
- ☑ Create, update, and delete
- ☑ Publish hosted feature layers
- ☑ Publish hosted tile layers (optional)
- ☑ Publish hosted scene layers (optional)

**Features:**
- ☑ Edit
- ☑ Edit with full control

**Sharing:**
- ☑ Share with groups
- ☑ Share with organization

**Additional Requirements for ArcGIS Enterprise:**
- ArcGIS Enterprise 11.2 or later
- Version Management Server configured
- User account privileges (configured by org administrator):
  - Publish hosted feature layers
  - Create and manage feature layer views
  - Enable branch versioning on feature layers
  - Manage versions

**To verify publishing permissions:**
1. Log in to your ArcGIS Enterprise portal
2. Go to **Organization → Members → Your Profile → Privileges**
3. Verify you have:
   - Publishing: Publish hosted layers
   - Features: Full editing control
   - Versioning: Create and manage versions

**What this enables:**
- Feature service creation and publishing
- Edit sessions with transaction support
- Branch-versioned editing workflows
- Version management operations
- Multi-user editing with conflict detection

**Credit consumption:**
- Publishing operations: ~0.001 credits per operation
- Storage costs for hosted layers
- Compute costs for edit sessions and version reconciliation

### 4. Verify Your Keys

After creating keys, verify they work:

```bash
# Test public key (optional)
ARCGIS_PUBLIC_KEY=your_key cargo test --features test-public test_credentials_available

# Test location key
ARCGIS_LOCATION_KEY=your_key cargo test --features test-location test_credentials_available

# Test portal key (requires org account)
ARCGIS_PORTAL_KEY=your_key cargo test --features test-portal test_credentials_available

# Test publishing key (requires enterprise)
ARCGIS_PUBLISH_KEY=your_key cargo test --features test-publishing test_credentials_available
```

### 5. Run Tests

```bash
# Run tests for the tier you have keys for
cargo test --features test-public    # No key required
cargo test --features test-location  # Requires ARCGIS_LOCATION_KEY
```

## Benefits

### Single File Management

✅ **One file to manage** - All keys in `.env`
✅ **No file switching** - Don't juggle `.env_public`, `.env_location`, etc.
✅ **Easy to share templates** - `.env.example` shows all options

### Configuration-Driven

✅ **Declarative mapping** - Config file, not hardcoded logic
✅ **Easy to extend** - Add new tier in TOML, no code changes
✅ **Battle-tested** - Uses `config` crate (industry standard)

### Developer Experience

✅ **Clear error messages** - "ARCGIS_LOCATION_KEY not found (required for tier: location)"
✅ **Self-documenting** - Config file shows all tiers and requirements
✅ **Flexible** - Have all keys or just the ones you need

## Adding a New Tier

To add a new test tier (e.g., `test-analytics`):

### 1. Add Feature to Cargo.toml

```toml
[features]
test-analytics = []
```

### 2. Add Configuration

```toml
# config/test-tiers.toml
[tiers.analytics]
env_var = "ARCGIS_ANALYTICS_KEY"
credits_per_request = 0.005
description = "Analytics services - spatial analysis"
```

### 3. Update .env.example

```bash
# Tier 2: Analytics services
ARCGIS_ANALYTICS_KEY=
```

### 4. Update Helper

```rust
// tests/common/mod.rs
fn active_tier() -> &'static str {
    if cfg!(feature = "test-analytics") { "analytics" }
    // ... existing tiers
}
```

That's it! No other code changes needed.

## Troubleshooting

### Error: "Environment variable ARCGIS_LOCATION_KEY not found"

**Problem:** Running `test-location` tests without the required key.

**Solution:** Add the key to your `.env` file:
```bash
ARCGIS_LOCATION_KEY=your_key_here
```

### Error: "Failed to load config/test-tiers.toml"

**Problem:** Config file missing or malformed.

**Solution:** Ensure `config/test-tiers.toml` exists in repository root and is valid TOML.

### All tests fail with key errors

**Problem:** No `.env` file or empty keys.

**Solution:**
```bash
cp .env.example .env
# Edit .env and add at least one key
```

## Best Practices

### For SDK Developers

- ✅ Keep all your keys in a single `.env` file
- ✅ Use the helper: `common::api_key()` in all tests
- ✅ Document credit costs in test tier config
- ✅ Update `.env.example` when adding new tiers

### For CI/CD

- ✅ Use GitHub Secrets for keys
- ✅ Template `.env` file in CI workflow
- ✅ Run only the tiers with available keys
- ✅ Skip expensive tiers (portal, publishing) in PR builds

### For Users

- ✅ Start with `test-public` (no key required)
- ✅ Add keys incrementally as needed
- ✅ Monitor credit usage in ArcGIS Dashboard
- ✅ Use separate keys for dev/prod environments

## References

- [API Key Testing Strategy](./API_KEY_TESTING_STRATEGY.md)
- [.env.example](../.env.example)
- [config/test-tiers.toml](../config/test-tiers.toml)
- [config crate documentation](https://docs.rs/config/)
