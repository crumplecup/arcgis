# ArcGIS REST API Coverage Roadmap

**Current Status**: v0.1.0 - 113 operations across 12 services
**Target**: 100% coverage of ArcGIS REST API surface area
**Last Updated**: 2026-01-11

---

## Current State: v0.1.0

### Implemented Services (12 total)

| Service | Operations | Status |
|---------|-----------|--------|
| **Elevation** | 3 | Complete ✅ |
| **Feature** | 17 | Complete ✅ |
| **Geocode** | 9 | Complete ✅ |
| **Geometry** | 8 | Core Complete ✅ |
| **Geoprocessing** | 7 | Complete ✅ |
| **Image** | 6 | Core Complete ✅ |
| **Map** | 10 | Complete ✅ |
| **Places** | 3 | Complete ✅ |
| **Portal** | 24 | Core Complete ✅ |
| **Routing** | 4 | Complete ✅ |
| **Vector Tile** | 6 | Core Complete ✅ |
| **Version Management** | 16 | Complete ✅ |
| **TOTAL** | **113** | **Baseline Established** |

---

## Missing Services

### High Priority (Essential for GIS Workflows)

1. **Scene Service** (3D visualization)
   - Query 3D features
   - Get scene layer metadata
   - Statistics and object IDs
   - Layer types: 3D Object, Integrated Mesh, Point Cloud, Building
   - **Estimated Operations**: 6-8

2. **Stream Service** (Real-time data)
   - WebSocket connections
   - Subscribe/unsubscribe to streams
   - Filter stream events (spatial, attribute, temporal)
   - **Estimated Operations**: 4-6

3. **Utility Network Service** (Network management)
   - Trace operations
   - Network diagram queries
   - Topology validation
   - Association queries
   - **Estimated Operations**: 8-12

### Medium Priority (Specialized Use Cases)

4. **GeoEnrichment Service** (Demographics)
   - Enrich locations with demographic data
   - Data collection queries
   - Variable introspection
   - **Estimated Operations**: 3-5

5. **Network Diagram Service** (Network visualization)
   - Generate network diagrams
   - Layout algorithms
   - Diagram persistence
   - **Estimated Operations**: 4-6

6. **Parcel Fabric Service** (Land records)
   - Parcel queries
   - Topology operations
   - Historic lineage
   - **Estimated Operations**: 6-8

### Low Priority (Administrative/Specialized)

7. **Printing Service** (Map export)
   - Print templates
   - Layout management
   - Export to PDF/PNG/JPG
   - **Estimated Operations**: 3-4

8. **Knowledge Graph Service** (Graph analytics)
   - Entity/relationship queries
   - Graph analytics
   - Path finding
   - **Estimated Operations**: 6-10

9. **Schematic Service** (Legacy diagrams)
   - Diagram generation
   - Layout operations
   - **Estimated Operations**: 4-5
   - **Note**: Legacy - may deprecate

10. **Geostatistics Service** (Statistical analysis)
    - Kriging interpolation
    - Trend analysis
    - Spatial statistics
    - **Estimated Operations**: 4-6

11. **OGC Services** (Interoperability)
    - WMS (Web Map Service)
    - WFS (Web Feature Service)
    - WCS (Web Coverage Service)
    - WMTS (Web Map Tile Service)
    - **Estimated Operations**: 12-20

12. **Administration Services** (Server management)
    - Service management (start/stop/configure)
    - Log queries
    - System health monitoring
    - User/role management
    - **Estimated Operations**: 15-25

---

## Missing Operations in Existing Services

### Geometry Service - Advanced Operations (10 operations)

**Why Missing**: Deferred in initial implementation as less commonly used

1. `intersect` - Find geometric intersections
2. `difference` - Compute geometric difference
3. `generalize` - Douglas-Peucker generalization
4. `offset` - Offset curves and polygons
5. `cut` - Cut geometry with polyline
6. `reshape` - Reshape with polyline
7. `densify` - Add vertices along segments
8. `trim_extend` - Trim or extend polylines
9. `convex_hull` - Compute convex hull
10. `label_points` - Find optimal label placement

**Priority**: Low (covered by 3rd party geometry libraries)

---

### Image Service - Advanced Operations (6 operations)

**Why Missing**: Core operations sufficient for most workflows

1. `measure` - Measure distances/areas on imagery
2. `compute_statistics_histograms` - Full statistics computation
3. `get_catalog_items` - Query raster catalog
4. `download` - Download raster file
5. `project` - Reproject imagery
6. `metadata` - Extended raster metadata

**Priority**: Medium

---

### Portal Service - Extended Operations (12+ operations)

**Why Missing**: Initial implementation focused on core workflows

**Content Management:**
1. `move_items` - Move items between folders
2. `copy_items` - Copy items to new location
3. `protect_item` / `unprotect_item` - Item protection
4. `reassign_item` - Change item ownership

