# Async Elevation Service Support Implementation Plan

**Date:** 2026-02-02
**Branch:** `feat/async-elevation`
**Goal:** Add async geoprocessing support for SummarizeElevation and Viewshed operations

---

## Executive Summary

The ArcGIS Elevation Service has two different execution patterns:
- **Profile**: Synchronous (ElevationSync GPServer) - ✅ **Already working**
- **SummarizeElevation & Viewshed**: Asynchronous (Elevation GPServer) - ❌ **Not yet supported**

We already have full async geoprocessing infrastructure in `GeoprocessingServiceClient` with:
- Job submission (`submit_job`)
- Status polling (`get_job_status`, `poll_until_complete`)
- Result retrieval (`get_job_result`)
- Cancellation (`cancel_job`)
- Exponential backoff polling

**Solution**: Create elevation-specific async wrappers using existing GP infrastructure.

---

## Architecture Overview

### Current State (Broken)

```rust
// ❌ These call non-existent synchronous endpoints
elevation.summarize_elevation(params).await?  // 400 Invalid URL
elevation.viewshed(params).await?              // 400 Invalid URL
```

### Target State

```rust
// ✅ Async operations with polling
let job = elevation.submit_summarize_elevation(params).await?;
let result = elevation.poll_summarize_elevation(job.job_id()).await?;

// ✅ Or convenience method that handles polling internally
let result = elevation.summarize_elevation_async(params).await?;

// ✅ Manual control for advanced users
let job = elevation.submit_viewshed(params).await?;
// ... do other work ...
let status = elevation.get_job_status(job.job_id()).await?;
if status.job_status() == &GPJobStatus::Succeeded {
    let result = elevation.get_viewshed_result(job.job_id()).await?;
}
```

---

## Implementation Plan

### Phase 1: Research & Verify API Endpoints (1 commit)

**Verify actual API endpoints and response formats:**

1. **Test SummarizeElevation**:
   ```bash
   # Submit job
   curl -X POST "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/SummarizeElevation/submitJob" \
     -d "InputPolygon=<FeatureSet>" \
     -d "DEMResolution=30m" \
     -d "IncludeSlope=true" \
     -d "IncludeAspect=true" \
     -d "f=json" \
     -d "token=<key>"

   # Get status
   curl "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/SummarizeElevation/jobs/<jobId>?f=json"

   # Get results
   curl "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/SummarizeElevation/jobs/<jobId>/results/OutputSummary?f=json"
   ```

2. **Test Viewshed**:
   ```bash
   # Similar pattern for Viewshed
   curl -X POST ".../Viewshed/submitJob" ...
   ```

3. **Document findings**:
   - Parameter names (InputPolygon vs input_geometry)
   - Result parameter names (OutputSummary, OutputViewshed)
   - Response format (GP result wrapper vs direct)

**Files to update:**
- `ASYNC_ELEVATION_PLAN.md` - document verified API behavior
- Notes on differences from synchronous Profile operation

**Commit:** `docs(elevation): document async API endpoints and parameters`

---

### Phase 2: Add Async Submit Methods (1 commit)

**Add job submission methods to ElevationClient:**

