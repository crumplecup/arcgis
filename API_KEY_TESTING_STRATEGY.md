# API Key Testing Strategy

**Last Updated**: 2026-01-11

---

## Executive Summary

This document outlines our testing strategy for the ArcGIS Rust SDK, focusing on API key management, privilege scoping, and security best practices. Our approach follows industry standards for API testing while accounting for ArcGIS's granular permission system.

**Key Principles**:
1. **Principle of Least Privilege**: Each API key has only the permissions needed for specific tests
2. **Environmental Isolation**: Separate keys for different test suites
3. **No Personal Scope in CI**: Avoid personal-scope privileges in automated tests
4. **Key Rotation**: Support multiple concurrent keys for zero-downtime rotation
5. **Public Test Compatibility**: Tests use public services where possible to avoid requiring API keys

---

## ArcGIS Privilege System

### Understanding Scopes

ArcGIS API keys use a **privilege-based** permission system with two scope types:

**Standard Scope** (Safe for public applications):
- Location services (basemaps, routing, geocoding, places, elevation)
- Spatial analysis services
- Public data access
- **✅ Safe for CI/CD** - Can be embedded in public repositories with rate limiting

**Personal Scope** (Private applications only):
- Portal operations (user management, content CRUD, groups)
- Publishing services
- Administrative operations
- **❌ Security Risk** - Should never be exposed in public applications or CI

### Complete Privilege List

#### Location Services (Standard Scope)

| Privilege | Service | Required For |
|-----------|---------|--------------|
| `premium:user:basemaps` | Basemap styles | Vector tile tests |
| `premium:user:staticbasemaptiles` | Static tiles | Map service tests |
| `premium:user:geocode:stored` | Geocoding (stored) | Geocoding tests (with storage) |
| `premium:user:geocode:temporary` | Geocoding (temporary) | Geocoding tests (no storage) |
| `premium:user:places` | Places service | Places API tests |
| `premium:user:elevation` | Elevation service | Elevation API tests |
| `premium:user:geoenrichment` | GeoEnrichment | Demographics tests |
| `premium:user:networkanalysis:routing` | Routing | Route tests |
| `premium:user:networkanalysis:servicearea` | Service area | Drive-time polygon tests |
| `premium:user:networkanalysis:closestfacility` | Closest facility | Facility analysis tests |
| `premium:user:networkanalysis:origindestinationcostmatrix` | OD matrix | Cost matrix tests |
| `premium:user:networkanalysis:optimizedrouting` | Fleet routing | Optimized route tests |
| `premium:user:networkanalysis:vehiclerouting` | Vehicle routing | Fleet tests |
| `premium:user:networkanalysis:locationallocation` | Location allocation | Allocation tests |
| `premium:user:networkanalysis:lastmiledelivery` | Last mile | Delivery tests |
| `premium:user:spatialanalysis` | Spatial analysis | Analysis tests |
| `premium:publisher:rasteranalysis` | Raster analysis | Image analysis tests |

#### Portal Services (Personal Scope - ⚠️ Use with Caution)

| Privilege | Service | Required For |
|-----------|---------|--------------|
| `portal:user:createItem` | Content management | Portal item CRUD tests |
| `portal:user:viewOrgItems` | Content viewing | Portal search tests |
| `portal:user:shareToGroup` | Sharing | Sharing tests |
| `portal:user:shareToOrg` | Organization sharing | Org-wide sharing tests |
| `portal:user:shareToPublic` | Public sharing | Public sharing tests |
| `portal:user:createGroup` | Group management | Group CRUD tests |
| `portal:user:joinGroup` | Group membership | Group join tests |
| `portal:user:viewOrgGroups` | Group viewing | Group search tests |
| `portal:publisher:publishFeatures` | Feature publishing | Publish tests |
| `portal:publisher:publishTiles` | Tile publishing | Tile publish tests |
| `portal:publisher:publishScenes` | Scene publishing | 3D publish tests |
| `portal:user:generateApiTokens` | API token generation | Token management tests |

#### Admin Privileges (Personal Scope - ❌ Never Use in Tests)

All `portal:admin:*` privileges should **never** be used in automated tests:
- User management (`portal:admin:updateUsers`, `portal:admin:deleteUsers`)
- Group administration (`portal:admin:deleteGroups`, `portal:admin:reassignGroups`)
- Content administration (`portal:admin:deleteItems`, `portal:admin:reassignItems`)
- Security management (`portal:admin:manageSecurity`)
- Infrastructure management (`portal:admin:manageServers`)

