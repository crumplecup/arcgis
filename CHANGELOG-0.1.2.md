# Changelog - v0.1.2

## Overview

This release significantly expands test coverage and fixes numerous type mismatches discovered through comprehensive testing. Major improvements include routing service examples, geoprocessing async operations, geometry operations, and field calculations.

## New Features

### Routing Services
- **Complete routing_navigation example** - Tests all 4 routing methods with comprehensive assertions
  - Service area analysis (drive-time polygons)
  - Closest facility routing (nearest service finder)
  - Origin-destination cost matrix (2×2 minimal matrix)
  - Enhanced existing route example with validation
- **Fixed multiple type mismatches** in routing types discovered during testing
  - Route field names now match API (Total_Miles, Total_TravelTime)
  - ClosestFacilityResult uses custom deserializer for FeatureSet conversion
  - ODCostMatrixResult handles nested map response format
  - NALocation::from_feature() for facilities/incidents
- **Fixed OD cost matrix endpoint** from `/generateOriginDestinationCostMatrix` to `/solveODCostMatrix`
- **Added impedance/accumulate attribute parameters** to routing methods

### Geoprocessing
- **Async GP operations** - SummarizeElevation and Viewshed with job monitoring
- **Job monitoring example** - Demonstrates async job polling and result fetching
- **Result data fetching** - Comprehensive support for GP output parameters
- **Fixed GP types** - ViewshedResult, SummarizeElevationResult now deserialize correctly

### Feature Services
- **Field calculations** - calculate_records() method for bulk field updates
  - Scalar value updates
  - SQL expression calculations
  - Multiple field calculations
- **Enhanced query examples** - Count queries, parameterized queries
- **Metadata example** - Feature service/layer metadata inspection
- **Advanced queries example** - Spatial queries, statistics, relationship queries

### Geometry Operations
- **New operations** - simplify, union, areasAndLengths
- **Advanced geometry example** - Demonstrates buffer, project, densify
- **SpatialReference builder** - Easy construction of spatial references
- **Fixed find_transformations response** - Proper deserialization

### Geocoding
- **Batch geocoding** - Proper BatchGeocodeRecord type implementation
- **Geocoding operations example** - Batch geocoding, reverse geocoding

### Image Services
- **Advanced identify example** - Catalog items, pixel values with params
- **Fixed identify_with_params** - Proper parameter handling

### Portal/Content
- **Portal item lifecycle example** - Create, share, update, delete items
- **Fixed UnshareItemResult** - Added default serde attribute

### Authentication
- **Helper methods** - agol() and enterprise() for common configurations

## Bug Fixes

### Type Mismatches (discovered through testing)
- Route fields now use correct API names (Total_Miles vs total_length)
- ClosestFacilityResult facilities/incidents now deserialize from FeatureSets
- ODCostMatrixResult handles nested map format from API
- BatchGeocodeRecord proper type definition
- ViewshedResult, SummarizeElevationResult deserialization
- UnshareItemResult.success default attribute
- Feature query parameter JSON serialization

### API Endpoints
- OD cost matrix endpoint corrected to /solveODCostMatrix
- Image service identify parameters fixed

## Documentation

### Examples Added
- routing_navigation.rs - All 4 routing methods with minimal API usage
- elevation_async_analysis.rs - Async GP job monitoring
- geoprocessing_job_monitoring.rs - Job status polling patterns
- feature_service_field_calculations.rs - Bulk field updates
- feature_service_metadata.rs - Metadata inspection
- advanced_queries.rs - Complex query scenarios
- geometry_advanced.rs - Geometry operation workflows
- geocoding_batch_operations.rs - Batch geocoding patterns
- image_service_identify_advanced.rs - Advanced identify operations
- portal_item_lifecycle.rs - Content management workflows

### Coverage Statistics
- RoutingServiceClient: 25% → 100%
- GeoprocessingServiceClient: Significantly improved async operations
- FeatureServiceClient: Enhanced with calculate_records
- GeometryServiceClient: Added 3 new operations
- ImageServiceClient: Fixed identify operations

## Instrumentation Improvements

- Enhanced calculate_records logging with detailed field tracking
- Comprehensive GP job monitoring logs
- All new functions properly instrumented per standards
- Better error context in deserialization failures

## Project Structure

- Flattened examples directory (removed subfolders)
- Version bumped to 0.1.2
- Author field formatting fixed

## Breaking Changes

None - All changes are additions or fixes to existing broken functionality.

## Testing

All examples tested with real ArcGIS services:
- ✅ Routing examples pass with public routing services
- ✅ GP async operations tested with elevation services
- ✅ Geometry operations verified
- ✅ Field calculations tested (requires owned service)
- ✅ Geocoding operations verified

## Methodology

**Pattern discovered**: "untested = broken" due to serialization/deserialization issues

**Testing approach**:
1. Research ESRI API documentation
2. Fix type definitions to match actual API responses
3. Create comprehensive examples with assertions
4. Test with real API
5. Fix bugs discovered during testing

This methodical approach uncovered numerous type mismatches that would have caused runtime failures.

## Credits

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
