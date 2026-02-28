## [0.1.3] - 2026-02-28

### 🚀 Features

- *(portal)* Add portal_group_workflow example and fix group API type handling
- *(portal)* Replace update_item_data with type-safe ItemDataUpload enum
- *(examples)* Add comprehensive item data upload examples
- *(examples)* Add OAuth group membership operations example
- *(examples)* Add service management operations example
- *(examples)* Add geoprocessing job cancellation example
- *(examples)* Add assertions to feature_attachments.rs
- *(examples)* Add assertions to geoprocessing_tools.rs
- *(examples)* Add assertions to elevation_analysis.rs
- *(examples)* Add assertions to elevation_async_analysis.rs
- *(examples)* Add assertions to image_service_raster.rs
- *(examples)* Add assertions to vector_tiles.rs
- *(examples)* Add assertions to map_service_basics.rs
- *(examples)* Add assertions to portal_content_management.rs
- *(examples)* Add assertions to client_credentials_flow.rs
- *(examples)* Enhance map_service_basics.rs assertions to 21 (Excellent coverage)
- *(examples)* Achieve 100% MapServiceClient coverage (9/9 methods)
- *(geocoding)* Complete GeocodeServiceClient coverage to 100%
- *(version-management)* Add version_management_basics.rs example with ARCGIS_FEATURE_URL
- *(geometry)* Add type-safe ProjectedPoint system for spatial references
- *(testing)* Add example execution tracking system
- *(examples)* Add comprehensive feature service batch editing examples
- *(version-management)* Achieve 100% coverage with comprehensive demonstrations
- *(examples)* Integrate execution tracking across all examples

### 🐛 Bug Fixes

- *(examples)* Update examples to use new ItemDataUpload API
- *(examples)* Remove needless borrows in portal_service_management
- *(examples)* Fix geometry_advanced, map_service_basics, and portal_item_data_text examples
- *(examples)* Fix geocoding_batch_operations reverse geocoding with custom SR
- *(portal)* Add ESRI error checking to all portal client methods
- *(examples)* Update portal_service_management to use file_type parameter
- *(portal)* Handle GeoJSON publish response structure in PublishResult
- *(portal)* Fix publish timeout and add ESRI error handling
- *(pre-merge)* Fix geocode doctests and formatting for v0.1.3

### 🚜 Refactor

- *(portal)* Remove redundant manual setters in PublishParameters

### 📚 Documentation

- Update gap analysis and planning documents for PortalClient completion
- Update gap analysis for GeoprocessingServiceClient completion
- Update assertion audit with completed examples
- *(assertions)* Update audit with medium-priority progress
- *(assertions)* Complete medium-priority phase - 26/30 examples with assertions!
- *(audit)* Complete assertion audit - 100% coverage achieved! 🎉
- *(gap-analysis)* Update for MapServiceClient 100% completion
- *(gap-analysis)* Defer PlacesClient testing - requires Location Platform
- *(gap-analysis)* Update coverage to 97% with latest achievements
- Archive completed planning documents
- Archive completed documentation files in docs/
- Add changelog for v0.1.3 release

### 🧪 Testing

- *(examples)* Add comprehensive assertions to geoprocessing_execution_modes
- *(examples)* Add comprehensive assertions to geometry_operations
- *(examples)* Add 10 assertions to geocode_addresses
- *(examples)* Add 11 assertions to query_features
- *(examples)* Add 10 assertions to spatial_query

### ⚙️ Miscellaneous Tasks

- *(gitignore)* Exclude example output and tracking files
- Remove unused ArcGISPoint import
## [0.1.2] - 2026-02-16

### 🚀 Features

- *(auth)* Add agol() and enterprise() helper methods
- *(geoprocessing)* Add result data fetching and comprehensive logging
- *(elevation)* Implement async SummarizeElevation and Viewshed operations
- *(examples)* Add elevation_async_analysis demonstrating async GP operations
- *(examples)* Add advanced feature queries example
- *(examples)* Add geometry_advanced example (partial implementation)
- *(examples)* Add portal_item_lifecycle example
- *(geometry)* Add simplify, union, and areasAndLengths operations
- *(examples)* Extend advanced_queries with count and params demos
- *(examples)* Add feature_service_metadata example
- *(examples)* Add geocoding_batch_operations example (partial)
- *(image)* Add image_service_identify_advanced example and fix identify_with_params
- *(geoprocessing)* Add job monitoring example and fix GP types
- *(feature)* Add field calculations support with calculate_records
- *(logging)* Improve calculate_records instrumentation and example guidance
- *(routing)* Add comprehensive examples and fix type mismatches