**Rationale**: Admin privileges can permanently modify or delete organizational resources. Tests should never have this level of access.

---

## Testing Strategy

### Test Classification

We organize tests into three tiers based on authentication requirements:

**Tier 1: Public Service Tests** (No API key required)
- Feature service queries against public datasets
- Map service exports from public services
- Geocoding with public locators
- **Implementation**: Use known public ArcGIS Online services
- **Coverage**: ~40% of test suite
- **CI/CD**: Run on every commit

**Tier 2: Standard Scope Tests** (API key with standard privileges)
- Location services (routing, places, elevation)
- Geocoding with storage
- Spatial analysis operations
- Vector tile access
- **Implementation**: Single API key with all standard-scope privileges
- **Coverage**: ~50% of test suite
- **CI/CD**: Run on PR merge to main

**Tier 3: Personal Scope Tests** (API key with personal privileges)
- Portal item CRUD operations
- Content publishing
- Group management
- Sharing operations
- **Implementation**: Separate API key with minimal personal privileges
- **Coverage**: ~10% of test suite
- **CI/CD**: Run manually or on release branches only
- **⚠️ Warning**: Uses real portal resources, may incur costs

### API Key Strategy

We use **four separate API keys** with isolated scopes:

#### Key 1: Public Testing (No Key)
```
Name: None (uses public services)
Scope: None
Used for: Feature queries, map exports, public geocoding
Environment: CI/CD (all branches)
Rotation: N/A
```

#### Key 2: Location Services Testing
```
Name: ARCGIS_LOCATION_KEY
Scope:
  ✅ premium:user:basemaps
  ✅ premium:user:geocode:temporary
  ✅ premium:user:places
  ✅ premium:user:elevation
  ✅ premium:user:networkanalysis:routing
  ✅ premium:user:networkanalysis:servicearea
  ✅ premium:user:networkanalysis:closestfacility
  ✅ premium:user:networkanalysis:origindestinationcostmatrix
  ✅ premium:user:spatialanalysis
  ❌ NO portal privileges
  ❌ NO admin privileges
Used for: Routing, geocoding, places, elevation, spatial analysis
Environment: CI/CD (main branch + releases)
Rotation: Monthly
Cost: Consumes credits (track usage)
```

#### Key 3: Portal Testing (Minimal Personal Scope)
```
Name: ARCGIS_PORTAL_KEY
Scope:
  ✅ portal:user:createItem
  ✅ portal:user:viewOrgItems
  ✅ portal:user:createGroup
  ✅ portal:user:joinGroup
  ✅ portal:user:shareToGroup
  ❌ NO shareToPublic (prevent public leaks)
  ❌ NO admin privileges
  ❌ NO publishing privileges (separate key)
Used for: Portal item/group CRUD, search, sharing
Environment: Manual testing only
Rotation: Monthly
Cost: May create portal items (cleanup required)
```

#### Key 4: Publishing Testing (Restricted Personal Scope)
```
Name: ARCGIS_PUBLISH_KEY
Scope:
  ✅ portal:user:createItem
  ✅ portal:publisher:publishFeatures
  ✅ portal:publisher:publishTiles
  ❌ NO admin privileges
  ❌ NO shareToPublic
Used for: Service publishing tests
Environment: Manual testing only
Rotation: Monthly
Cost: Creates hosted services (cleanup critical!)
```

### Key Management Best Practices

**1. Environmental Isolation**
```bash
# Development (.env.development)
ARCGIS_LOCATION_KEY=dev_location_key_...
ARCGIS_PORTAL_KEY=dev_portal_key_...

# CI/CD (GitHub Secrets)
ARCGIS_LOCATION_KEY=ci_location_key_...
# Note: No portal key in CI

# Production (never commit)
# Production keys should never exist for testing
```

**2. Key Rotation Support**
```rust
// Support multiple concurrent keys for zero-downtime rotation
pub struct ApiKeyAuth {
    primary_key: String,
    fallback_key: Option<String>, // Used during rotation
}

impl ApiKeyAuth {
    pub fn with_fallback(primary: String, fallback: String) -> Self {
        Self {
            primary_key: primary,
            fallback_key: Some(fallback),
        }
    }
}
```