**Folders:**
5. `create_folder` - Create user folder
6. `delete_folder` - Delete folder
7. `add_item_to_folder` - Organize items

**Living Atlas:**
8. `get_living_atlas_items` - Query curated content
9. `subscribe_to_living_atlas` - Subscribe to layers

**Webhooks:**
10. `create_webhook` - Event notifications
11. `list_webhooks` - Get webhooks
12. `delete_webhook` - Remove webhook

**Priority**: Medium-High (workflow automation)

---

### Vector Tile Service - Extended Operations (2 operations)

**Why Missing**: Core functionality complete

1. `get_service_metadata` - Extended service info
2. `get_sources` - Get tile sources metadata

**Priority**: Low

---

### Feature Service - Extended Operations (4 operations)

**Why Missing**: Advanced administrative operations

1. `generate_renderer` - Generate renderer for layer
2. `validate_sql` - Validate SQL expressions
3. `add_to_definition` / `delete_from_definition` - Modify service schema
4. `refresh` - Refresh materialized views

**Priority**: Medium

---

## Complete Coverage Estimate

### Current Coverage Breakdown

**Tier 1 Services** (Essential - must have):
- Feature Service ✅
- Map Service ✅
- Geocoding Service ✅
- Geometry Service ✅ (core)
- Version Management ✅
- Portal ✅ (core)

**Tier 2 Services** (Important - common workflows):
- Routing/Network ✅
- Geoprocessing ✅
- Image Service ✅ (core)
- Vector Tile ✅ (core)
- Elevation ✅
- Places ✅

**Tier 3 Services** (Specialized - specific use cases):
- Scene Service ❌
- Stream Service ❌
- Utility Network ❌
- GeoEnrichment ❌
- Network Diagram ❌
- Parcel Fabric ❌
- Printing ❌

**Tier 4 Services** (Administrative/Legacy):
- Knowledge Graph ❌
- Schematic Service ❌
- Geostatistics ❌
- OGC Services ❌
- Administration Services ❌

---

### Coverage Calculation

**Current Implementation**:
- 113 operations
- 12 services (all Tier 1 & 2)
- Estimated ~65% of essential functionality

**To Reach 100% (Full ArcGIS REST API)**:
- Add 7 Tier 3 services: ~40-50 operations
- Add 5 Tier 4 services: ~45-70 operations
- Complete existing services: ~35 operations
- **Total Estimated**: 233-268 total operations

**Realistic "Full Coverage" Target** (95% use case coverage):
- Complete all Tier 1 & 2 services: +10 operations
- Add 4-5 Tier 3 services: +25-30 operations
- Skip Tier 4 (administrative/legacy): defer
- **Target**: 150-160 operations (70% of total API surface, 95% of use cases)

---

## Roadmap to Full Coverage

### Phase 1: Complete Existing Services (15-20 operations)

**Goal**: Fill gaps in implemented services

**Image Service Extensions** (6 operations)
- `measure` - Mensuration on imagery
- `get_catalog_items` - Query raster catalog
- `download` - Download rasters
- `project` - Reproject imagery
- `compute_statistics_histograms` - Full statistics
- `get_metadata` - Extended metadata

**Portal Service Extensions** (12 operations)
- `move_items` / `copy_items` - Content management
- `protect_item` / `unprotect_item` - Item protection
- `create_folder` / `delete_folder` - Folder management
- `add_item_to_folder` - Organization
- `get_living_atlas_items` - Curated content
- `create_webhook` / `list_webhooks` / `delete_webhook` - Automation
- `reassign_item` - Ownership transfer

**Feature Service Extensions** (4 operations)
- `generate_renderer` - Dynamic symbolization
- `validate_sql` - Query validation
- `add_to_definition` / `delete_from_definition` - Schema modification

**Geometry Service Extensions** (optional - low priority)
- Defer to geometry libraries (geo, geos)

---

### Phase 2: Essential Tier 3 Services (25-35 operations)

**Scene Service** (6-8 operations)
- `query_3d_features` - Query 3D objects
- `get_layer_info` - Scene layer metadata
- `get_statistics` - 3D feature statistics
- `query_object_ids` - Get object IDs
- `get_node_pages` - Retrieve I3S node pages
- `get_textures` / `get_geometries` - Asset retrieval

**Stream Service** (4-6 operations)
- `connect` - WebSocket connection
- `subscribe` / `unsubscribe` - Stream management
- `filter_stream` - Apply filters (spatial, attribute, temporal)
- `get_stream_info` - Stream metadata
- `disconnect` - Close connection

**GeoEnrichment Service** (3-5 operations)
- `enrich` - Enrich locations with demographics
- `get_data_collections` - Available data
- `get_variables` - Variable introspection
- `create_report` - Generate enrichment reports

**Utility Network Service** (8-12 operations)
- `trace` - Network trace operations
- `query_associations` - Association queries
- `validate_topology` - Topology validation
- `get_network_diagrams` - Diagram retrieval
- `query_diagram_info` - Diagram metadata
- `create_diagram` - Generate diagrams
- `update_diagram` - Modify diagrams
- `delete_diagram` - Remove diagrams

