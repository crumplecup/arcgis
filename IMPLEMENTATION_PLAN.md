# ArcGIS Rust SDK - Strategic Implementation Plan

## Current Status (Updated: 2026-01-04)

**Branch**: `dev`
**Latest Version**: v0.3.0-ready (Map Service complete)

**✅ Completed Phases**:
- ✅ **Phase 1**: OAuth 2.0 Client Credentials authentication (fully automated)
- ✅ **Phase 2**: Feature Service query API with auto-pagination
  - ✅ Custom URL query parameter serialization (Vec<T> → comma-separated, geometry → JSON)
  - ✅ Count-only query support with proper response handling
- ✅ **Phase 3**: Feature Service CRUD operations (add, update, delete, batch)
- ✅ **Phase 3**: Attachment operations (query, add, update, delete, download with streaming)
- ✅ **Phase 3**: Edit Sessions for branch-versioned geodatabases (startEditing/stopEditing)
- ✅ **Phase 3**: Version Management Service (complete operation suite)
  - ✅ Read sessions (startReading/stopReading)
  - ✅ Version lifecycle (create, alter, delete, get_info, list_versions)
  - ✅ Reconcile & Post workflow (reconcile, post, partial post)
  - ✅ Conflict management (conflicts, inspect_conflicts, delete_forward_edits)
  - ✅ Analysis operations (differences, restore_rows)
- ✅ **Phase 4.1**: Map Service (export map, tiles, legends, metadata, identify)
  - ✅ Export map with dynamic rendering (25+ parameters)
  - ✅ Export cached tiles
  - ✅ Legend retrieval
  - ✅ Service metadata
  - ✅ Feature identification
  - ✅ Binary streaming to Path/Bytes/Writer
  - ✅ Fluent builder API with 20+ methods
- ✅ **Phase 4.2**: Geocoding Service (findAddressCandidates, reverseGeocode, suggest)

**🚧 In Progress**:
- Phase 4.3: Advanced Queries (statistics, related records)

**Recent Commits**:
- `780ee4d` - fix(feature_service): add custom serializers for URL query parameters
- `818e932` - feat(feature_service): implement attachment operations with streaming support
- `1ad49fc` - feat(version_management): implement differences and restore_rows operations
- `7969da2` - feat(version_management): implement conflict management operations
- `fd4985f` - feat(version_management): implement reconcile and post operations

---

## Vision Statement

Build a type-safe, ergonomic Rust SDK for the ArcGIS REST API that makes invalid states unrepresentable through compile-time guarantees, leveraging the Rust ecosystem (GeoRust, oauth2, reqwest/tokio) to provide developers with a superior experience compared to stringly-typed alternatives.

## Coverage Levels Defined

### Minimal Coverage (v0.1.0)
**Goal**: Enable the most common workflow - querying features with API key auth

**Scope**:
- API Key authentication only
- Feature Service query operations only (read-only)
- Basic geometry support via GeoRust integration
- Single-threaded pagination
- Essential error handling

**User Story**: "As a developer, I can authenticate with an API key and query features from a feature service, receiving strongly-typed results with geometry."

**Success Criteria**:
- ✅ Can query a public feature service
- ✅ Can filter with WHERE clauses
- ✅ Can retrieve geometry as `geo-types`
- ✅ Can paginate through large result sets
- ✅ Published to crates.io

### Basic Coverage (v0.2.0)
**Goal**: Full CRUD on features + OAuth authentication

**Scope**:
- OAuth 2.0 authentication (all flows)
- Feature Service editing operations (create, update, delete)
- Batch operations (Apply Edits)
- Attachment support
- Transaction handling

**User Story**: "As a developer, I can authenticate via OAuth and perform full CRUD operations on feature services, including managing attachments."

**Success Criteria**:
- ✅ OAuth 2.0 authorization code flow working
- ✅ Can create, update, delete features
- ✅ Can batch edit features atomically
- ✅ Can upload/download attachments
- ✅ Proper error handling for edit failures

### Intermediate Coverage (v0.3.0)
**Goal**: Multi-service support for core workflows

**Scope**:
- Map Service (tile/image export)
- Geocoding Service (forward, reverse, batch)
- Service metadata and capabilities discovery
- Advanced query capabilities (spatial queries, related records)
- Streaming pagination

**User Story**: "As a developer, I can geocode addresses, perform spatial queries, and export map images, all with the same type-safe API."

**Success Criteria**:
- ✅ Can geocode addresses and reverse geocode coordinates
- ✅ Can export map tiles and images
- ✅ Can perform spatial relationship queries
- ✅ Can query related records
- ✅ Async stream-based pagination

### Advanced Coverage (v0.4.0)
**Goal**: Advanced services and enterprise features

**Scope**:
- Geometry Service (spatial operations)
- Routing Service (directions, service areas)
- GeoEnrichment Service
- Geoprocessing Service integration
- Advanced spatial reference handling
- Protocol Buffer (PBF) format support

**User Story**: "As a developer, I can perform complex spatial operations, calculate routes, and run geoprocessing tasks programmatically."

**Success Criteria**:
- ✅ Can project geometries between spatial references
- ✅ Can calculate routes and service areas
- ✅ Can execute geoprocessing tasks
- ✅ PBF format support for high-performance queries
- ✅ Comprehensive spatial operation support