**3. Rate Limiting & Credit Tracking**
```rust
// Track credit consumption in tests
#[cfg(feature = "api")]
pub struct CreditTracker {
    consumed: Arc<Mutex<f64>>,
}

impl CreditTracker {
    pub fn record(&self, operation: &str, credits: f64) {
        // Log credit usage for cost monitoring
        tracing::info!(
            operation = operation,
            credits = credits,
            "API credits consumed"
        );
    }
}
```

**4. Secret Detection Prevention**
```gitignore
# .gitignore - Prevent accidental commits
.env
.env.*
!.env.example
*.key
*_key.txt
arcgis_credentials.json
```

**5. Key Validation**
```rust
#[cfg(test)]
fn validate_api_key_scope() {
    // Ensure test keys don't have dangerous privileges
    let key = env::var("ARCGIS_PORTAL_KEY").ok();
    if let Some(key) = key {
        // Key should be scoped appropriately
        assert!(key.starts_with("AAPK"), "Invalid API key format");

        // Check key doesn't have admin scope (requires introspection API)
        // This is a placeholder - actual implementation would query ArcGIS
    }
}
```

---

## Test Organization

### Directory Structure

```
tests/
├── public/                      # Tier 1: No authentication
│   ├── feature_query_test.rs   # Public feature service queries
│   ├── map_export_test.rs      # Public map exports
│   └── geocode_basic_test.rs   # Public geocoding
│
├── location/                    # Tier 2: Standard scope (ARCGIS_LOCATION_KEY)
│   ├── routing_test.rs         # Routing operations
│   ├── places_test.rs          # Places API
│   ├── elevation_test.rs       # Elevation service
│   ├── geocode_stored_test.rs  # Geocoding with storage
│   └── spatial_analysis_test.rs
│
├── portal/                      # Tier 3: Personal scope (ARCGIS_PORTAL_KEY)
│   ├── item_crud_test.rs       # Item create/update/delete
│   ├── group_crud_test.rs      # Group management
│   ├── search_test.rs          # Portal search
│   └── sharing_test.rs         # Sharing operations
│
└── publishing/                  # Tier 3: Publishing scope (ARCGIS_PUBLISH_KEY)
    ├── publish_features_test.rs
    ├── publish_tiles_test.rs
    └── cleanup.rs               # Critical: Delete test services after run
```

### Feature Flags

```toml
# Cargo.toml
[features]
default = []

# Test tiers
test-public = []         # Tier 1: No key required
test-location = []       # Tier 2: ARCGIS_LOCATION_KEY
test-portal = []         # Tier 3: ARCGIS_PORTAL_KEY (manual only)
test-publishing = []     # Tier 3: ARCGIS_PUBLISH_KEY (manual only)

# Legacy (deprecated)
api = ["test-public", "test-location"]  # Backward compatibility
```

### Running Tests

```bash
# Run all public tests (CI/CD safe, no key needed)
cargo test --features test-public

# Run location service tests (requires ARCGIS_LOCATION_KEY)
ARCGIS_LOCATION_KEY=your_key cargo test --features test-location

# Run portal tests (manual only, requires ARCGIS_PORTAL_KEY)
ARCGIS_PORTAL_KEY=your_key cargo test --features test-portal

# Run all non-publishing tests
cargo test --features test-public,test-location,test-portal

# DANGEROUS: Run publishing tests (creates hosted services!)
ARCGIS_PUBLISH_KEY=your_key cargo test --features test-publishing
```

### Test Annotations

```rust
// Tier 1: Public - runs in CI
#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_public_feature_service() {
    // No authentication needed
}

// Tier 2: Location - runs on main branch CI
#[tokio::test]
#[cfg(feature = "test-location")]
async fn test_routing_service() {
    let key = env::var("ARCGIS_LOCATION_KEY")
        .expect("ARCGIS_LOCATION_KEY required for location tests");
    // Test routing with standard scope key
}

// Tier 3: Portal - manual only
#[tokio::test]
#[cfg(feature = "test-portal")]
#[ignore] // Ignored by default - run with --ignored
async fn test_create_portal_item() {
    let key = env::var("ARCGIS_PORTAL_KEY")
        .expect("ARCGIS_PORTAL_KEY required for portal tests");
    // Creates real portal items - cleanup required!
}
```

---

## Service-Specific Requirements

### Feature Service
**Public Tests**: ✅ Query public datasets (no key)
**Requires Key**: Publishing, editing private layers
**Privileges Needed**: `portal:publisher:publishFeatures` (publishing only)

