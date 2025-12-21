# ArcGIS REST API Research Document

## Executive Summary

This document provides detailed research on the ESRI ArcGIS REST API specification to inform the development of a Rust SDK. As of 2025, ESRI does not provide an official Rust SDK for ArcGIS, presenting an opportunity to create a community-driven implementation.

**Key Findings**:
- ArcGIS REST APIs lack official OpenAPI specifications - **code generation is not viable**
- **Hand-written implementation** is required for all service wrappers
- Leverage existing Rust ecosystem heavily to avoid reinventing the wheel:
  - **GeoRust** (`geo-types`, `geojson`, `geo`) for all spatial types
  - **oauth2 crate** for authentication flows
  - **reqwest + tokio** for async HTTP
- A layered architecture maximizes code reuse and maintainability
- Comprehensive documentation reading and translation to Rust types is the primary effort

## Official Documentation

### Primary Resources

- **Main Documentation Hub**: [developers.arcgis.com/rest/](https://developers.arcgis.com/rest/)
- **Services Reference**: [ArcGIS Server Services Directory REST API](https://developers.arcgis.com/rest/services-reference/)
- **Glossary**: [ArcGIS REST APIs](https://developers.arcgis.com/documentation/glossary/arcgis-rest-apis/)

The ArcGIS REST APIs are the API specifications for all ArcGIS location services and ArcGIS Enterprise services, defining the operations, parameters, and structures required to make HTTPS requests.

### Recent Updates (2025)

- **Release 2.5.0** (February 10, 2025): Added support for 'no' language code for Norway
- **Release 2.10.0** (September 2, 2025): Added support for Basemap Sessions, permitting requests for basemap tiles for up to 12 hours for developers with an ArcGIS Location Platform account

## Core API Components

### 1. Service Types

#### Feature Services

A feature service is one of the most important service types for a Rust SDK implementation.

**Key Characteristics**:
- Allows clients to query and edit feature geometry and attributes
- Can contain datasets (tables/views) with or without a spatial column
- Datasets with spatial columns are "layers"; those without are "tables"
- Returns basic information about the feature service, including layers and tables

**Capabilities**:
- Service-level: Create, Delete, Extract, Query, Update, Sync, Uploads
- Layer-level: Query, Create, Delete, Update, Editing, Sync, Uploads, Extract
  - Editing capability is included if Create, Delete, or Update is enabled

**Operations**:
- Add Features
- Append
- Apply Edits
- Calculate
- Delete Features
- Generate Renderer
- Query
- Query Attachments
- Query Top Features
- Query Related Records
- Update Features
- Validate SQL

**Documentation**: [Feature Service](https://developers.arcgis.com/rest/services-reference/enterprise/feature-service/)

#### Map Services

Map services provide map visualization capabilities with different behavior based on hosting.

**Hosted Services (ArcGIS Online/Portal)**:
- Can only return tiles from the server's cache
- Cannot draw images dynamically
- Do not allow query of individual features
- Best used in conjunction with feature services for queries and pop-ups

**Server-Hosted Services (ArcGIS Server)**:
- Support a larger set of operations
- Export map - Exports a map image from a dynamic map service
- More flexible rendering capabilities

**Documentation**: [Map Service](https://developers.arcgis.com/rest/services-reference/enterprise/map-service/)

### 2. Authentication

ArcGIS supports three authentication types that a Rust SDK must implement:

#### API Key Authentication
- For building public apps with location services
- Simplest authentication method
- Suitable for client-side applications

#### User Authentication (OAuth 2.0)
- For creating private apps for organizations
- Requires OAuth 2.0 flow implementation
- Users sign in to their ArcGIS organization

**OAuth Flow**:
1. Application guides user to login page hosted by ArcGIS organization
2. User authenticates
3. Application receives access token on behalf of the user
4. Token is used to access ArcGIS organization resources

**Key Endpoints**:
- `/oauth2/authorize` - Initiates OAuth flow
- `/oauth2/token` - Grants access token

**Token Grant Types**:
- `authorization_code` - Standard OAuth flow
- `client_credentials` - Server-to-server authentication
- `exchange_refresh_token` - Exchange refresh token for access token
- `refresh_token` - Refresh an expired access token

#### App Authentication
- For server-enabled applications
- Works with ArcGIS Enterprise resources
- Client credentials flow

**Documentation**:
- [Authentication Overview](https://developers.arcgis.com/rest/users-groups-and-items/authentication/)
- [REST Authentication Operations](https://developers.arcgis.com/documentation/security-and-authentication/reference/rest-authentication-operations/)
- [Token Endpoint](https://developers.arcgis.com/rest/users-groups-and-items/token/)
- [Authorize Endpoint](https://developers.arcgis.com/rest/users-groups-and-items/authorize/)

### 3. Service Endpoints

ArcGIS services support two types of endpoints:

**Standard Endpoints**:
- Traditional REST API access
- Widely supported across all service types

**Enhanced Endpoints**:
- Differ in service URL, functionality, security certification level, and geographic region
- May offer improved performance or additional features

**Endpoint Structure**:
- RESTful architectural style
- Hierarchical information revelation
- Base pattern: `rest/services` indicates the REST services endpoint

**Documentation**: [Service Endpoints](https://developers.arcgis.com/rest/geocode/service-endpoints/)

### 4. Core Services Available

The REST API provides access to:

- **Basemaps** - Base map tiles and styles
- **Geocoding** - Address to coordinate conversion and reverse
- **Places** - Point of interest data
- **Routing** - Navigation and directions
- **GeoEnrichment** - Demographic and business data
- **Elevation** - Terrain and elevation data
- **Feature Service** - Vector data storage and editing
- **Geometry Service** - Spatial operations
- **Geoprocessing Service** - Custom analytical workflows
- **Stream Service** - Real-time data streaming

## Existing Rust Implementations

### Current State

As of 2025, **ESRI does not provide an official Rust SDK** for ArcGIS. According to an ESRI Product Manager (2021), there are no specific plans to deliver geospatial capabilities in Rust.

### Related Rust Crates

#### arcpbf / esripbf
- **Purpose**: Handles ArcGIS Protocol Buffer (PBF) format
- **Implementation**: Built with `prost` for protocol buffer handling
- **Functionality**: Reads FeatureCollection Protocol Buffer results from ArcGIS REST API
- **Primary Use**: R-ArcGIS project integration
- **Repository**: [R-ArcGIS/arcpbf](https://github.com/R-ArcGIS/arcpbf)
- **Crates.io**: [arcgis keyword search](https://crates.io/keywords/arcgis)

This is the only ArcGIS-specific Rust library currently available, and it's limited to protocol buffer parsing.

### GeoRust Ecosystem

While not ArcGIS-specific, the GeoRust collective maintains several relevant geospatial crates:

- **Website**: [georust.org](https://georust.org/)
- **GitHub**: [github.com/georust](https://github.com/georust)
- **Awesome List**: [awesome-georust](https://github.com/pka/awesome-georust)

These libraries provide general geospatial computing capabilities that could complement an ArcGIS SDK.

### Port Opportunity

ESRI has released an open-source geometry engine in Java, which could potentially be ported to Rust, though no such port currently exists.

## Implementation Strategy: Hand-Written Wrappers

### Reality Check

**Code generation is not viable** for the ArcGIS REST API:
- No official OpenAPI specifications exist (except Places API)
- Community specs are incomplete and unmaintained
- Creating and maintaining custom OpenAPI specs would be more work than hand-rolling code
- The API surface is large and documentation-driven

**Approach**: Hand-written Rust wrappers leveraging existing crates for foundational concerns.

### Why Rust: Type Safety as a Design Requirement

**Rust's type system is our primary tool for correctness.**

The purpose of this SDK is not just to wrap HTTP endpoints - it's to provide **compile-time guarantees** about data correctness. Rust enables us to encode API invariants in the type system so invalid states are unrepresentable.

#### Core Type Safety Principles

**1. No Stringly-Typed APIs**

❌ **Bad** (runtime validation):
```rust
struct MapService {
    service_type: String,  // Could be anything: "WebMap", "FeatureServer", "garbage"
}
```

✅ **Good** (compile-time validation):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    MapServer,
    FeatureServer,
    ImageServer,
    GeocodeServer,
    GeometryServer,
    GlobeServer,
    GPServer,
    StreamServer,
}
```

**2. Strong Temporal Types**

❌ **Bad**:
```rust
struct Token {
    expires_at: String,  // "2025-12-21T10:30:00Z" - could be malformed
}
```

✅ **Good**:
```rust
use chrono::{DateTime, Utc};

struct Token {
    expires_at: DateTime<Utc>,  // Type system enforces valid timestamps
}
```

**3. Newtype Pattern for Domain Values**

❌ **Bad**:
```rust
fn query_layer(layer_id: i32) -> Result<...> {
    // layer_id could be negative, could be wrong layer
}
```

✅ **Good**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerId(pub u32);

impl LayerId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

fn query_layer(layer_id: LayerId) -> Result<...> {
    // Type system ensures it's a layer ID, not some random number
}
```

**4. Typed API Parameters**

❌ **Bad**:
```rust
struct QueryParams {
    spatial_rel: Option<String>,  // "esriSpatialRelIntersects" - typo-prone
    format: Option<String>,       // "json", "geojson", "pbf" - error-prone
}
```

✅ **Good**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialRel {
    Intersects,
    Contains,
    Crosses,
    EnvelopeIntersects,
    IndexIntersects,
    Overlaps,
    Touches,
    Within,
}

impl SpatialRel {
    fn as_arcgis_str(&self) -> &'static str {
        match self {
            Self::Intersects => "esriSpatialRelIntersects",
            Self::Contains => "esriSpatialRelContains",
            // ...
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    Json,
    GeoJson,
    Pbf,
}

struct QueryParams {
    spatial_rel: Option<SpatialRel>,
    format: ResponseFormat,  // Not optional - always have a format
}
```

**5. Builder Pattern with Type State**

For complex operations, use type-state pattern to enforce required parameters at compile time:

```rust
pub struct FeatureQueryBuilder<State> {
    url: String,
    params: QueryParams,
    _state: PhantomData<State>,
}

pub struct Incomplete;
pub struct Complete;

impl FeatureQueryBuilder<Incomplete> {
    pub fn new(url: String) -> Self {
        Self {
            url,
            params: QueryParams::default(),
            _state: PhantomData,
        }
    }

    pub fn where_clause(mut self, clause: impl Into<String>) -> FeatureQueryBuilder<Complete> {
        self.params.where_clause = Some(clause.into());
        FeatureQueryBuilder {
            url: self.url,
            params: self.params,
            _state: PhantomData,
        }
    }
}

impl FeatureQueryBuilder<Complete> {
    pub async fn execute(self) -> Result<Features> {
        // Only Complete builders can execute
    }
}
```

**6. Validated Construction**

For values with constraints, use validated constructors:

```rust
#[derive(Debug, Clone)]
pub struct WhereClause(String);

impl WhereClause {
    pub fn new(clause: impl Into<String>) -> Result<Self, ValidationError> {
        let clause = clause.into();

        // Validate SQL injection risks, reserved words, etc.
        if clause.is_empty() {
            return Err(ValidationError::EmptyClause);
        }

        Ok(Self(clause))
    }

    pub fn always_true() -> Self {
        Self("1=1".to_string())
    }
}
```

#### Type-Driven Design Philosophy

**Every string constant in the API documentation should be an enum variant.**

When reading ArcGIS documentation, translate like this:

| ArcGIS Docs | Rust Type |
|-------------|-----------|
| `"geometryType": "esriGeometryPoint"` | `GeometryType::Point` |
| `"units": "esriSRUnit_Meter"` | `SpatialUnit::Meter` |
| `"capabilities": "Query,Create,Update"` | `HashSet<Capability>` |
| `"currentVersion": 10.91` | `Version` (newtype over `f64` or structured) |
| `"epoch": 1609459200000` | `DateTime<Utc>` |

**Every numeric ID should be a newtype:**

```rust
pub struct ServiceId(u64);
pub struct LayerId(u32);
pub struct FeatureId(i64);  // Can be negative in some contexts
pub struct ObjectId(u32);
```

**Every compound value should be a struct:**

```rust
// Not: (f64, f64)
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

// Or use geo-types
use geo_types::Point;
```

#### Benefits of This Approach

1. **Impossible to misuse the API** - Invalid requests don't compile
2. **IDE autocomplete** - Discover valid options via type system
3. **Refactoring safety** - Changing types breaks all invalid usage
4. **Self-documenting** - Types explain what values are valid
5. **No runtime validation overhead** - Checked at compile time
6. **Serialization still works** - `serde` handles enum ↔ string conversion

#### Implementation Rules

**REQUIRED (not optional):**

1. ✅ Use enums for all enumerated string values in the API
2. ✅ Use newtypes for all ID types (LayerId, ServiceId, etc.)
3. ✅ Use `chrono` types for all temporal values
4. ✅ Use `geo-types` for all spatial primitives
5. ✅ Use builder patterns with required fields enforced
6. ✅ Implement `Display` to convert types back to ArcGIS strings
7. ✅ Derive `serde::Serialize`/`Deserialize` with proper renames

**FORBIDDEN:**

1. ❌ Using `String` for enumerated values
2. ❌ Using `String` for temporal values
3. ❌ Using bare integers for IDs
4. ❌ Using tuples for compound values
5. ❌ Accepting invalid states "because the API might accept it"

### Manual Implementation Philosophy

**Read documentation → Encode in types → Make invalid states unrepresentable**

This SDK will be built by:
1. Reading ESRI's REST API documentation thoroughly
2. **Identifying all enumerated values and creating enums**
3. **Identifying all ID types and creating newtypes**
4. Translating JSON request/response structures to strongly-typed Rust structs
5. Implementing client methods that accept only valid types
6. Testing against real ArcGIS services
7. Iterating based on actual API behavior **and type safety opportunities**

## Leveraging Existing Rust Crates

**Don't reinvent the wheel.** The Rust ecosystem has excellent crates for all foundational concerns.

### Authentication: oauth2 Crate

The [oauth2 crate](https://github.com/ramosbugs/oauth2-rs) is an **extensible, strongly-typed** OAuth2 implementation (RFC 6749).

**Key Features**:
- Supports all OAuth 2.0 grant types (authorization code, client credentials, refresh token)
- Token introspection (RFC 7662) and revocation (RFC 7009)
- Async and sync support via `reqwest`
- Security: Built-in SSRF protection (disable redirects)

**Integration Plan**:
- Use `oauth2::Client` for all OAuth flows
- Implement custom token source for ArcGIS-specific endpoints
- Leverage automatic token refresh mechanisms
- Support all three ArcGIS auth types:
  - API Key: Custom implementation (simple header/query param)
  - OAuth 2.0: Direct `oauth2` crate usage
  - App Auth: `oauth2::ClientCredentials` flow

**Documentation**: [docs.rs/oauth2](https://docs.rs/oauth2/)

### Spatial Types: GeoRust Ecosystem

The GeoRust ecosystem provides **industry-standard spatial types** that should be used instead of custom implementations.

**Core Crates**:

1. **geo-types** ([docs](https://docs.rs/geo-types/))
   - Fundamental geometric types: `Point`, `LineString`, `Polygon`, `MultiPoint`, etc.
   - Zero-cost abstractions
   - Used by all GeoRust crates as the common type system

2. **geojson** ([docs](https://docs.rs/geojson/))
   - RFC 7946 compliant GeoJSON reading/writing
   - `TryInto` conversions to/from `geo-types`
   - Serde integration for serialization

3. **geo** ([docs](https://docs.rs/geo/))
   - Geospatial algorithms (area, distance, simplification, etc.)
   - Re-exports `geo-types`
   - IO support via `geojson` and `geozero`

4. **geozero** ([docs](https://docs.rs/geozero/))
   - Zero-copy reading/writing of multiple formats
   - Supports WKT, WKB, GeoJSON, MVT, GDAL
   - Excellent for ArcGIS geometry conversions

**Integration Strategy**:
```rust
use geo_types::{Point, Polygon, Geometry};
use geojson::GeoJson;

// ArcGIS API responses -> GeoJSON -> geo-types
let response: ArcGISFeatureResponse = ...;
let geojson: GeoJson = serde_json::from_str(&response.geometry)?;
let geometry: Geometry = geojson.try_into()?;

// geo-types -> GeoJSON -> ArcGIS API requests
let point = Point::new(-118.0, 34.0);
let geojson: GeoJson = point.into();
```

**Benefits**:
- Interoperability with entire GeoRust ecosystem
- Well-tested spatial algorithms
- Community-maintained, active development
- Avoid reinventing spatial types

### HTTP & Async: reqwest + tokio

Use the standard async Rust HTTP stack:

- **reqwest** ([docs](https://docs.rs/reqwest/)): Full-featured async HTTP client
  - JSON support via `serde`
  - Automatic redirect handling (disable for OAuth security)
  - Connection pooling
  - Middleware support

- **tokio** ([docs](https://docs.rs/tokio/)): Industry-standard async runtime
  - Required by `reqwest`
  - Provides timers for token refresh
  - Task spawning for background operations

### Additional Recommended Crates

- **url** ([docs](https://docs.rs/url/)): URL parsing and building
- **serde** / **serde_json**: JSON serialization (required)
- **thiserror**: Error type derivation
- **anyhow**: Flexible error handling for applications
- **chrono**: Date/time for token expiration
- **secrecy**: Secure storage of API keys/secrets
- **tracing**: Structured logging (better than `log`)
- **arcpbf**: Protocol buffer support (existing ArcGIS crate)

## Layered Architecture for Hand-Written Implementation

A **4-layer architecture** provides separation of concerns and maintainability:

### Layer 1: Foundation
**Fully hand-written, uses existing crates**

Components:
- **HTTP Client**: `reqwest::Client` with connection pooling
- **Authentication**: `oauth2` crate for all OAuth flows, custom API key handler
- **Error Types**: `thiserror`-based error hierarchy
- **Geometry Bridge**: Conversions between `geo-types` and ArcGIS JSON geometry format

Goals:
- Type-safe authentication token management
- Automatic token refresh
- Request/response error handling
- Geometry serialization/deserialization

### Layer 2: Low-Level Service API
**Hand-written strongly-typed request/response types**

Components:
- **Service Structs**: One per ArcGIS service type (FeatureService, MapService, etc.)
- **Domain Types**: Enums and newtypes for all API constants and IDs
- **Request Types**: Strongly-typed structs representing query parameters
- **Response Types**: Strongly-typed structs with `serde` derives for JSON responses
- **Builders**: Type-state builder pattern for complex queries

Example:
```rust
// Domain types (enums, not strings)
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GeometryType {
    #[serde(rename = "esriGeometryPoint")]
    Point,
    #[serde(rename = "esriGeometryPolyline")]
    Polyline,
    #[serde(rename = "esriGeometryPolygon")]
    Polygon,
    #[serde(rename = "esriGeometryMultipoint")]
    Multipoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerId(pub u32);

// Request types (strongly typed)
#[derive(Debug, serde::Serialize)]
pub struct FeatureQueryParams {
    #[serde(rename = "where")]
    pub where_clause: String,  // Or WhereClause newtype with validation

    #[serde(rename = "outFields")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_fields: Option<Vec<String>>,  // List, not comma-separated string

    #[serde(rename = "returnGeometry")]
    pub return_geometry: bool,  // Required, not optional - have a default

    #[serde(rename = "geometryType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry_type: Option<GeometryType>,  // Enum, not string

    #[serde(rename = "spatialRel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial_rel: Option<SpatialRel>,  // Enum, not string

    // ... dozens more fields, all strongly typed
}

// Response types (strongly typed)
#[derive(Debug, serde::Deserialize)]
pub struct FeatureQueryResponse {
    pub features: Vec<Feature>,

    #[serde(rename = "geometryType")]
    pub geometry_type: Option<GeometryType>,  // Enum, not string

    pub exceeded_transfer_limit: Option<bool>,

    #[serde(default)]
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, serde::Deserialize)]
pub struct FieldInfo {
    pub name: String,

    #[serde(rename = "type")]
    pub field_type: FieldType,  // Enum: Integer, Double, String, Date, etc.

    pub alias: Option<String>,
    pub length: Option<u32>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FieldType {
    #[serde(rename = "esriFieldTypeInteger")]
    Integer,
    #[serde(rename = "esriFieldTypeSmallInteger")]
    SmallInteger,
    #[serde(rename = "esriFieldTypeDouble")]
    Double,
    #[serde(rename = "esriFieldTypeString")]
    String,
    #[serde(rename = "esriFieldTypeDate")]
    Date,
    #[serde(rename = "esriFieldTypeOID")]
    ObjectId,
    #[serde(rename = "esriFieldTypeGeometry")]
    Geometry,
    #[serde(rename = "esriFieldTypeBlob")]
    Blob,
    #[serde(rename = "esriFieldTypeGlobalID")]
    GlobalId,
    #[serde(rename = "esriFieldTypeGUID")]
    Guid,
}
```

### Layer 3: High-Level Service Clients
**Hand-written ergonomic wrappers**

Components:
- **Typed Clients**: `FeatureServiceClient`, `MapServiceClient`, etc.
- **Async Iterators**: Stream-based pagination handling
- **Fluent Builders**: Chainable query construction
- **Type Safety**: Leverage Rust enums for string constants

Example:
```rust
let client = FeatureServiceClient::new(url, auth);

let features = client
    .query()
    .where_clause("POPULATION > 100000")
    .out_fields(&["NAME", "POPULATION"])
    .return_geometry(true)
    .spatial_filter(polygon)
    .execute()
    .await?;
```

### Layer 4: Convenience Layer
**Hand-written helpers and utilities**

Components:
- **Service Discovery**: Auto-detect service capabilities
- **Batch Operations**: Helper functions for bulk edits
- **Retry Logic**: Automatic retry with exponential backoff
- **Caching**: Optional response caching layer
- **Error Recovery**: Smart error handling and suggestions

### Implementation Order

**Prioritize by value:**

1. **Foundation layer** (required for everything)
2. **Feature Service low-level API** (most important service)
3. **Feature Service high-level client** (make it usable)
4. **Authentication flows** (all three types)
5. **Map Service** (second most important)
6. **Geocoding Service** (high utility)
7. **Geometry Service** (useful for spatial ops)
8. **Other services** as needed

### Code Organization

```
arcgis/
├── src/
│   ├── lib.rs
│   ├── types/             # Domain types (shared across services)
│   │   ├── mod.rs
│   │   ├── geometry.rs    # GeometryType, SpatialRel enums
│   │   ├── spatial.rs     # SpatialReference, Unit enums
│   │   ├── field.rs       # FieldType enum, field-related types
│   │   ├── ids.rs         # LayerId, ServiceId, ObjectId newtypes
│   │   └── time.rs        # Temporal types, epoch conversions
│   ├── auth/              # Layer 1: Authentication
│   │   ├── mod.rs
│   │   ├── api_key.rs
│   │   ├── oauth.rs
│   │   └── token.rs
│   ├── client.rs          # Layer 1: Core HTTP client
│   ├── error.rs           # Layer 1: Error types
│   ├── geometry/          # Layer 1: Geometry conversions
│   │   ├── mod.rs
│   │   ├── convert.rs     # geo-types <-> ArcGIS JSON
│   │   └── serde.rs       # Custom serde implementations
│   ├── services/          # Layers 2 & 3
│   │   ├── mod.rs
│   │   ├── feature/       # Feature Service
│   │   │   ├── mod.rs
│   │   │   ├── types.rs   # Layer 2: Strongly-typed Request/Response
│   │   │   ├── enums.rs   # Service-specific enums
│   │   │   ├── client.rs  # Layer 3: High-level client
│   │   │   └── query.rs   # Layer 3: Type-state query builder
│   │   ├── map/           # Map Service
│   │   │   ├── types.rs
│   │   │   ├── enums.rs   # ImageFormat, LayerType, etc.
│   │   │   └── client.rs
│   │   ├── geocode/       # Geocoding Service
│   │   │   ├── types.rs
│   │   │   ├── enums.rs   # LocatorType, AddressComponent, etc.
│   │   │   └── client.rs
│   │   └── ...
│   └── util/              # Layer 4: Utilities
│       ├── retry.rs
│       ├── pagination.rs
│       └── cache.rs
└── tests/
    ├── unit/
    │   └── serde_enums.rs # Test all enum <-> string conversions
    └── integration/       # Tests against live API
```

### Development Workflow

For each service:
1. **Read ESRI docs** for the service thoroughly
2. **Extract all enumerated values** → Create enums
3. **Identify all ID types** → Create newtypes
4. **Create types.rs** with strongly-typed request/response structs
5. **Write unit tests** for serialization/deserialization (verify enum mappings)
6. **Implement client.rs** with type-safe API calls
7. **Test against live API** to validate behavior and discover missing enum variants
8. **Add high-level builders** with type-state pattern for ergonomics
9. **Document** with examples showing type safety benefits
10. **Iterate** based on real-world usage and discovered edge cases

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
**Goal**: Core infrastructure working

Tasks:
- Set up Cargo workspace with proper dependencies
- Implement core HTTP client with `reqwest` + `tokio`
- Create error type hierarchy with `thiserror`
- Implement API Key authentication (simplest)
- Write geometry conversion layer (`geo-types` ↔ ArcGIS JSON)
- Basic logging with `tracing`

Deliverable: Can make authenticated requests and handle responses

### Phase 2: Feature Service - Query (Week 3-4)
**Goal**: Can query features from a service

Tasks:
- Define `FeatureQueryParams` struct with all query parameters
- Define `FeatureQueryResponse` struct
- Implement `Feature`, `Field`, `Geometry` response types
- Create `FeatureServiceClient::query()` method
- Handle pagination for large result sets
- Support GeoJSON and JSON response formats

Deliverable: Can query and retrieve features with geometry

### Phase 3: Feature Service - Editing (Week 5-6)
**Goal**: Can create, update, delete features

Tasks:
- Implement `AddFeatures` operation
- Implement `UpdateFeatures` operation
- Implement `DeleteFeatures` operation
- Implement `ApplyEdits` (batch operations)
- Handle transaction responses and errors
- Support attachment operations

Deliverable: Full CRUD operations on feature services

### Phase 4: OAuth Authentication (Week 7)
**Goal**: Support OAuth 2.0 flows

Tasks:
- Integrate `oauth2` crate
- Implement authorization code flow
- Implement client credentials flow
- Implement token refresh logic
- Add token storage/retrieval abstraction
- Test with ArcGIS Online organization

Deliverable: Can authenticate users via OAuth 2.0

### Phase 5: Map Service (Week 8-9)
**Goal**: Can retrieve map tiles and images

Tasks:
- Implement Map Service metadata retrieval
- Implement tile export for cached services
- Implement dynamic map export
- Handle image format options (PNG, JPEG)
- Support spatial reference transformations
- Implement legend and layer info retrieval

Deliverable: Can retrieve map images and tiles

### Phase 6: Geocoding Service (Week 10)
**Goal**: Address geocoding and reverse geocoding

Tasks:
- Implement `findAddressCandidates` (forward geocoding)
- Implement `reverseGeocode` operation
- Implement `suggest` (autocomplete)
- Implement batch geocoding
- Support various locator types
- Handle geocoding response formats

Deliverable: Full geocoding capabilities

### Phase 7: Polish & Documentation (Week 11-12)
**Goal**: Production-ready release

Tasks:
- Comprehensive API documentation
- Usage examples for all services
- Integration tests against public ArcGIS services
- Performance benchmarks
- README with quickstart guide
- CHANGELOG and semantic versioning setup
- CI/CD with GitHub Actions

Deliverable: v0.1.0 release ready for crates.io

## Technical Considerations

### Required Crates (Core Dependencies)
- `reqwest` - Async HTTP client
- `tokio` - Async runtime
- `serde` / `serde_json` - JSON serialization/deserialization
- `oauth2` - OAuth 2.0 authentication flows
- `geo-types` - Spatial geometry types (GeoRust standard)
- `geojson` - GeoJSON format support
- `url` - URL parsing and building
- `thiserror` - Error type derivation
- `tracing` - Structured logging

### Optional/Recommended Crates
- `geo` - Geospatial algorithms (area, distance, etc.)
- `geozero` - Zero-copy multi-format geometry parsing
- `arcpbf` - ArcGIS Protocol Buffer support (for PBF format responses)
- `chrono` - Date/time handling for token expiration
- `secrecy` - Secure credential storage
- `anyhow` - Application-level error handling in examples

### API Response Format
- Primary format: JSON
- Alternative format: Protocol Buffers (PBF) for feature services
- Image formats: PNG, JPEG for map services

### Error Handling
- HTTP errors (4xx, 5xx)
- API-specific errors (error codes and messages)
- Rate limiting
- Token expiration
- Network timeouts

### Authentication Token Management
- Secure storage of credentials
- Automatic token refresh before expiration
- Token expiration typically ranges from minutes to hours
- Support for both short-lived and long-lived tokens

## API Patterns to Implement

### RESTful Structure
```
https://services.arcgis.com/
  └─ {organizationId}/arcgis/rest/services/
      └─ {serviceName}/{serviceType}/
          └─ {layerId}/
              └─ {operation}
```

### Common Query Parameters
- `f` - Response format (json, pjson, geojson, pbf)
- `token` - Authentication token
- `where` - SQL where clause for filtering
- `outFields` - Fields to return
- `returnGeometry` - Include geometry in response
- `spatialRel` - Spatial relationship for queries
- `geometry` - Geometry for spatial queries
- `geometryType` - Type of geometry provided

### Pagination
- `resultOffset` - Starting record position
- `resultRecordCount` - Number of records to return
- `exceededTransferLimit` - Indicator that more records exist

## References

### Official Documentation
- [Build powerful apps with ArcGIS services](https://developers.arcgis.com/rest/)
- [ArcGIS Server Services Directory REST API](https://developers.arcgis.com/rest/services-reference/)
- [Feature Service Documentation](https://developers.arcgis.com/rest/services-reference/feature-service.htm)
- [Map Service Documentation](https://developers.arcgis.com/rest/services-reference/enterprise/map-service/)
- [Authentication Documentation](https://developers.arcgis.com/rest/users-groups-and-items/authentication/)
- [Security and Authentication](https://developers.arcgis.com/documentation/security-and-authentication/)
- [Get Started with Services Directory](https://developers.arcgis.com/rest/services-reference/get-started-with-the-services-directory.htm)

### Community Resources
- [GeoRust Organization](https://github.com/georust)
- [Awesome GeoRust](https://github.com/pka/awesome-georust)
- [Geospatial Libraries on Lib.rs](https://lib.rs/science/geo)
- [R-ArcGIS/arcpbf](https://github.com/R-ArcGIS/arcpbf)

### Related Tools
- [arcgis-rest-js](https://github.com/Esri/arcgis-rest-js) - Official JavaScript wrapper (reference implementation)
- [Geospatial Programming with Rust](https://pka.github.io/rust-for-geo/)

### Rust Crate Resources
- [oauth2-rs](https://github.com/ramosbugs/oauth2-rs) - OAuth 2.0 client library
- [oauth2 crate docs](https://docs.rs/oauth2/)
- [geo-types crate](https://docs.rs/geo-types/) - Spatial geometry types
- [geojson crate](https://docs.rs/geojson/) - GeoJSON support
- [geo crate](https://docs.rs/geo/) - Geospatial algorithms
- [geozero crate](https://docs.rs/geozero/) - Multi-format geometry I/O
- [reqwest crate](https://docs.rs/reqwest/) - HTTP client
- [tokio crate](https://docs.rs/tokio/) - Async runtime
- [thiserror crate](https://docs.rs/thiserror/) - Error derive macros

---

**Document Version**: 4.0 - Type-Safe Hand-Written Implementation Strategy
**Last Updated**: December 21, 2025
**Research Date**: December 21, 2025
