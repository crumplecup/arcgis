# ArcGIS Rust SDK v0.1.3 Release

**Release Date:** February 28, 2026
**Status:** ⚠️ Active Development - Not Recommended for Production Use

---

## Overview

Version 0.1.3 represents a significant shift toward **transparency and systematic testing** of the ArcGIS Rust SDK. While we've achieved 97% example coverage (114/117 methods), this release is notable for what it *revealed* rather than what it celebrates.

**Key Achievement:** We now have honest, measurable assessment of SDK quality through systematic integration testing.

**Sobering Reality:** Approximately **one-third of operations are currently failing** in integration tests.

---

## What Changed

### 🔬 Testing Infrastructure (The Real Win)

We built a comprehensive testing and tracking system that functions as both documentation and integration tests:

**Example Execution Tracker**
- Automated tracking of all example runs with CSV logging
- Records: method coverage, success/failure status, execution time
- Enables systematic identification of broken operations
- Examples now fail loudly in CI when broken (instead of silently)

**Assertion Coverage**
- 100% of examples now include comprehensive assertions
- Examples verify API response structure and data integrity
- Positive and negative test cases where appropriate
- Total: 250+ assertions across 33 examples

### 📊 Coverage Achievement

**Methods Tested:** 114/117 (97%)
- **10 services at 100% coverage**: Geometry, Routing, Elevation, Image, Vector Tiles, Portal, Geoprocessing, Map, Geocoding, Version Management
- **1 service near-complete**: Feature Service (95%, 19/20 methods)
- **3 methods deferred**: PlacesClient (requires Location Platform subscription)

**New Examples Added** (8):
- `feature_service_batch_editing.rs` - Batch operations and global ID workflows
- `feature_service_truncate_safe.rs` - Safe truncation using version management
- `geoprocessing_execution_modes.rs` - Async job execution patterns
- `portal_group_membership.rs` - OAuth-based group operations
- `portal_group_workflow.rs` - Complete group lifecycle
- `portal_item_data_files.rs` - Diverse file format uploads
- `portal_item_data_text.rs` - Text content management
- `version_management_basics.rs` - Branch versioning workflows

### 🐛 Critical Bug Fixes

**Portal Publishing** (Major Fix)
- Fixed GeoJSON publish timeout (synchronous vs asynchronous handling)
- Added ESRI error checking to prevent confusing "missing field 'success'" errors
- Fixed PublishResult to handle both GeoJSON and .sd file response structures
- Added file_type parameter support (geojson, shapefile, csv, etc.)

**Type Safety Improvements**
- New `ProjectedPoint` trait for compile-time spatial reference validation
- Support for WGS84, Web Mercator, and State Plane coordinate systems
- Prevents runtime coordinate system confusion

**API Refinements**
- Removed redundant manual setters (leveraged derive_setters features)
- Replaced broken `update_item_data()` with type-safe `ItemDataUpload` enum
- Fixed geocoding reverse geocode to use ProjectedPoint trait

---

## The Hard Truth

### What Testing Revealed

Building comprehensive examples exposed significant issues:

**Broken Until Tested:**
- `simplify()`, `union()`, `areas_and_lengths()` - Serialization bugs
- `update_item_data()`, `get_item_data()` - Incorrect API workflow
- `identify_with_params` - Parameter serialization issues
- Multiple response types missing `#[serde(default)]` attributes

**Pattern Identified:** Untested methods are likely broken. Recent testing confirmed serialization/deserialization issues are common across untested operations.

**Current State:** While we have 97% *coverage*, approximately **one-third of operations fail in integration tests**. The examples exist and track coverage, but many don't pass yet.

### Why This Matters

Previous versions shipped with silent failures. Users would attempt operations that appeared to work in compilation but failed at runtime with confusing errors. This release:

1. **Makes failures visible** through CI integration
2. **Documents known issues** in tracked example runs
3. **Provides a roadmap** for fixes by identifying exactly what's broken

### Active Development Notice

⚠️ **This SDK is under active development and NOT recommended for production use.**

**Before Production Use:**
- Verify your specific use case has passing example coverage
- Test thoroughly in development environments
- Expect breaking changes in future releases
- Report issues through GitHub - we're actively fixing identified problems

---

## Looking Forward

### Progress is Honest Assessment

Each failing example we fix gets us closer to our goal: a reliable, production-ready ArcGIS SDK for Rust. Version 0.1.3 provides the infrastructure to measure that progress honestly.

**What We Gained:**
- Systematic testing infrastructure
- Clear visibility into what works and what doesn't
- Concrete roadmap for improvement (fix the 33% that's failing)
- CI integration that prevents regressions

**What's Next:**
- Fix identified failing operations (tracked in example_runs.csv)
- Improve error messages and API ergonomics
- Continue expanding coverage for untested operations
- Work toward production-ready status with confident test coverage

### How to Help

**Test Your Use Cases:**
- Run examples for operations you need
- Report failures with reproduction steps
- Contribute fixes for broken operations

**Share Feedback:**
- What operations are critical for your workflow?
- Which examples are most valuable?
- Where is documentation unclear?

---

## Migration Guide

### Breaking Changes

**ItemDataUpload API** (Portal)
```rust
// Before (v0.1.2)
portal.update_item_data(item_id, data.as_bytes()).await?;

// After (v0.1.3)
use arcgis::ItemDataUpload;

let upload = ItemDataUpload::File {
    data: data.as_bytes().to_vec(),
    filename: "data.json".to_string(),
    mime_type: "application/json".to_string(),
};
portal.update_item_data_v2(item_id, upload).await?;
```

**Geocoding with ProjectedPoint**
```rust
// Before (v0.1.2)
let point = ArcGISPoint::new(lon, lat);

// After (v0.1.3)
use arcgis::Wgs84Point;

let point = Wgs84Point::new(lon, lat);
let response = geocoder.reverse_geocode(&point).await?;
```

**PublishParameters** (derive_setters)
```rust
// Before (manual Option wrapping)
let params = PublishParameters::new("ServiceName")
    .with_description(Some("Description".to_string()));

// After (strip_option enabled)
let params = PublishParameters::new("ServiceName")
    .with_description("Description")
    .with_file_type("geojson");
```

---

## Installation

```toml
[dependencies]
arcgis = "0.1.3"
```

⚠️ **Remember:** Test thoroughly before production use. Check example coverage for your specific operations.

---

## Documentation

- [CHANGELOG.md](CHANGELOG.md) - Complete change history
- [Examples](examples/) - 33 comprehensive examples with assertions
- [Gap Analysis](docs/gap_analysis_testing_2026-02-14.md) - Detailed coverage tracking
- [CLAUDE.md](CLAUDE.md) - Development standards and practices

---

## Acknowledgments

This release prioritizes **honesty over hype**. We believe users deserve to know exactly what works and what doesn't. The testing infrastructure built in v0.1.3 makes that transparency possible.

Thank you to everyone testing, reporting issues, and contributing to making this SDK production-ready.

**Bottom Line:** We're not there yet, but we now have the tools to measure progress honestly and fix what's broken systematically.

---

## Statistics

- **Lines Changed:** 68 files, +7,377 insertions, -9,971 deletions
- **Documentation Cleaned:** 14 obsolete planning documents archived
- **Coverage:** 97% (114/117 testable methods)
- **Examples:** 33 total (8 new, 25 enhanced)
- **Assertions:** 250+ across all examples
- **Pre-Merge Checks:** All passing (clippy, fmt, doctests, audit, features)

---

**Version:** v0.1.3
**Git Tag:** `v0.1.3`
**Rust Version:** 1.83+
**License:** MIT OR Apache-2.0