### Map Service
**Public Tests**: ✅ Export public maps (no key)
**Requires Key**: Export private maps, dynamic layers
**Privileges Needed**: `premium:user:staticbasemaptiles` (if using basemaps)

### Geocoding Service
**Public Tests**: ✅ Single address geocoding
**Requires Key**: Batch geocoding, geocoding with storage
**Privileges Needed**:
- `premium:user:geocode:temporary` (single addresses, no storage)
- `premium:user:geocode:stored` (batch, persistent storage)

### Geometry Service
**Public Tests**: ✅ All operations (public service)
**Requires Key**: None (uses ArcGIS Online public geometry service)
**Privileges Needed**: None

### Routing Service
**Public Tests**: ❌ None (requires authentication)
**Requires Key**: All operations
**Privileges Needed**:
- `premium:user:networkanalysis:routing` (basic routes)
- `premium:user:networkanalysis:servicearea` (drive-time polygons)
- `premium:user:networkanalysis:closestfacility` (facility analysis)
- `premium:user:networkanalysis:origindestinationcostmatrix` (OD matrix)

### Places Service
**Public Tests**: ❌ None (requires authentication)
**Requires Key**: All operations
**Privileges Needed**: `premium:user:places`

### Elevation Service
**Public Tests**: ❌ None (requires authentication)
**Requires Key**: All operations
**Privileges Needed**: `premium:user:elevation`

### Geoprocessing Service
**Public Tests**: ✅ Public GP services (no key)
**Requires Key**: Private/secured GP services
**Privileges Needed**: Depends on GP service requirements

### Image Service
**Public Tests**: ✅ Public image services (no key)
**Requires Key**: Private image services, advanced analysis
**Privileges Needed**: `premium:publisher:rasteranalysis` (for analysis)

### Vector Tile Service
**Public Tests**: ✅ Public vector tiles (no key)
**Requires Key**: Private tile services, basemap styles
**Privileges Needed**: `premium:user:basemaps` (for basemap styles)

### Portal Service
**Public Tests**: ❌ None (all operations personal scope)
**Requires Key**: All operations
**Privileges Needed**:
- `portal:user:createItem` (CRUD)
- `portal:user:viewOrgItems` (search)
- `portal:user:createGroup` (groups)
- `portal:user:shareToGroup` (sharing)
- `portal:publisher:publishFeatures` (publishing)

### Version Management Service
**Public Tests**: ❌ None (enterprise feature)
**Requires Key**: OAuth 2.0 (not API key)
**Privileges Needed**: Enterprise server access (not API key based)

---

## CI/CD Pipeline

### GitHub Actions Workflow

```yaml
name: API Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  # Tier 1: Always run (no secrets needed)
  public-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run public tests
        run: cargo test --features test-public

  # Tier 2: Run on main branch only (requires secret)
  location-tests:
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run location service tests
        env:
          ARCGIS_LOCATION_KEY: ${{ secrets.ARCGIS_LOCATION_KEY }}
        run: cargo test --features test-location

  # Tier 3: Never run in CI (manual only)
  # portal-tests and publishing-tests deliberately omitted
```

### Secret Configuration

**GitHub Repository Secrets**:
- `ARCGIS_LOCATION_KEY`: Location services key (standard scope only)

**Never Store**:
- ❌ Portal keys in GitHub Secrets (personal scope = security risk)
- ❌ Publishing keys anywhere (manual testing only)
- ❌ Admin keys (should never exist for testing)

---

## Cost Management

### Credit Consumption Tracking

```rust
pub struct CreditEstimate {
    operation: String,
    estimated_credits: f64,
}

impl CreditEstimate {
    // ArcGIS Online credit costs (2026)
    pub const GEOCODE_STORED: f64 = 0.04;      // per geocode
    pub const GEOCODE_TEMPORARY: f64 = 0.0;     // free
    pub const ROUTE_SIMPLE: f64 = 0.005;        // per route
    pub const SERVICE_AREA: f64 = 0.005;        // per polygon
    pub const CLOSEST_FACILITY: f64 = 0.005;    // per analysis
    pub const ELEVATION_PROFILE: f64 = 0.0;     // free tier available
    pub const PLACES_SEARCH: f64 = 0.004;       // per search
}

#[cfg(feature = "test-location")]
#[tokio::test]
async fn test_with_credit_tracking() {
    let tracker = CreditTracker::new();

    // Perform operation
    let result = service.route(...).await?;

    // Track credits
    tracker.record("route", CreditEstimate::ROUTE_SIMPLE);

    // Assert on results...
}
```