### 🐛 Bug Fixes

- *(feature)* Add JSON serializers for complex query parameters
- *(geometry)* Add SpatialReference builder and fix find_transformations response
- *(portal)* Add default serde attribute to UnshareItemResult.success
- Correct type definitions and remove broken batch geocoding
- *(geocode)* Implement proper BatchGeocodeRecord type for geocode_addresses
- *(pre-merge)* Fix clippy warnings, doctests, and test assertions

### 💼 Other

- Exclude log files from repository

### 📚 Documentation

- *(standards)* Require instrumentation on ALL functions
- Update coverage statistics after portal_item_lifecycle

### 🎨 Styling

- *(examples)* Format advanced_queries.rs with rustfmt
## [0.1.1] - 2026-02-09

### 🚀 Features

- *(auth)* Add ApiKeyTier enum for privilege-separated authentication
- *(examples)* Add geoprocessing_tools example
- *(examples)* Add portal_publishing example with dual workflows
- *(examples)* Add vector_tiles example demonstrating MVT operations
- *(examples)* Add places_poi_search example demonstrating POI discovery
- *(examples)* Add elevation_analysis example demonstrating terrain analysis
- *(geo)* Add ESRI geometry integration planning and implementation
- *(portal)* Add strongly-typed service definitions (Phase 1)
- *(portal)* Integrate strongly-typed service definitions (Phase 2)
- *(portal)* Add phase 3 service definition types and relationship classes
- *(portal)* Implement TableDefinition for phase 4 service definition types
- *(portal)* Add phase 5 service definition validation with agent-friendly errors
- *(feature)* Add phase 6 service definition retrieval
- *(portal)* Implement addToDefinition for hosted feature services

### 🐛 Bug Fixes

- *(portal)* Add OAuth2 support with AppInfo and optional fields
- *(geoprocessing)* Make result parameters optional and fix message type serialization
- *(places)* Add authentication token handling and builder defaults
- *(elevation)* Add authentication token handling
- *(elevation)* Correct service URL and add builder defaults
- *(geometry)* Address clippy warnings and formatting
- *(geometry)* Restore untagged serde for ArcGISGeometry enum
- *(elevation)* Use FeatureSet instead of raw JSON Value
- *(elevation)* Correct API parameters and add observability
- *(docs)* Correct doctest examples and apply code quality fixes
- *(services)* Add ESRI error response checking to prevent silent failures

### 🚜 Refactor

- *(examples)* Pivot portal_publishing Workflow B to item data management
- *(geometry)* Copy superior implementation from geo module
- *(geometry)* Expose new types alongside legacy types
- *(geometry)* Migrate geometry service to new types
- *(geometry)* Migrate routing service to new types
- *(geometry)* Migrate feature service to new types
- *(geometry)* Migrate remaining services to new types
- *(geometry)* Migrate examples to new types
- *(geometry)* Remove legacy implementation and V2 suffixes
- *(geometry)* Move inline tests to tests directory
- *(geometry)* Complete cleanup - remove geo module and consolidate types
- *(examples)* Add workflow validation assertions to portal_publishing

### 📚 Documentation

- *(examples)* Update examples to use ApiKeyTier system
- *(geometry)* Mark geometry consolidation complete
- *(examples)* Update elevation_analysis to use corrected API
- *(examples)* Update elevation and geoprocessing examples
- *(portal)* Fix inaccurate branch versioning claims in service definition
- Add comprehensive gap analysis (2026-02-08)

### 🎨 Styling

- Apply cargo fmt to service files

### ⚙️ Miscellaneous Tasks

- Simplify CI workflow to essential checks on main
- Install protobuf compiler for build.rs
- Remove tests until API keys migrated to GitHub secrets
- *(examples)* Remove outdated edit_session and branch_versioning_workflow
- Prepare release v0.1.1

### ◀️ Revert

- Remove places_poi_search example (requires Location Platform account)
## [0.1.0] - 2026-01-26

