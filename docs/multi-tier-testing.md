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

### 3. Get Keys

- **Free tier keys:** [developers.arcgis.com/api-keys](https://developers.arcgis.com/api-keys/)
- **OAuth credentials:** [developers.arcgis.com/applications](https://developers.arcgis.com/applications/)

### 4. Run Tests

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