### Monthly Budget

**Recommended Test Budget**: 100 credits/month
- Public tests: 0 credits (free)
- Location tests (CI): ~10 credits/month (main branch only)
- Portal tests (manual): ~5 credits/month (infrequent)
- Publishing tests (manual): ~10 credits/month (cleanup prevents accumulation)

**Budget Alerts**:
- Set ArcGIS Online credit alerts at 50, 75, 100 credits
- Monitor via ArcGIS Online organization dashboard
- Log credit consumption in tests for auditing

---

## Security Best Practices

### 1. Principle of Least Privilege
✅ **DO**: Create separate keys for each test tier with minimal required privileges
❌ **DON'T**: Use a single key with all privileges for all tests

### 2. Environmental Isolation
✅ **DO**: Use different keys for dev, CI, and manual testing
❌ **DON'T**: Share keys across environments

### 3. No Personal Scope in CI
✅ **DO**: Only use standard-scope privileges in automated CI/CD
❌ **DON'T**: Put personal-scope keys in GitHub Secrets or CI variables

### 4. Secret Detection
✅ **DO**: Use `.gitignore` and secret scanners (like `gitleaks`, `truffleHog`)
❌ **DON'T**: Commit keys to version control (even in private repos)

### 5. Key Rotation
✅ **DO**: Rotate keys monthly and support concurrent key fallback
❌ **DON'T**: Use the same key indefinitely

### 6. Rate Limiting
✅ **DO**: Implement retry backoff and respect rate limits
❌ **DON'T**: Hammer the API with rapid-fire test requests

### 7. Cleanup After Tests
✅ **DO**: Delete portal items/services created during tests
❌ **DON'T**: Leave test resources accumulating in your organization

### 8. Monitoring & Auditing
✅ **DO**: Log API key usage, track credit consumption
❌ **DON'T**: Run tests blindly without cost/usage awareness

### 9. Public Service Preference
✅ **DO**: Use public ArcGIS Online services for tests when possible
❌ **DON'T**: Require authentication for tests that could use public data

### 10. Never Use Admin Privileges
✅ **DO**: Test administrative features with mocked services
❌ **DON'T**: Create API keys with `portal:admin:*` privileges for testing

---

## Implementation Checklist

### Phase 1: Setup (Week 1)
- [ ] Create 4 API keys in ArcGIS Online with documented scopes
- [ ] Add keys to `.env.example` with documentation
- [ ] Configure GitHub Secrets for `ARCGIS_LOCATION_KEY`
- [ ] Set up secret scanning (gitleaks pre-commit hook)
- [ ] Document key rotation schedule

### Phase 2: Test Organization (Week 2)
- [ ] Reorganize tests into `public/`, `location/`, `portal/`, `publishing/` dirs
- [ ] Add feature flags: `test-public`, `test-location`, `test-portal`, `test-publishing`
- [ ] Annotate existing tests with appropriate feature flags
- [ ] Add `#[ignore]` to all personal-scope tests

### Phase 3: CI/CD Integration (Week 3)
- [ ] Update GitHub Actions to run public tests on all branches
- [ ] Add location tests for main branch only
- [ ] Remove personal-scope tests from CI
- [ ] Add credit consumption logging to location tests

### Phase 4: Documentation (Week 4)
- [ ] Document API key creation process in `CONTRIBUTING.md`
- [ ] Add testing guide to `tests/README.md`
- [ ] Create troubleshooting guide for test failures
- [ ] Document cost estimates and budget management

### Phase 5: Monitoring (Ongoing)
- [ ] Set up credit usage alerts in ArcGIS Online
- [ ] Monthly key rotation schedule
- [ ] Quarterly review of test coverage vs cost
- [ ] Annual audit of privilege scopes

---

## Troubleshooting

### Test Fails: "Invalid API Key"
**Cause**: Key expired, invalid, or insufficient privileges
**Solution**:
1. Verify key is valid: `curl https://www.arcgis.com/sharing/rest/community/self?f=json&token=YOUR_KEY`
2. Check key privileges in ArcGIS Online dashboard
3. Ensure key is for correct environment (dev vs CI)