### 🚀 Features

- Initial SDK foundation with type-safe architecture
- Implement geometry conversion layer (Milestone 1.2)
- *(milestone-1.3)* Complete Feature Query API implementation
- *(milestone-1.4)* Implement QueryBuilder with auto-pagination
- *(auth)* Implement OAuth 2.0 + PKCE authentication provider
- *(auth)* Implement OAuth 2.0 Client Credentials Flow for automated authentication
- *(feature-service)* Implement Phase 3 CRUD operations
- *(geocode)* Implement Geocoding Service (Phase 4, Milestone 4.2)
- *(version-mgmt)* Implement Version Management Server with edit sessions
- *(version-mgmt)* Implement version lifecycle operations
- *(version_management)* Implement read session operations
- *(version_management)* Implement reconcile and post operations
- *(version_management)* Implement conflict management operations
- *(version_management)* Implement differences and restore_rows operations
- *(feature_service)* Implement attachment operations with streaming support
- *(map_service)* Implement comprehensive Map Service support with binary streaming
- *(feature_service)* Implement statistics queries with GROUP BY and HAVING
- *(feature_service)* Implement related records queries with comprehensive parameter support
- *(feature_service)* Implement query top features with group-based ranking
- Expand service client APIs with batch operations and data management
- Complete Phase 1 with 100% Tier 1 service coverage
- *(geometry_service)* Implement comprehensive geometry service operations
- *(routing_service)* Implement routing service with route operation
- *(routing)* Add Service Area, Closest Facility, and OD Cost Matrix operations
- *(geoprocessing)* Implement complete geoprocessing service
- *(vector_tile)* Implement complete vector tile service
- *(image_service)* Implement comprehensive raster operations
- *(places_service)* Implement POI search and details
- *(elevation_service)* Implement terrain analysis operations
- *(pbf)* Add Protocol Buffer format support foundation
- *(feature_service)* Integrate PBF format support into query methods
- *(feature_service)* Implement PBF geometry decoding for all geometry types
- *(feature_service)* Add convenient builder methods for response format selection
- *(portal)* Implement Portal/Content Service foundation with reqwest 0.13 upgrade
- Add free-tier public testing with CI/CD support
- *(justfile)* Add omnibor and cargo-dist support with pre-publish workflow
- *(justfile)* Add setup recipe for installing development tools
- *(justfile)* Use /tmp for check-all logs instead of local directory
- *(justfile)* Modernize tooling and workflows
- Add automatic .env loading and secure credential management
- *(examples)* Add comprehensive query_features and geocode_addresses examples
- *(examples)* Add geometry_operations and spatial_query examples
- *(auth,error,examples)* Add ApiKeyAuth::from_env() with proper error handling and fix GeoJSON/PBF support
- *(auth)* Add ClientCredentialsAuth::from_env() helper method
- *(examples)* Add routing_navigation.rs - comprehensive routing example
- *(examples)* Add portal content management example
- *(tests)* Migrate from 'api' feature to privilege-tier features
- *(tests)* Implement multi-tier API key configuration with config crate
- *(config)* Centralize environment variable loading with EnvConfig
- *(examples)* Add map_service_basics.rs public example
- *(examples)* Add image_service_raster example with ImageService bug fixes

### 🐛 Bug Fixes

- Apply CLAUDE.md compliance to Feature Service implementation
- *(auth)* Protect client_secret with SecretString
- *(feature_service)* Add custom serializers for URL query parameters
- *(docs)* Use correct GitHub username in clone URL
- Convert private field access to accessor method calls in tests and doctests
- *(auth)* Support NoAuth provider for public services
- *(auth)* Complete NoAuth support for remaining service clients
- *(geometry)* Fix project operation parameter handling and add error response detection
- *(geometry,examples)* Geometry service is free (no auth) but API format issues remain
- *(geometry)* PROJECT OPERATION NOW WORKS! Fixed geometries parameter format
- *(geometry)* Buffer and distance operations now fully functional ✅
- *(routing)* Add builder defaults to routing parameter types
- *(examples)* Refocus portal example on discovery operations
- *(attachments)* Handle nested API response structures for add/update operations
- *(routing)* Fix service area polygon deserialization and add missing parameters
- *(routing)* Fix closest facility API parameter and response field names
- *(map)* Fix relative URL handling and FindParams builder
- *(map)* Improve error handling and update example to use dynamic MapServer
- *(examples)* Add bbox_sr to image export to fix solid black output
- Include examples in published package