```rust
// src/services/elevation/client.rs

impl<'a> ElevationClient<'a> {
    /// Submits an asynchronous SummarizeElevation job.
    ///
    /// Use this for calculating elevation statistics within polygons.
    /// After submission, poll for completion using `poll_summarize_elevation`
    /// or manage the job manually with GP service methods.
    ///
    /// # Example
    /// ```no_run
    /// let job = elevation.submit_summarize_elevation(params).await?;
    /// tracing::info!(job_id = %job.job_id(), "Job submitted");
    /// ```
    #[instrument(skip(self, params))]
    pub async fn submit_summarize_elevation(
        &self,
        params: SummarizeElevationParameters,
    ) -> Result<GPJobInfo> {
        tracing::debug!("Submitting SummarizeElevation job");

        // Create GP service client for async Elevation service
        let gp_service = GeoprocessingServiceClient::new(
            "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/SummarizeElevation",
            self.client,
        );

        // Convert params to HashMap
        let param_map = params_to_hashmap(&params)?;

        // Submit job
        let job = gp_service.submit_job(param_map).await?;

        tracing::info!(
            job_id = %job.job_id(),
            status = ?job.job_status(),
            "SummarizeElevation job submitted"
        );

        Ok(job)
    }

    /// Submits an asynchronous Viewshed job.
    #[instrument(skip(self, params))]
    pub async fn submit_viewshed(
        &self,
        params: ViewshedParameters,
    ) -> Result<GPJobInfo> {
        // Similar implementation
    }

    /// Helper to convert typed params to HashMap<String, Value>
    fn params_to_hashmap<T: Serialize>(params: &T) -> Result<HashMap<String, Value>> {
        let json_value = serde_json::to_value(params)?;
        let map = json_value
            .as_object()
            .ok_or_else(|| BuilderError::new("Failed to convert params to map"))?
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Ok(map)
    }
}
```

**Files to modify:**
- `src/services/elevation/client.rs` - add submit methods
- `src/services/elevation/mod.rs` - re-export GP types (GPJobInfo, GPJobStatus)

**Testing:**
```rust
let job = elevation.submit_summarize_elevation(params).await?;
assert!(job.job_id().len() > 0);
assert!(job.job_status().is_running());
```

**Commit:** `feat(elevation): add async job submission for SummarizeElevation and Viewshed`

---

### Phase 3: Add Polling & Result Retrieval (1 commit)

**Add convenience methods for polling and results:**

```rust
impl<'a> ElevationClient<'a> {
    /// Polls a SummarizeElevation job until completion and returns the result.
    ///
    /// This is a convenience wrapper around GP polling that extracts the
    /// typed SummarizeElevationResult from the job results.
    ///
    /// # Arguments
    /// * `job_id` - Job identifier from `submit_summarize_elevation`
    /// * `timeout_ms` - Optional timeout in milliseconds
    ///
    /// # Example
    /// ```no_run
    /// let job = elevation.submit_summarize_elevation(params).await?;
    /// let result = elevation.poll_summarize_elevation(job.job_id(), Some(60000)).await?;
    /// ```
    #[instrument(skip(self), fields(job_id, timeout_ms))]
    pub async fn poll_summarize_elevation(
        &self,
        job_id: &str,
        timeout_ms: Option<u64>,
    ) -> Result<SummarizeElevationResult> {
        tracing::info!("Polling SummarizeElevation job");

        let gp_service = GeoprocessingServiceClient::new(
            "https://elevation.arcgis.com/arcgis/rest/services/Tools/Elevation/GPServer/SummarizeElevation",
            self.client,
        );

        // Poll with exponential backoff
        let job_info = gp_service.poll_until_complete(
            job_id,
            1000,  // initial delay: 1s
            30000, // max delay: 30s
            timeout_ms,
        ).await?;

        // Check final status
        if job_info.job_status() != &GPJobStatus::Succeeded {
            tracing::error!(
                status = ?job_info.job_status(),
                messages = ?job_info.messages(),
                "Job did not succeed"
            );
            return Err(BuilderError::new(format!(
                "SummarizeElevation job failed with status: {:?}",
                job_info.job_status()
            )).into());
        }

        // Extract OutputSummary result
        let output_summary = job_info.results()
            .get("OutputSummary")
            .ok_or_else(|| {
                BuilderError::new("Job results missing OutputSummary parameter")
            })?;

        let value = output_summary.value()
            .as_ref()
            .ok_or_else(|| {
                BuilderError::new("OutputSummary parameter missing value")
            })?;

        // Parse FeatureSet
        let feature_set: FeatureSet = serde_json::from_value(value.clone())?;

        // Extract statistics from first feature attributes
        let stats = extract_elevation_stats(&feature_set)?;

        tracing::info!("SummarizeElevation job completed successfully");

        Ok(stats)
    }

    /// Polls a Viewshed job until completion and returns the result.
    #[instrument(skip(self), fields(job_id, timeout_ms))]
    pub async fn poll_viewshed(
        &self,
        job_id: &str,
        timeout_ms: Option<u64>,
    ) -> Result<ViewshedResult> {
        // Similar implementation
    }

    /// Convenience method: submit + poll in one call
    #[instrument(skip(self, params))]
    pub async fn summarize_elevation_async(
        &self,
        params: SummarizeElevationParameters,
        timeout_ms: Option<u64>,
    ) -> Result<SummarizeElevationResult> {
        let job = self.submit_summarize_elevation(params).await?;
        self.poll_summarize_elevation(job.job_id(), timeout_ms).await
    }

    /// Convenience method: submit + poll in one call
    #[instrument(skip(self, params))]
    pub async fn viewshed_async(
        &self,
        params: ViewshedParameters,
        timeout_ms: Option<u64>,
    ) -> Result<ViewshedResult> {
        let job = self.submit_viewshed(params).await?;
        self.poll_viewshed(job.job_id(), timeout_ms).await
    }
}