### Test Fails: "Insufficient Privileges"
**Cause**: API key missing required privilege for operation
**Solution**:
1. Check service-specific requirements in this document
2. Add required privilege to API key in ArcGIS Online
3. Wait 5 minutes for privilege changes to propagate

### CI Fails: "Environment Variable Not Set"
**Cause**: GitHub Secret not configured or incorrect name
**Solution**:
1. Verify secret exists in repo settings
2. Check secret name matches exactly (case-sensitive)
3. Ensure workflow has access to secrets (not on forks)

### High Credit Consumption
**Cause**: Tests running too frequently or inefficiently
**Solution**:
1. Review credit logs: `RUST_LOG=info cargo test`
2. Reduce test frequency (run location tests on main only)
3. Use mocked responses for rapid iteration
4. Consider reducing test dataset sizes

---

## References

### Official Documentation
- [ArcGIS API Key Authentication](https://developers.arcgis.com/documentation/mapping-apis-and-services/security/api-keys/)
- [API Key Credentials Guide](https://developers.arcgis.com/documentation/security-and-authentication/api-key-authentication/api-key-credentials/)
- [Complete Privilege Reference](https://developers.arcgis.com/documentation/security-and-authentication/reference/privileges/)
- [Tutorial: Manage API Key Credentials](https://developers.arcgis.com/documentation/security-and-authentication/api-key-authentication/tutorials/manage-api-key-credentials/)
- [API Key Legacy Retirement Notice](https://developers.arcgis.com/documentation/security-and-authentication/api-key-authentication/api-key-legacy/)

### Industry Best Practices
- [API Key Management Best Practices](https://multitaskai.com/blog/api-key-management-best-practices/)
- [API Key Security Best Practices for 2026](https://dev.to/alixd/api-key-security-best-practices-for-2026-1n5d)
- [Securing APIs: Guide to API Keys & Scopes](https://fusionauth.io/blog/securing-your-api)
- [Google Maps Platform Security Guidance](https://developers.google.com/maps/api-security-best-practices)
- [Building Secure APIs in 2026](https://acmeminds.com/building-secure-apis-in-2026-best-practices-for-authentication-and-authorization/)

### ArcGIS Credits & Pricing
- [ArcGIS Online Service Credits Overview](https://developers.arcgis.com/documentation/mapping-apis-and-services/deployment/service-credits/)
- [Location Services Pricing](https://developers.arcgis.com/pricing/)

---

## Appendix: Key Creation Guide

### Creating API Keys in ArcGIS Online

**Step 1**: Log in to [ArcGIS Developers Dashboard](https://developers.arcgis.com/dashboard)

**Step 2**: Navigate to API Keys → Create API Key

**Step 3**: Configure Key Scopes

For **ARCGIS_LOCATION_KEY**:
```
Name: SDK Location Services Testing
Description: Rust SDK CI/CD - Location services only (routing, geocoding, places, elevation)

Privileges (Standard Scope):
  ☑ Basemaps
  ☑ Geocoding (not stored)
  ☑ Places
  ☑ Elevation
  ☑ Routing
  ☑ Service area
  ☑ Closest facility
  ☑ Origin destination cost matrix
  ☑ Spatial analysis

Referrers: *.github.com/* (restrict to GitHub Actions)
Expiration: 1 year
```

For **ARCGIS_PORTAL_KEY** (Manual Testing Only):
```
Name: SDK Portal Testing (Manual)
Description: Rust SDK manual testing - Portal operations only

Privileges (Personal Scope - MINIMAL):
  ☑ Create, update, and delete content
  ☑ View items shared with the organization
  ☑ Create groups
  ☑ Join groups
  ☑ Share content to groups
  ☐ Share content to organization (disabled)
  ☐ Share content publicly (disabled - security)

Referrers: localhost:* (local testing only)
Expiration: 3 months
```

**Step 4**: Save and Document

- Copy API key immediately (shown only once)
- Store in password manager (1Password, LastPass, etc.)
- Add to `.env` file (never commit)
- Document key purpose and scope in team wiki

**Step 5**: Test Key

```bash
# Verify key works
curl "https://www.arcgis.com/sharing/rest/community/self?f=json&token=YOUR_KEY"

# Should return JSON with user info (not error)
```

---

## Change Log

| Date | Change | Reason |
|------|--------|--------|
| 2026-01-11 | Initial version | Establish testing strategy with privilege-based scoping |
