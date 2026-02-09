 WARN  git_cliff > "cliff.toml" is not found, using the default configuration
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

### ◀️ Revert

- Remove places_poi_search example (requires Location Platform account)