/// Helper to extract elevation statistics from FeatureSet
fn extract_elevation_stats(feature_set: &FeatureSet) -> Result<SummarizeElevationResult> {
    if feature_set.features().is_empty() {
        return Err(BuilderError::new("OutputSummary has no features").into());
    }

    let attrs = feature_set.features()[0].attributes();

    let min_elevation = attrs.get("MinElevation").and_then(|v| v.as_f64());
    let max_elevation = attrs.get("MaxElevation").and_then(|v| v.as_f64());
    let mean_elevation = attrs.get("MeanElevation").and_then(|v| v.as_f64());
    let area = attrs.get("Area").and_then(|v| v.as_f64());

    Ok(SummarizeElevationResult::new(
        feature_set.clone(),
        min_elevation,
        max_elevation,
        mean_elevation,
        area,
    ))
}
```

**Files to modify:**
- `src/services/elevation/client.rs` - add polling and convenience methods
- `src/services/elevation/types.rs` - add `new()` constructors to result types

**Testing:**
```rust
// Manual polling
let job = elevation.submit_summarize_elevation(params).await?;
let result = elevation.poll_summarize_elevation(job.job_id(), Some(60000)).await?;

// Convenience method
let result = elevation.summarize_elevation_async(params, Some(60000)).await?;
```

**Commit:** `feat(elevation): add polling and convenience methods for async operations`

---

### Phase 4: Update Parameters for Async Operations (1 commit)

**Fix parameter names and formats for async endpoints:**

Based on API verification, update:

```rust
// src/services/elevation/types.rs

#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct SummarizeElevationParameters {
    /// Input polygon features as FeatureSet JSON.
    #[serde(rename = "InputPolygon")]  // Verify actual parameter name
    input_polygon: String,

    /// DEM resolution (FINEST, 10m, 30m, 90m).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DEMResolution")]
    dem_resolution: Option<String>,

    /// Include slope statistics.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "IncludeSlope")]  // Verify casing
    include_slope: Option<bool>,

    /// Include aspect statistics.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "IncludeAspect")]  // Verify casing
    include_aspect: Option<bool>,

    // Remove geometry_type - it's part of FeatureSet
    // Remove spatial reference params if not supported
}

// Similar updates for ViewshedParameters
```

**Files to modify:**
- `src/services/elevation/types.rs` - fix parameter names based on API verification

**Testing:**
```rust
let params = SummarizeElevationParametersBuilder::default()
    .input_polygon(polygon_featureset)
    .dem_resolution("30m")
    .include_slope(true)
    .include_aspect(true)
    .build()?;
```

**Commit:** `fix(elevation): correct parameter names for async operations`

---

### Phase 5: Deprecate Broken Sync Methods (1 commit)

**Mark old methods as deprecated and update documentation:**

```rust
impl<'a> ElevationClient<'a> {
    /// ⚠️ **DEPRECATED**: Use `summarize_elevation_async` instead.
    ///
    /// SummarizeElevation is an asynchronous operation and cannot be called
    /// synchronously. This method will be removed in a future version.
    #[deprecated(
        since = "0.2.0",
        note = "Use summarize_elevation_async for async operation with polling"
    )]
    pub async fn summarize_elevation(
        &self,
        params: SummarizeElevationParameters,
    ) -> Result<SummarizeElevationResult> {
        // Redirect to async version with default timeout
        self.summarize_elevation_async(params, Some(300000)).await
    }

    /// ⚠️ **DEPRECATED**: Use `viewshed_async` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use viewshed_async for async operation with polling"
    )]
    pub async fn viewshed(
        &self,
        params: ViewshedParameters,
    ) -> Result<ViewshedResult> {
        self.viewshed_async(params, Some(300000)).await
    }
}
```

**Files to modify:**
- `src/services/elevation/client.rs` - add deprecation warnings
- Module-level docs explaining async vs sync operations

**Commit:** `refactor(elevation): deprecate broken sync methods in favor of async`

---

### Phase 6: Update Examples (1 commit)

**Update elevation_analysis example to demonstrate async operations:**

```rust
// examples/enterprise/elevation_analysis.rs