### Maximal Coverage (v1.0.0)
**Goal**: Production-ready SDK with full API surface coverage

**Scope**:
- Stream Service (real-time data)
- Places Service
- Utility Network Service
- Version Management Service
- Enterprise administration APIs
- Comprehensive caching layer
- Connection pooling optimization
- Retry/circuit breaker patterns
- Telemetry and instrumentation

**User Story**: "As a developer, I have a production-grade SDK that covers all ArcGIS REST API services with excellent performance, reliability, and observability."

**Success Criteria**:
- ✅ All documented services implemented
- ✅ Production-grade error handling and retry logic
- ✅ Performance benchmarks published
- ✅ Comprehensive documentation and examples
- ✅ CI/CD with automated integration tests
- ✅ Semantic versioning and stability guarantees

## Phase-by-Phase Roadmap

**CRITICAL**: Automated authentication is Phase 1, Priority 0. Without it, no
development or testing is possible.

---

## Phase 1: Automated Authentication (v0.1.0-alpha) - Week 1

### Milestone 1.1: OAuth Client Credentials Implementation (Week 1)

**Priority**: P0 - BLOCKING for all other work

**Context**: We need **automated** authentication for servers and scripts.
OAuth Authorization Code + PKCE requires browser interaction and is NOT
suitable for our use case.

**Deliverables**:
- ✅ API Key authentication (already implemented)
- ✅ OAuth 2.0 Client Credentials Flow (automated, no human interaction)
- ✅ Token refresh logic with expiration checking
- ✅ Secure HTTP client (SSRF prevention)
- ✅ Integration with existing AuthProvider trait

**Technical Tasks**:
```rust
// src/auth/client_credentials.rs
use oauth2::{
    basic::BasicClient,
    ClientId, ClientSecret, TokenUrl,
};

pub struct ClientCredentialsAuth {
    client: BasicClient,
    http_client: reqwest::Client,
    token: Arc<RwLock<Option<StandardTokenResponse>>>,
}

impl ClientCredentialsAuth {
    pub fn new(client_id: String, client_secret: String) -> Result<Self> {
        let token_url = TokenUrl::new(
            "https://www.arcgis.com/sharing/rest/oauth2/token".to_string()
        )?;

        let client = BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_token_uri(token_url);

        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self { client, http_client, token: Arc::new(RwLock::new(None)) })
    }

    async fn fetch_token(&self) -> Result<()> {
        let token = self.client
            .exchange_client_credentials()
            .request_async(&self.http_client)
            .await?;

        *self.token.write().await = Some(token);
        Ok(())
    }
}

#[async_trait]
impl AuthProvider for ClientCredentialsAuth {
    async fn get_token(&self) -> Result<String> {
        // Fetch token on first use or refresh if expired
        // Returns access_token string
    }
}
```

**Success Criteria**:
- ✅ Can authenticate without human interaction
- ✅ Token fetch is automatic on first use
- ✅ Token refresh works automatically before expiration (5-minute buffer)
- ✅ HTTP client prevents SSRF (redirects disabled)
- ✅ Implements AuthProvider trait
- ✅ Works in CI/CD and server environments

### Milestone 1.2: Example + Testing (Week 1)

**Deliverables**:
- ✅ Working CLI example demonstrating client credentials
- ✅ Integration test (manual, with real ArcGIS credentials)
- ✅ Documentation explaining both auth methods (API Key vs Client Credentials)
- ✅ Error handling guide
- ✅ Token refresh verification

**Technical Tasks**:
```rust
// examples/client_credentials_flow.rs
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // Fully automated - no browser required!
    let auth = ClientCredentialsAuth::new(
        env::var("CLIENT_ID")?,
        env::var("CLIENT_SECRET")?,
    )?;

    println!("Fetching access token...");
    let token = auth.get_token().await?;
    println!("Token: {}...", &token[..20]);

    // Use authenticated client
    let client = ArcGISClient::new(auth);
    // ... make API calls - token refresh is automatic ...
}
```

**Success Criteria**:
- ✅ CLI example works end-to-end with real credentials
- ✅ No browser interaction required
- ✅ Token refresh tested and working
- ✅ Documentation explains when to use each auth method
- ✅ Error messages are helpful
- ✅ Can make authenticated API requests

**v0.1.0-alpha Release Checklist**:
- [ ] API Key authentication working (already done)
- [ ] Client Credentials authentication working
- [ ] CLI example functional
- [ ] Documentation complete
- [ ] Integration test passing
- [ ] CHANGELOG.md updated
- [ ] Version bumped
- [ ] Tagged in git

---

## Phase 2: Core Infrastructure (v0.2.0) - Weeks 3-5

**Note**: Now that OAuth is working, we can build and test the SDK core.

### Milestone 2.1: Foundation & Geometry (Week 3)

**Deliverables**:
- ✅ Core HTTP client wrapper around `reqwest`
- ✅ Error type hierarchy with `derive_more`
- ✅ Logging infrastructure with `tracing`
- ✅ `geo-types` ↔ ArcGIS JSON geometry conversion
- ✅ Core geometry type enums (GeometryType, SpatialRel)
- ✅ Spatial reference handling (basic)