### 💼 Other

- *(dist)* Add cargo-dist configuration

### 🚜 Refactor

- Bring codebase to CLAUDE.md compliance standards
- Enforce CLAUDE.md module visibility compliance
- Eliminate feature gate complexity
- *(error)* Implement exceptional error handling with type preservation
- *(error)* Eliminate unwrap_or_default and enhance error exports
- *(tests)* Eliminate all .expect and .unwrap calls
- *(auth)* Replace .expect() with proper error handling
- Replace public struct fields with private fields and derives
- *(portal)* Split monolithic client into modular structure
- *(version_management)* Modularize client into focused domain files
- *(feature)* Split FeatureServiceClient into modular structure
- *(examples)* Adopt structured logging and proper error handling
- *(tests)* Remove 'api' feature and relocate integration helpers
- *(examples)* Organize into public and enterprise subdirectories
- *(auth)* Rename CLIENT_ID/CLIENT_SECRET to ARCGIS_CLIENT_ID/ARCGIS_CLIENT_SECRET
- *(examples)* Improve feature_attachments readability with EnvConfig and named functions
- *(examples)* Improve edit_session readability with EnvConfig and named functions
- *(examples)* Improve geocode_addresses readability with named functions
- *(examples)* Improve geometry_operations readability with named functions
- *(examples)* Improve portal_content_management readability with named functions
- *(examples)* Improve routing_navigation readability with named functions
- *(examples)* Extract public examples into named demonstration functions

### 📚 Documentation

- Update repository URLs to actual GitHub location
- Add CLAUDE.md compliance review
- Add planning document index
- Archive historical documentation
- Prioritize OAuth authentication as Phase 1
- Update README to reflect Phase 2 completion
- Update IMPLEMENTATION_PLAN.md to reflect Phase 3 and 4.2 completion
- Update implementation plan with Version Management Service roadmap
- *(plan)* Update implementation plan to reflect current state
- *(plan)* Update implementation plan for Phase 4.3 statistics completion
- Add comprehensive full coverage implementation plan
- Update planning documents to reflect Phase 1 progress
- Mark Phase 1 as complete in full coverage plan
- Update tracking plan for Phase 2 Geometry Service completion
- Update tracking plan for Phase 3 Routing Service completion
- Update planning index for Phase 3 completion
- Update tracking plan for Phase 4 Geoprocessing Service completion
- Update planning index for Phase 4 completion
- Update FULL_COVERAGE_PLAN.md for Phase 7 completion
- Update PLANNING_INDEX.md with Phase 7 completion
- Update FULL_COVERAGE_PLAN.md for Places Service completion
- Update PLANNING_INDEX.md with Places Service completion
- Update FULL_COVERAGE_PLAN.md for Elevation Service completion
- Update PLANNING_INDEX.md with Elevation Service completion
- Update Phase 5 status to reflect PBF implementation completion
- Replace FULL_COVERAGE_PLAN with fresh COVERAGE_ROADMAP
- Update README and roadmap to reflect v0.1.0 baseline
- Add comprehensive API key testing strategy
- Rewrite CONTRIBUTING.md to encourage early user feedback
- *(examples)* Comprehensive README update documenting all examples
- Add comprehensive examples expansion plan
- *(multi-tier-testing)* Add granular permission checklists for each tier
- *(multi-tier-testing)* Replace fabricated checkboxes with actual ArcGIS privilege strings
- *(multi-tier-testing)* Use actual UI labels instead of machine-readable privilege strings
- Add comprehensive example coverage assessment

### 🎨 Styling

- Apply cargo fmt formatting

### 🧪 Testing

- Add integration test infrastructure with .env support
- Add comprehensive tests for ID newtypes
- Standardize test infrastructure with tracing and anyhow

### ⚙️ Miscellaneous Tasks

- Add GitHub Actions workflows and Dependabot config
- Update workflow to use main branch only
- Run CI only on main branch to reduce spam
- Ignore map export example output files
- Run cargo fmt and update config crate to 0.15
- Update Cargo.lock for config 0.15