async fn demonstrate_terrain_statistics(elevation: &ElevationClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Terrain Statistics (Async) ===");
    tracing::info!("Calculate elevation statistics using asynchronous job");

    let valley_features = r#"{
        "geometryType": "esriGeometryPolygon",
        "features": [{
            "geometry": {
                "rings": [[[-119.60,37.80],[-119.50,37.80],[-119.50,37.90],[-119.60,37.90],[-119.60,37.80]]],
                "spatialReference": {"wkid": 4326}
            }
        }],
        "spatialReference": {"wkid": 4326}
    }"#;

    let params = SummarizeElevationParametersBuilder::default()
        .input_polygon(valley_features)
        .dem_resolution("30m")
        .include_slope(true)
        .include_aspect(true)
        .build()?;

    tracing::info!("   Submitting asynchronous job...");

    // Option 1: Manual control
    let job = elevation.submit_summarize_elevation(params.clone()).await?;
    tracing::info!("   Job submitted: {}", job.job_id());
    tracing::info!("   Polling for completion...");

    let result = elevation.poll_summarize_elevation(
        job.job_id(),
        Some(60000)  // 60 second timeout
    ).await?;

    // OR Option 2: Convenience method
    // let result = elevation.summarize_elevation_async(params, Some(60000)).await?;

    tracing::info!("✅ Terrain statistics calculated");

    if let Some(min) = result.min_elevation() {
        tracing::info!("   Minimum elevation: {:.1} meters", min);
    }
    // ... rest of output
}
```

**Files to modify:**
- `examples/enterprise/elevation_analysis.rs` - update SummarizeElevation and Viewshed demos

**Commit:** `docs(examples): update elevation_analysis for async operations`

---

### Phase 7: Add Helper Methods to Result Types (1 commit)

**Add convenience helpers similar to ProfileResult:**

```rust
impl SummarizeElevationResult {
    /// Gets the elevation range (max - min).
    pub fn elevation_range(&self) -> Option<f64> {
        match (self.min_elevation, self.max_elevation) {
            (Some(min), Some(max)) => Some(max - min),
            _ => None,
        }
    }

    /// Gets the area in square kilometers.
    pub fn area_km2(&self) -> Option<f64> {
        self.area.map(|a| a / 1_000_000.0)
    }
}

impl ViewshedResult {
    /// Gets the visibility percentage (0-100).
    pub fn visibility_percentage(&self) -> Option<f64> {
        match (self.visible_area, self.total_area) {
            (Some(visible), Some(total)) if total > 0.0 => {
                Some((visible / total) * 100.0)
            }
            _ => None,
        }
    }

    /// Gets visible area in square kilometers.
    pub fn visible_area_km2(&self) -> Option<f64> {
        self.visible_area.map(|a| a / 1_000_000.0)
    }
}
```

**Files to modify:**
- `src/services/elevation/types.rs` - add helper methods

**Commit:** `feat(elevation): add convenience methods to result types`

---

### Phase 8: Add Integration Tests (1 commit)

**Create async operation tests:**

```rust
// tests/elevation_async_test.rs

#[tokio::test]
#[cfg_attr(not(feature = "test-location"), ignore)]
async fn test_summarize_elevation_async() -> Result<()> {
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let elevation = ElevationClient::new(&client);

    let polygon = create_test_polygon();
    let params = SummarizeElevationParametersBuilder::default()
        .input_polygon(polygon)
        .dem_resolution("90m")  // Faster for tests
        .build()?;

    // Test submit
    let job = elevation.submit_summarize_elevation(params).await?;
    assert!(!job.job_id().is_empty());
    assert!(job.job_status().is_running() || job.job_status() == &GPJobStatus::Succeeded);

    // Test poll
    let result = elevation.poll_summarize_elevation(job.job_id(), Some(60000)).await?;
    assert!(result.min_elevation().is_some());
    assert!(result.max_elevation().is_some());

    Ok(())
}
```

**Files to create:**
- `tests/elevation_async_test.rs` - integration tests

**Commit:** `test(elevation): add integration tests for async operations`

---

### Phase 9: Documentation & Module Re-exports (1 commit)

**Update module documentation and exports:**

```rust
// src/services/elevation/mod.rs

//! Elevation Service.
//!
//! The Elevation Service provides terrain analysis operations:
//!
//! ## Synchronous Operations
//! - **Profile**: Generate elevation profiles along lines (immediate results)
//!
//! ## Asynchronous Operations
//! - **SummarizeElevation**: Calculate elevation statistics within polygons (job-based)
//! - **Viewshed**: Determine visible areas from observer points (job-based)
//!
//! # Async vs Sync
//!
//! Profile operations are synchronous and return results immediately.
//! SummarizeElevation and Viewshed are asynchronous geoprocessing jobs that require
//! polling. Use the `*_async` convenience methods or manage jobs manually:
//!
//! ```no_run
//! // Convenience method (recommended)
//! let result = elevation.summarize_elevation_async(params, Some(60000)).await?;
//!
//! // Manual job control (advanced)
//! let job = elevation.submit_summarize_elevation(params).await?;
//! let result = elevation.poll_summarize_elevation(job.job_id(), Some(60000)).await?;
//! ```