**Technical Tasks**:
```rust
// src/client.rs
pub struct ArcGISClient {
    http: reqwest::Client,
    auth: Arc<dyn AuthProvider>,
}

// src/types/geometry.rs
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum GeometryType {
    #[serde(rename = "esriGeometryPoint")]
    Point,
    #[serde(rename = "esriGeometryPolyline")]
    Polyline,
    #[serde(rename = "esriGeometryPolygon")]
    Polygon,
}
```

**Success Criteria**:
- ✅ Can make authenticated HTTP requests with OAuth
- ✅ Errors properly typed and structured
- ✅ Geometry round-trip conversion works
- ✅ All instrumentation in place

### Milestone 2.2: Feature Query API (Week 4)

**Deliverables**:
- ✅ Feature Service metadata types
- ✅ FeatureQueryParams with all query parameters
- ✅ Custom serde serializers for URL query parameters (Vec<T>, geometry)
- ✅ Basic query execution
- ✅ WHERE clause support
- ✅ Query builder pattern
- ✅ Pagination support (auto-pagination with execute_all)
- ✅ Count-only query support

**Technical Tasks**:
```rust
// src/services/feature/query.rs
pub struct QueryBuilder {
    client: FeatureServiceClient,
    layer_id: LayerId,
    params: FeatureQueryParams,
}

impl QueryBuilder {
    pub fn where_clause(mut self, clause: impl Into<String>) -> Self { ... }
    pub fn out_fields(mut self, fields: &[&str]) -> Self { ... }
    pub async fn execute(self) -> Result<FeatureSet> { ... }
    pub async fn execute_all(self) -> Result<FeatureSet> { /* auto-paginate */ }
}
```

**Success Criteria**:
- ✅ Can query features with OAuth authentication
- ✅ WHERE clauses work correctly
- ✅ Pagination automatic and transparent (execute_all)
- ✅ Can retrieve large datasets
- ✅ Count-only queries return proper count field
- ✅ All query parameters properly serialized to ArcGIS REST API format

### Milestone 2.3: Documentation & Testing (Week 5)

**Deliverables**:
- ✅ Comprehensive API documentation
- ✅ Query examples with OAuth
- ✅ Integration tests against live services
- ✅ README and quickstart guide
- ⏸️ Performance benchmarks (deferred)

**Success Criteria**:
- ✅ All public APIs documented
- ✅ Examples work end-to-end
- ✅ Integration tests passing
- ✅ Ready for v0.2.0 release

**v0.2.0 Release Checklist**:
- [ ] OAuth + core SDK working
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Examples functional
- [ ] Published to crates.io

---

## Phase 3: CRUD Operations (v0.3.0) - Weeks 6-8

### Milestone 3.1: Feature Editing (Week 6)

**Deliverables**:
- ✅ AddFeatures operation
- ✅ UpdateFeatures operation
- ✅ DeleteFeatures operation
- ✅ Edit result handling
- ✅ Transaction rollback on error

**Technical Tasks**:
```rust
// src/services/feature/edit.rs
pub struct AddFeaturesRequest {
    pub features: Vec<Feature>,
    pub gdb_version: Option<String>,
    pub rollback_on_failure: bool,
}

impl FeatureServiceClient {
    pub async fn add_features(&self, layer_id: LayerId, request: AddFeaturesRequest)
        -> Result<EditResult> { ... }

    pub async fn update_features(&self, layer_id: LayerId, features: Vec<Feature>)
        -> Result<EditResult> { ... }

    pub async fn delete_features(&self, layer_id: LayerId, object_ids: &[ObjectId])
        -> Result<EditResult> { ... }
}
```

**Success Criteria**:
- ✅ Can create new features
- ✅ Can update existing features
- ✅ Can delete features
- ✅ Edit failures properly reported

### Milestone 3.2: Batch Operations & Edit Sessions (Week 7)

**Deliverables**:
- ✅ ApplyEdits operation (add + update + delete in one transaction)
- ✅ Batch result handling
- ✅ Partial success handling
- ✅ Edit session support (startEditing/stopEditing for branch versioning)
- ✅ Session ID integration with edit operations
- ✅ Attachment support (query, add, update, delete, download with streaming)

**Technical Tasks**:
```rust
pub struct ApplyEditsRequest {
    pub adds: Vec<Feature>,
    pub updates: Vec<Feature>,
    pub deletes: Vec<ObjectId>,
    pub gdb_version: Option<String>,
    pub rollback_on_failure: bool,
}

pub struct ApplyEditsResult {
    pub add_results: Vec<EditResultItem>,
    pub update_results: Vec<EditResultItem>,
    pub delete_results: Vec<EditResultItem>,
}
```

**Success Criteria**:
- ✅ Can perform complex edits atomically
- ✅ Partial failures properly reported
- ✅ Rollback works as expected
- ✅ Can query, upload, update, delete, and download attachments
- ✅ Attachment streaming support for large files

### Milestone 3.3: Documentation & Testing (Week 8)

**Deliverables**:
- ✅ CRUD operation examples (in doctests)
- ✅ Integration tests for editing
- ✅ Error handling documentation
- ⏸️ Performance benchmarks for batch operations (deferred)

**Success Criteria**:
- ✅ All CRUD operations documented
- ✅ Examples demonstrate best practices
- ✅ Tests cover error scenarios