---

### Phase 3: Additional Tier 3 Services (15-20 operations)

**Network Diagram Service** (4-6 operations)
- `generate_diagram` - Create network diagrams
- `apply_layout` - Layout algorithms
- `save_diagram` - Persist diagrams
- `get_diagram_info` - Metadata
- `delete_diagram` - Cleanup

**Parcel Fabric Service** (6-8 operations)
- `query_parcels` - Parcel queries
- `get_lineage` - Historic lineage
- `validate_topology` - Topology checks
- `create_seeds` - Create seeds
- `build_parcels` - Build from COGO
- `assign_to_record` - Record management

**Printing Service** (3-4 operations)
- `get_layout_templates` - Available templates
- `export_web_map` - Print map
- `get_service_info` - Service metadata

---

### Phase 4: Tier 4 Services (Optional - defer indefinitely)

**Knowledge Graph Service** (6-10 operations)
- Advanced graph analytics
- Specialized use case
- Low demand

**OGC Services** (12-20 operations)
- WMS, WFS, WCS, WMTS implementations
- Better served by dedicated OGC libraries
- Interoperability focus

**Administration Services** (15-25 operations)
- Server management
- Monitoring and logging
- User/role administration
- Highly specialized, server-admin focused
- Not typical SDK use case

**Geostatistics Service** (4-6 operations)
- Kriging, trend analysis
- Better served by statistical libraries (SciPy, etc.)

**Schematic Service** (4-5 operations)
- Legacy service
- Replaced by Network Diagrams
- Deprecation candidate

---

## Implementation Priority Matrix

### Critical Path to 70% Coverage (Phase 1 + Phase 2)

**High Impact, High Demand**:
1. Portal extensions (workflows, automation)
2. Scene Service (3D visualization)
3. Stream Service (real-time IoT)
4. Image Service extensions (remote sensing)

**Medium Impact, Moderate Demand**:
5. Utility Network (network industries)
6. GeoEnrichment (demographics)
7. Feature Service extensions (advanced admin)

### Extended Coverage to 85% (Phase 3)

**Specialized, Lower Demand**:
8. Network Diagram (utilities)
9. Parcel Fabric (land records)
10. Printing Service (map export)

### Optional/Future (Phase 4)

**Low Priority, Niche**:
11. Knowledge Graph
12. Geostatistics
13. OGC Services
14. Administration
15. Schematic (legacy)

---

## Coverage Milestones

| Milestone | Operations | Services | Coverage | Use Case Coverage |
|-----------|-----------|----------|----------|-------------------|
| **v0.1.0** (Current) | 113 | 12 | 65% | 80% |
| **v0.2.0** (Phase 1) | 133 | 12 | 70% | 85% |
| **v0.3.0** (Phase 2) | 163 | 16 | 75% | 92% |
| **v0.4.0** (Phase 3) | 183 | 19 | 80% | 95% |
| **v1.0.0** (Target) | 183-200 | 19-20 | 80-85% | 95%+ |

---

## Implementation Estimates

### Phase 1: Complete Existing Services
- **Operations**: 15-20
- **Effort**: 3-4 weeks
- **Complexity**: Medium (extending existing patterns)

### Phase 2: Essential Tier 3 Services
- **Operations**: 25-35
- **Effort**: 6-8 weeks
- **Complexity**: High (new service types, WebSockets, 3D)

### Phase 3: Additional Tier 3 Services
- **Operations**: 15-20
- **Effort**: 4-5 weeks
- **Complexity**: Medium-High (specialized domains)

### Phase 4: Optional Tier 4 Services
- **Operations**: 45-70
- **Effort**: 10-15 weeks
- **Complexity**: High (administrative, legacy, interop)
- **Priority**: Deferred (low ROI)

---

## Success Metrics

### Coverage Metrics
- Operations count
- Service count
- % of total API surface area
- % of use case coverage (survey-based)

### Quality Metrics
- Zero clippy warnings
- 80%+ test coverage
- 100% public API documented
- All examples compile

### Community Metrics
- GitHub stars
- crates.io downloads
- Contributors
- Issue resolution time

---

## Notes

**What "100% Coverage" Means**:
- We define 100% as implementing all Tier 1-3 services comprehensively
- Tier 4 services are optional (administrative, legacy, better served elsewhere)
- Target is 80-85% of total API surface, achieving 95%+ of use cases
- This is a living document - ArcGIS REST API evolves, we track and adapt

**Version Numbering Philosophy**:
- New features do not increment version
- Breaking API changes increment version
- Follow semantic versioning (MAJOR.MINOR.PATCH)
- Track progress by operation count and coverage %, not version numbers

**Maintenance Strategy**:
- Update this roadmap quarterly
- Track new ArcGIS REST API additions
- Prioritize based on community feedback
- Deprecate when Esri deprecates