pub use client::ElevationClient;
pub use types::{
    DemResolution, ElevationPoint,
    ProfileParameters, ProfileParametersBuilder, ProfileResult,
    SummarizeElevationParameters, SummarizeElevationParametersBuilder, SummarizeElevationResult,
    ViewshedParameters, ViewshedParametersBuilder, ViewshedResult,
};

// Re-export GP types for job management
pub use crate::services::geoprocessing::{GPJobInfo, GPJobStatus};
```

**Files to modify:**
- `src/services/elevation/mod.rs` - update docs and exports
- `src/lib.rs` - export GP types at crate level

**Commit:** `docs(elevation): update module documentation for async operations`

---

## Success Criteria

- ✅ SummarizeElevation works via async job submission + polling
- ✅ Viewshed works via async job submission + polling
- ✅ Convenience methods (`*_async`) hide job management complexity
- ✅ Advanced users can manage jobs manually
- ✅ Proper error handling for job failures
- ✅ Observability: job ID, status transitions logged
- ✅ Integration tests pass
- ✅ Examples demonstrate both patterns
- ✅ Old sync methods deprecated with clear migration path
- ✅ All existing Profile tests still pass

---

## Testing Strategy

### Unit Tests
- Parameter serialization
- Result deserialization
- Helper method calculations

### Integration Tests (feature-gated)
- Job submission returns valid job ID
- Polling reaches terminal state
- Successful jobs return expected results
- Failed jobs return error information
- Timeout handling works correctly

### Manual Testing
Run elevation_analysis example:
```bash
RUST_LOG=debug cargo run --example elevation_analysis
```

Should show:
- Profile: immediate results (sync)
- SummarizeElevation: job submission → polling → results (async)
- Viewshed: job submission → polling → results (async)

---

## Migration Guide for Users

### Before (Broken)
```rust
// ❌ This fails with "Invalid URL"
let result = elevation.summarize_elevation(params).await?;
```

### After (Working)
```rust
// ✅ Option 1: Convenience method (recommended)
let result = elevation.summarize_elevation_async(params, Some(60000)).await?;

// ✅ Option 2: Manual job control
let job = elevation.submit_summarize_elevation(params).await?;
tracing::info!("Job ID: {}", job.job_id());
let result = elevation.poll_summarize_elevation(job.job_id(), Some(60000)).await?;

// ✅ Option 3: Advanced - check status periodically
let job = elevation.submit_summarize_elevation(params).await?;
loop {
    let status = elevation.get_job_status(job.job_id()).await?;
    if status.job_status().is_terminal() {
        break;
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
}
let result = elevation.get_summarize_elevation_result(job.job_id()).await?;
```

---

## Risks & Mitigation

**Risk 1**: API parameter names differ from documentation
- **Mitigation**: Verify all parameters with actual API calls before coding

**Risk 2**: Job polling timeout defaults may be too short
- **Mitigation**: Make timeout configurable, document recommended values

**Risk 3**: Breaking existing code that calls `summarize_elevation`
- **Mitigation**: Deprecate old methods but keep them working via redirect

**Risk 4**: Users don't understand async pattern
- **Mitigation**: Provide clear examples, comprehensive docs, convenience methods

---

## Timeline Estimate

- Phase 1 (Research): 1 hour
- Phase 2 (Submit methods): 1 hour
- Phase 3 (Polling): 2 hours
- Phase 4 (Parameters): 1 hour
- Phase 5 (Deprecation): 30 min
- Phase 6 (Examples): 1 hour
- Phase 7 (Helpers): 30 min
- Phase 8 (Tests): 1 hour
- Phase 9 (Docs): 30 min

**Total**: ~8.5 hours of focused work across 9 commits

---

## References

- [Summarize elevation API](https://developers.arcgis.com/rest/elevation-analysis/summarize-elevation/)
- [Viewshed API](https://developers.arcgis.com/rest/elevation-analysis/viewshed/)
- Existing `GeoprocessingServiceClient` implementation
- Profile operation (reference for patterns)