**v0.3.0 Release Checklist**:
- [ ] All CRUD operations tested
- [ ] Documentation complete
- [ ] Migration guide from v0.2.0
- [ ] Published to crates.io

---

## Phase 3.5: Version Management Service - CRITICAL Enterprise Feature

**IMPORTANT**: Version Management is NOT optional - it's "bread and butter basic GIS management" for enterprise workflows. Both Branch and Traditional versioning use the same Version Management Service API (as of ArcGIS Server 11.1+).

### Understanding Traditional vs Branch Versioning

**Key Architectural Differences**:

| Aspect | Traditional Versioning | Branch Versioning |
|--------|----------------------|-------------------|
| **Storage Model** | Delta tables (A/D tables) | Temporal model (same base table) |
| **Compression** | Required weekly/daily | Not needed |
| **Access Model** | Direct database connection | Service-oriented (web layers) |
| **Version Hierarchy** | Multi-level (grandchildren, etc.) | Single-level (off DEFAULT only) |
| **Concurrent Editing** | Multiple editors per version | Exclusive lock (one editor per version) |
| **Edit Sessions** | Optional | Required for editing |
| **REST API** | Deprecated (11.1+) | Version Management Service |
| **Maintenance** | Admin burden (compress) | Minimal |
| **Use Cases** | Legacy enterprise workflows | Modern web-based editing |

**CRITICAL INSIGHT**: We **don't** need separate `BranchVersionClient` vs `TraditionalVersionClient`. The Version Management Service handles both types - the server enforces versioning-type constraints. Our abstraction provides all operations, and the server returns appropriate errors for unsupported operations.

### Milestone 3.6: Version Management Service - Core Operations (Week 8)

**Status**: ✅ COMPLETE - All core operations implemented

**Deliverables**:
- ✅ Read sessions (startReading/stopReading)
- ✅ Version lifecycle (create, alter, delete, get_info, list_versions)
- ✅ Reconcile & Post workflow (reconcile, post, partial post)
- ✅ Conflict management (conflicts, inspect_conflicts, delete_forward_edits)
- ✅ Analysis operations (differences, restore_rows)
- ✅ Complete documentation (comprehensive docstrings and examples)
- ⏸️ Comprehensive integration tests (deferred - requires live test environment)

**Technical Tasks**:

```rust
// src/services/version_management/client.rs

impl VersionManagementClient {
    // ✅ COMPLETE: Edit Session Management
    pub async fn start_editing(&self, version_guid, session_id) -> Result<StartEditingResponse>
    pub async fn stop_editing(&self, version_guid, session_id, save_edits) -> Result<StopEditingResponse>

    // 🚧 NEW: Read Session Management
    pub async fn start_reading(&self, version_guid, session_id) -> Result<StartReadingResponse>
    pub async fn stop_reading(&self, version_guid, session_id) -> Result<StopReadingResponse>

    // 🚧 NEW: Version Lifecycle
    pub async fn create(&self, params: CreateVersionParams) -> Result<VersionInfo>
    pub async fn alter(&self, version_guid, properties: AlterVersionParams) -> Result<AlterResponse>
    pub async fn delete(&self, version_guid) -> Result<DeleteResponse>
    pub async fn get_info(&self, version_guid) -> Result<VersionInfo>
    pub async fn list_versions(&self, params: ListVersionsParams) -> Result<Vec<VersionInfo>>

    // 🚧 NEW: Reconcile & Post Workflow (CRITICAL)
    pub async fn reconcile(&self, version_guid, options: ReconcileOptions) -> Result<ReconcileResponse>
    pub async fn post(&self, version_guid, session_id) -> Result<PostResponse>
    pub async fn partial_post(&self, version_guid, session_id, params: PartialPostParams) -> Result<PostResponse>

    // 🚧 NEW: Conflict Management
    pub async fn conflicts(&self, version_guid) -> Result<ConflictsResponse>
    pub async fn inspect_conflicts(&self, version_guid, params: InspectConflictsParams) -> Result<InspectConflictsResponse>
    pub async fn delete_forward_edits(&self, version_guid, session_id, params: DeleteEditsParams) -> Result<DeleteEditsResponse>

    // 🚧 NEW: Analysis Operations
    pub async fn differences(&self, version_guid, options: DifferencesOptions) -> Result<DifferencesResponse>
    pub async fn restore_rows(&self, version_guid, session_id, params: RestoreRowsParams) -> Result<RestoreResponse>
}
```

**New Types Required**:

```rust
// Version Lifecycle Types
pub struct CreateVersionParams {
    pub version_name: String,
    pub permission: VersionPermission,  // Public, Protected, Private
    pub description: Option<String>,
}

pub enum VersionPermission {
    Public,
    Protected,
    Private,
}

pub struct AlterVersionParams {
    pub version_name: Option<String>,
    pub description: Option<String>,
    pub access: Option<VersionPermission>,
}

pub struct ListVersionsParams {
    pub include_hidden: bool,
    pub owner_filter: Option<String>,
}

// Reconcile & Post Types
pub struct ReconcileOptions {
    pub end_with_conflict: bool,  // Abort if conflicts detected
    pub conflict_detection: ConflictDetection,  // ByObject or ByAttribute
    pub with_post: bool,  // Automatically post after successful reconcile
}

pub enum ConflictDetection {
    ByObject,
    ByAttribute,
}

pub struct ReconcileResponse {
    pub success: bool,
    pub has_conflicts: bool,
    pub moment: Option<String>,  // Timestamp
    pub error: Option<EditSessionError>,
}

pub struct PartialPostParams {
    pub layers: Vec<LayerRowsToPost>,
}

pub struct LayerRowsToPost {
    pub layer_id: LayerId,
    pub object_ids: Vec<ObjectId>,
}

// Conflict Types
pub struct ConflictsResponse {
    pub layers: Vec<LayerConflicts>,
}

pub struct LayerConflicts {
    pub layer_id: LayerId,
    pub conflicts: Vec<Conflict>,
}

pub struct Conflict {
    pub object_id: ObjectId,
    pub conflict_type: ConflictType,  // Update-Update, Update-Delete, Delete-Update
}

pub enum ConflictType {
    UpdateUpdate,
    UpdateDelete,
    DeleteUpdate,
}

// Differences Types
pub struct DifferencesOptions {
    pub result_type: DifferenceResultType,  // Features or ObjectIds
    pub layers: Vec<LayerId>,
    pub future: bool,  // Include future edits
}

pub enum DifferenceResultType {
    Features,
    ObjectIds,
}

pub struct DifferencesResponse {
    pub layers: Vec<LayerDifferences>,
}

pub struct LayerDifferences {
    pub layer_id: LayerId,
    pub inserts: Vec<Feature>,  // or Vec<ObjectId> depending on result_type
    pub updates: Vec<Feature>,
    pub deletes: Vec<ObjectId>,
}
```

**Session Requirements Matrix**:

| Operation | Read Session | Edit Session | Notes |
|-----------|--------------|--------------|-------|
| get_info | No | No | Metadata only |
| create | No | No | Creates new version |
| alter | No | No | Modifies properties |
| delete | No | No | Deletes version |
| start_reading | No | No | Initiates read lock |
| start_editing | No | No | Initiates write lock |
| stop_reading | Yes | No | Releases read lock |
| stop_editing | No | Yes | Releases write lock, save/discard |
| reconcile | No | Yes | Merges changes from parent |
| post | No | Yes | Pushes changes to parent |
| partial_post | No | Yes | Posts subset of changes |
| conflicts | No | No | Query only |
| inspect_conflicts | No | No | Detailed analysis |
| delete_forward_edits | No | Yes | Conflict resolution |
| differences | No | No | Compare with parent |
| restore_rows | No | Yes | Undo edits |

**Success Criteria**:
- ✅ Complete workflow: create version → edit → reconcile → post (implemented)
- ✅ Conflict detection and resolution working (implemented)
- ✅ Read sessions for concurrent readers (implemented)
- ✅ Differences API for PR-style review (implemented)
- ✅ All operations documented with examples (comprehensive docstrings)
- ⏸️ Integration tests against real Version Management Service (deferred)
- ✅ Error handling for versioning-type constraints (server-enforced via HTTP status)

