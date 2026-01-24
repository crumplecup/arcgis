# Integration Tests

## Setup

Integration tests require ArcGIS credentials and the `api` feature flag.

### 1. Create `.env` file

Copy the example file:

```bash
cp .env.example .env
```

### 2. Add your credentials

Edit `.env` and add either:

**Option A: API Key** (recommended for read-only testing):
```bash
ARCGIS_API_KEY=your_api_key_here
```

Get an API key from: https://developers.arcgis.com/dashboard

**Option B: OAuth Credentials** (for full read/write testing):
```bash
ARCGIS_CLIENT_ID=your_client_id_here
ARCGIS_CLIENT_SECRET=your_client_secret_here
```

### 3. Run integration tests

Run all integration tests (hits live AGOL):

```bash
cargo test --features api
```

Or use the justfile recipe:

```bash
just test-api
```

Run specific test:

```bash
cargo test --features api test_public_feature_service_accessible
```

## Rate Limiting

Tests include `rate_limit()` calls to be polite to the ArcGIS Online API. Do not remove these delays.

## Target Environment

All integration tests target **ArcGIS Online (AGOL)**, not ArcGIS Enterprise.

## Test Organization

- `tests/common/` - Shared utilities, credential loading
- `tests/integration_basic.rs` - Basic connectivity tests
- `tests/integration_feature_service.rs` - Feature Service tests (TODO)
- `tests/integration_geocoding.rs` - Geocoding tests (TODO)

## Public Test Data

Some tests use ESRI's public sample data:
- World Cities Feature Service (read-only, no auth required)

These tests verify basic API structure without requiring credentials.
