## v0.1.3 - Systematic Testing & Honest Assessment

⚠️ **Status: Active Development - Not Recommended for Production Use**

### Overview

Version 0.1.3 focuses on **transparency through systematic testing**. We built comprehensive integration testing infrastructure that revealed the true state of the SDK: while we have 97% example coverage, approximately **one-third of operations currently fail** in integration tests.

**Key Achievement:** Examples now function as integration tests with automated tracking and fail loudly in CI when broken.

---

### 🔬 Testing Infrastructure

**Example Execution Tracker**
- Automated CSV logging of all example runs
- Tracks method coverage, success/failure status, execution time
- Enables systematic identification of broken operations

**Assertion Coverage**
- 100% of examples include comprehensive assertions
- 250+ assertions across 33 examples
- Examples verify API response structure and data integrity

### 📊 Coverage

- **97% of methods tested** (114/117)
- **10 services at 100% coverage**: Geometry, Routing, Elevation, Image, Vector Tiles, Portal, Geoprocessing, Map, Geocoding, Version Management
- **8 new examples** demonstrating batch operations, version management, portal workflows
- **33 total examples** with full assertion coverage

### 🐛 Critical Fixes

**Portal Publishing**
- Fixed GeoJSON publish timeout (synchronous vs asynchronous handling)
- Added ESRI error checking to prevent confusing error messages
- Fixed PublishResult to handle different response structures
- Added file_type parameter support (geojson, shapefile, csv)

**Type Safety**
- New `ProjectedPoint` trait for compile-time spatial reference validation
- Support for WGS84, Web Mercator, State Plane coordinates
- Prevents runtime coordinate system confusion

---

### ⚠️ Active Development Notice

**What Testing Revealed:**
- Pattern identified: untested methods are likely broken
- Common issues: serialization bugs, incorrect API workflows, missing serde attributes
- ~33% of operations fail in integration tests despite 97% coverage

**Before Use:**
- ✅ Verify your use case has passing example coverage
- ✅ Test thoroughly in development environments
- ✅ Expect breaking changes in future releases
- ✅ Report issues - we're actively fixing identified problems

---

### 📈 What This Means

**Progress Through Honesty:**

Previous versions shipped with silent failures. This release makes failures visible through CI integration and provides a clear roadmap for improvement.

**We're Not There Yet:** But we now have the infrastructure to measure progress honestly and fix what's broken systematically.

---

### 🔄 Breaking Changes

**ItemDataUpload API**
```rust
// v0.1.2
portal.update_item_data(item_id, data.as_bytes()).await?;

// v0.1.3
let upload = ItemDataUpload::File {
    data: data.as_bytes().to_vec(),
    filename: "data.json".to_string(),
    mime_type: "application/json".to_string(),
};
portal.update_item_data_v2(item_id, upload).await?;
```

**Geocoding with ProjectedPoint**
```rust
// v0.1.2
let point = ArcGISPoint::new(lon, lat);

// v0.1.3
use arcgis::Wgs84Point;
let point = Wgs84Point::new(lon, lat);
let response = geocoder.reverse_geocode(&point).await?;
```

See [Migration Guide](RELEASE-0.1.3.md#migration-guide) for complete details.

---

### 📚 Documentation

- [Complete Release Notes](RELEASE-0.1.3.md) - Full details
- [CHANGELOG.md](CHANGELOG.md) - Complete change history
- [Examples](examples/) - 33 comprehensive examples
- [Gap Analysis](docs/gap_analysis_testing_2026-02-14.md) - Coverage tracking

---

### 🙏 How to Help

- Test your use cases and report failures
- Share which operations are critical for your workflow
- Contribute fixes for broken operations

**Bottom Line:** Honest assessment over hype. We're building the infrastructure for a production-ready SDK, but we're not there yet.

---

**Full Changelog**: https://github.com/crumplecup/arcgis/compare/v0.1.2...v0.1.3