**References**:
- [Version Management Service Documentation](https://developers.arcgis.com/rest/services-reference/enterprise/version-management-service/)
- [Version Resource Operations](https://developers.arcgis.com/rest/services-reference/enterprise/version/)
- [Start Editing Operation](https://developers.arcgis.com/rest/services-reference/enterprise/start-editing/)
- [Reconcile Version (Deprecated)](https://developers.arcgis.com/rest/services-reference/enterprise/reconcile-version/)
- [Working with Branch Versioning (Python API)](https://developers.arcgis.com/python/latest/guide/working-with-branch-versioning/)

---

## Phase 4: Multi-Service Support (v0.4.0) - Weeks 9-12

### Milestone 4.1: Map Service (Weeks 9-10) - ✅ COMPLETE

**Deliverables**:
- ✅ Map Service metadata (`get_metadata()`)
- ✅ Export Map (dynamic rendering with 25+ parameters)
- ✅ Export Tile (cached tiles via `TileCoordinate`)
- ✅ Legend support (`get_legend()`)
- ✅ Feature identification (`identify()`)
- ✅ Image format enums (PNG, PNG8/24/32, JPEG, GIF, PDF, SVG, etc.)
- ✅ Binary streaming to Path/Bytes/Writer
- ✅ Fluent builder API (`ExportMapBuilder`)

**Implementation Details**:
```rust
// src/services/map/enums.rs (140 lines)
pub enum ImageFormat {
    Png, Png8, Png24, Png32, Jpg, Pdf, Bmp, Gif, Svg, Svgz, Emf, Ps
}
pub enum LayerOperation { Show, Hide, Include, Exclude }
pub enum ResponseFormat { Html, Json, PJson, Image, Kmz }
pub enum LayerSelection { Top, Visible, All }
pub enum TimeRelation { Overlaps, After, Before }

// src/services/map/types.rs (640 lines)
pub struct ExportMapParams {
    pub bbox: String,  // Required
    pub size: Option<String>,
    pub dpi: Option<i32>,
    pub format: Option<ImageFormat>,
    pub transparent: Option<bool>,
    // ... 20+ more parameters
}
pub enum ExportTarget { Path(PathBuf), Bytes, Writer(Box<dyn AsyncWrite>) }
pub enum ExportResult { Path(PathBuf), Bytes(Vec<u8>), Written(u64) }

// src/services/map/client.rs (657 lines)
impl MapServiceClient {
    pub async fn export_map(&self, params: ExportMapParams, target: ExportTarget)
        -> Result<ExportResult>
    pub async fn export_tile(&self, coord: TileCoordinate, target: ExportTarget)
        -> Result<ExportResult>
    pub async fn get_legend(&self) -> Result<LegendResponse>
    pub async fn get_metadata(&self) -> Result<MapServiceMetadata>
    pub async fn identify(&self, params: IdentifyParams) -> Result<IdentifyResponse>
    pub fn export(&self) -> ExportMapBuilder  // Fluent API
}

// src/services/map/export.rs (361 lines)
impl ExportMapBuilder {
    pub fn bbox(self, bbox: impl Into<String>) -> Self
    pub fn size(self, width: u32, height: u32) -> Self
    pub fn format(self, format: ImageFormat) -> Self
    pub fn transparent(self, transparent: bool) -> Self
    pub fn dpi(self, dpi: i32) -> Self
    pub fn layer_visibility(self, op: LayerOperation, ids: &[i32]) -> Self
    // ... 15+ more fluent methods
    pub async fn execute(self, target: ExportTarget) -> Result<ExportResult>
}
```

**Success Criteria**:
- ✅ Can export map images with full parameter control
- ✅ Can retrieve cached tiles efficiently
- ✅ All 12 image formats properly handled
- ✅ Can retrieve legend graphics for all layers
- ✅ Binary streaming working for all target types
- ✅ Fluent builder provides ergonomic API
- ✅ All validation passing (clippy, fmt, tests, doctests)
- ✅ Comprehensive documentation with examples

### Milestone 4.2: Geocoding Service (Week 11)

**Deliverables**:
- ✅ Forward geocoding (findAddressCandidates)
- ✅ Reverse geocoding
- ✅ Autocomplete/suggest
- ⏸️ Batch geocoding (deferred)
- ✅ Geocoding result types

**Technical Tasks**:
```rust
// src/services/geocode/types.rs
pub struct AddressCandidate {
    pub address: String,
    pub location: Point,
    pub score: f64,
    pub attributes: HashMap<String, serde_json::Value>,
}

// src/services/geocode/client.rs
impl GeocodeServiceClient {
    pub async fn find_address_candidates(&self, address: &str)
        -> Result<Vec<AddressCandidate>> { ... }

    pub async fn reverse_geocode(&self, location: Point)
        -> Result<AddressCandidate> { ... }

    pub async fn suggest(&self, text: &str, max_suggestions: u32)
        -> Result<Vec<Suggestion>> { ... }
}
```

**Success Criteria**:
- ✅ Can geocode addresses
- ✅ Can reverse geocode coordinates
- ✅ Autocomplete working
- ✅ Batch geocoding efficient

### Milestone 4.3: Advanced Queries (Week 12)

**Deliverables**:
- [ ] Spatial queries (intersects, contains, etc.)
- [ ] Related records queries
- [ ] Query statistics
- [ ] Query top features
- [ ] Spatial relationship enums

**Technical Tasks**:
```rust
// src/types/geometry.rs
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum SpatialRel {
    #[serde(rename = "esriSpatialRelIntersects")]
    Intersects,
    #[serde(rename = "esriSpatialRelContains")]
    Contains,
    #[serde(rename = "esriSpatialRelWithin")]
    Within,
    // ... all spatial relationships
}

// Enhance QueryBuilder
impl QueryBuilder {
    pub fn spatial_filter(mut self, geometry: Geometry, rel: SpatialRel) -> Self { ... }
    pub fn query_related_records(self, relationship_id: u32) -> RelatedQueryBuilder { ... }
}
```

**Success Criteria**:
- ✅ All spatial relationship queries working
- ✅ Can query related records
- ✅ Statistics queries returning correct aggregates

**v0.4.0 Release Checklist**:
- [ ] Map, Geocoding services fully functional
- [ ] Advanced query capabilities documented
- [ ] Performance acceptable for production use
- [ ] Published to crates.io

---

## Phase 4: Advanced Services (v0.4.0) - Weeks 11-14

### Milestone 4.1: Geometry Service (Week 11)

**Deliverables**:
- [ ] Project operation (coordinate transformation)
- [ ] Buffer operation
- [ ] Simplify operation
- [ ] Union, Intersect, Difference operations
- [ ] Area and length calculations

**Technical Tasks**:
```rust
// src/services/geometry/client.rs
impl GeometryServiceClient {
    pub async fn project(&self, geometries: Vec<Geometry>,
                         to_sr: SpatialReference) -> Result<Vec<Geometry>> { ... }

    pub async fn buffer(&self, geometries: Vec<Geometry>,
                        distances: Vec<f64>) -> Result<Vec<Polygon>> { ... }

    pub async fn union(&self, geometries: Vec<Geometry>) -> Result<Geometry> { ... }
}
```

**Success Criteria**:
- ✅ Coordinate transformations accurate
- ✅ All geometry operations working
- ✅ Integration with local GeoRust operations

### Milestone 4.2: Routing Service (Week 12)

**Deliverables**:
- [ ] Route calculation (directions)
- [ ] Service area calculation
- [ ] Closest facility
- [ ] Route result types
- [ ] Travel mode enums

**Technical Tasks**:
```rust
// src/services/routing/types.rs
pub struct RouteParameters {
    pub stops: Vec<Point>,
    pub barriers: Vec<Geometry>,
    pub return_directions: bool,
    pub return_routes: bool,
}

pub struct RouteResult {
    pub routes: Vec<Route>,
    pub directions: Vec<DirectionSet>,
}
```

**Success Criteria**:
- ✅ Can calculate routes between points
- ✅ Service areas calculated correctly
- ✅ Directions properly formatted

### Milestone 4.3: Protocol Buffer Support (Week 13)

**Deliverables**:
- [ ] PBF format support for queries
- [ ] Integration with `arcpbf` crate
- [ ] Performance benchmarks
- [ ] Format selection logic

**Technical Tasks**:
```rust
// src/services/feature/query.rs
impl QueryBuilder {
    pub fn response_format(mut self, format: ResponseFormat) -> Self {
        self.params.format = format;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResponseFormat {
    Json,
    GeoJson,
    Pbf,  // High performance binary format
}
```

**Success Criteria**:
- ✅ PBF queries 3-5x faster than JSON
- ✅ Seamless format switching
- ✅ Benchmarks documented

### Milestone 4.4: Geoprocessing Service (Week 14)

**Deliverables**:
- [ ] Execute geoprocessing task
- [ ] Submit job (async tasks)
- [ ] Job status polling
- [ ] Result retrieval
- [ ] Parameter type handling

**Success Criteria**:
- ✅ Can execute synchronous GP tasks
- ✅ Can submit and monitor async jobs
- ✅ Results properly parsed

**v0.4.0 Release Checklist**:
- [ ] All advanced services functional
- [ ] PBF benchmarks published
- [ ] Comprehensive examples
- [ ] Published to crates.io

---

## Phase 5: Production Hardening (v1.0.0) - Weeks 15-20

### Milestone 5.1: Reliability Features (Week 15-16)

**Deliverables**:
- [ ] Automatic retry with exponential backoff
- [ ] Circuit breaker pattern
- [ ] Request timeout configuration
- [ ] Connection pooling optimization
- [ ] Rate limit handling

**Technical Tasks**:
```rust
// src/client.rs
pub struct ClientConfig {
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub timeout: Duration,
    pub circuit_breaker_threshold: u32,
}

impl ArcGISClient {
    pub fn with_config(config: ClientConfig) -> Self { ... }
}
```

**Success Criteria**:
- ✅ Transient failures automatically retried
- ✅ Circuit breaker prevents cascading failures
- ✅ Rate limits respected

### Milestone 5.2: Caching & Performance (Week 17)

**Deliverables**:
- [ ] Response caching layer
- [ ] Service metadata caching
- [ ] Cache invalidation strategy
- [ ] Performance benchmarks
- [ ] Memory usage profiling

**Success Criteria**:
- ✅ Repeated queries hit cache
- ✅ Memory usage acceptable
- ✅ Benchmarks show improvement

### Milestone 5.3: Remaining Services (Week 18)

**Deliverables**:
- [ ] Stream Service (real-time data)
- [ ] Places Service
- [ ] Utility Network Service (if applicable)
- [ ] Version Management Service
- [ ] Any other documented services

**Success Criteria**:
- ✅ All documented services have coverage
- ✅ Service coverage matrix complete

### Milestone 5.4: Documentation & Examples (Week 19)

**Deliverables**:
- [ ] Comprehensive API documentation
- [ ] Tutorial series (getting started → advanced)
- [ ] Example applications (CLI, web service)
- [ ] Migration guides
- [ ] Troubleshooting guide
- [ ] Performance tuning guide

**Success Criteria**:
- ✅ Every public API documented
- ✅ Examples cover common use cases
- ✅ Documentation site published

### Milestone 5.5: Stability & Release (Week 20)

**Deliverables**:
- [ ] Semantic versioning policy
- [ ] Deprecation policy
- [ ] API stability guarantees
- [ ] Security audit
- [ ] Final integration tests
- [ ] Release announcement

**v1.0.0 Release Checklist**:
- [ ] All services implemented
- [ ] Documentation complete
- [ ] Security audit passed
- [ ] Performance benchmarks published
- [ ] Stability guarantees documented
- [ ] Published to crates.io with 1.0.0 tag
- [ ] Blog post / announcement

---

## Service Implementation Priority Matrix

| Service | Priority | Complexity | User Value | Phase |
|---------|----------|------------|------------|-------|
| Feature Service (Query) | P0 | Medium | Critical | 1 |
| Feature Service (Edit) | P0 | Medium | Critical | 2 |
| **Version Management Service** | **P0** | **High** | **Critical** | **3** |
| Map Service | P1 | Low | High | 4 |
| Geocoding Service | P1 | Low | High | 4 |
| Geometry Service | P2 | Medium | Medium | 4 |
| Routing Service | P2 | High | Medium | 4 |
| Geoprocessing Service | P2 | High | Medium | 4 |
| Stream Service | P3 | High | Low | 5 |
| Places Service | P3 | Low | Low | 5 |
| Utility Network | P3 | Very High | Low | 5 |

**Priority Rationale**:
- **P0 - Version Management**: Essential for enterprise GIS workflows with versioned geodatabases. Both Traditional and Branch versioning require this service. Core infrastructure for multi-user editing, conflict resolution, and versioned workflows.

## Type Safety Enforcement Strategy

Throughout all phases, enforce type safety:

### Enumeration Extraction Process

For each service:
1. Read API documentation page
2. Identify all string constants (e.g., `"esriGeometryPoint"`)
3. Create enum with descriptive variant names
4. Implement `serde` serialization with rename attributes
5. Write unit tests for round-trip conversion
6. Update shared types if used across services

### ID Type Strategy

Create newtypes for each ID concept:
- `ServiceId` - Unique service identifier
- `LayerId` - Layer within a service
- `ObjectId` - Feature object ID (can be any field)
- `FeatureId` - Internal feature identifier
- `AttachmentId` - Attachment identifier
- `RelationshipId` - Relationship identifier

### Validation Strategy

For constrained values:
- Use `TryFrom` for conversion from primitives
- Return `Result` from constructors
- Provide validated "smart constructors"
- Document invariants in type docs

## Testing Strategy

### Unit Tests
- Every enum ↔ string conversion
- Every geometry conversion
- Every builder method
- Error handling paths

### Integration Tests
- Against public ArcGIS services (read-only)
- Against test ArcGIS Online organization (read-write, CI)
- Pagination with large datasets
- Error scenarios (rate limits, auth failures)

### Performance Tests
- Query performance benchmarks
- Geometry conversion overhead
- Memory usage profiling
- Concurrent request handling

## Documentation Strategy

### API Documentation
- Every public type documented
- Examples for complex operations
- Link to relevant ArcGIS documentation
- Type safety benefits highlighted

### Guides
- Getting Started (< 5 minutes to first query)
- Authentication Guide (all flows)
- Geometry Guide (conversions, operations)
- Error Handling Guide
- Performance Tuning Guide
- Migration Guides (between versions)

### Examples
- Simple query example
- CRUD operations example
- Spatial query example
- OAuth flow example
- Map export example
- Geocoding example
- Batch operations example

## Success Metrics

### Technical Metrics
- Test coverage > 80%
- Documentation coverage: 100% of public API
- Zero unsafe code (except where necessary, documented)
- Compile time < 2 minutes for clean build
- Binary size < 5MB for basic features

### User Metrics
- Downloads from crates.io
- GitHub stars
- Issue response time < 48 hours
- Community contributions
- Production usage reports

## Risk Mitigation

### API Changes Risk
- **Risk**: ArcGIS API changes break compatibility
- **Mitigation**:
  - Monitor ArcGIS release notes
  - Maintain backward compatibility where possible
  - Use semver for breaking changes
  - Version detection in client

### Type Safety vs Flexibility Risk
- **Risk**: Overly strict types prevent valid use cases
- **Mitigation**:
  - Provide escape hatches (raw methods)
  - Document why types are strict
  - Iterate based on user feedback
  - Use `#[non_exhaustive]` for enums

### Maintenance Burden Risk
- **Risk**: Large API surface becomes unmaintainable
- **Mitigation**:
  - Consistent patterns across services
  - Code generation for repetitive parts (internal tooling)
  - Good test coverage for refactoring safety
  - Community contributions encouraged

### Performance Risk
- **Risk**: Type safety overhead impacts performance
- **Mitigation**:
  - Benchmark early and often
  - Profile memory allocations
  - Use zero-cost abstractions
  - PBF format for high-performance scenarios

## Post-1.0 Roadmap

### v1.1.0 - Offline Support
- Local tile caching
- Offline geocoding with local data
- Sync framework for offline editing

### v1.2.0 - Advanced Features
- WebSocket support for Stream Service
- Server-sent events for subscriptions
- Advanced caching strategies
- Query result streaming

### v2.0.0 - Breaking Improvements
- Lessons learned from v1.x
- API refinements based on usage
- New ArcGIS API features
- Performance optimizations

---

## Appendix: Quick Start Timeline

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1 | Infrastructure | Core client, auth, errors |
| 2 | Feature Query | Basic query working |
| 3 | Polish | Pagination, docs, v0.1.0-alpha |
| 4 | Feature Edit | Add, update, delete |
| 5 | Batch Edits | ApplyEdits operation |
| 6 | OAuth | OAuth 2.0 flows, v0.2.0 |
| 7-8 | Map Service | Export maps and tiles |
| 9 | Geocoding | Address lookup |
| 10 | Advanced Query | Spatial queries, v0.3.0 |
| 11 | Geometry Service | Spatial operations |
| 12 | Routing | Route calculation |
| 13 | PBF Support | High-performance queries |
| 14 | Geoprocessing | GP tasks, v0.4.0 |
| 15-16 | Reliability | Retry, circuit breaker |
| 17 | Performance | Caching, optimization |
| 18 | Remaining Services | Stream, Places, etc. |
| 19 | Documentation | Comprehensive guides |
| 20 | Release | v1.0.0 |

**Total Timeline**: ~20 weeks (5 months) from start to v1.0.0

---

**Document Version**: 1.3
**Last Updated**: January 4, 2026
**Status**: Active Development (Phase 3 complete, Phase 4 in progress)
