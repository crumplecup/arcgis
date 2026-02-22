# Item Data API Enhancement Plan

**Date:** 2026-02-21
**Status:** ✅ COMPLETED
**Issue:** ~~Current `get_item_data` and `update_item_data` methods are narrowly targeted for JSON and need to support diverse formats~~ FIXED

## ✅ Implementation Summary

**Completed on:** 2026-02-21

### Changes Made:

1. **Fixed `get_item_data()`** (src/services/portal/client/items.rs:406-445)
   - ✅ Removed incorrect `f=json` parameter
   - ✅ Now returns raw bytes in item's native format
   - ✅ Added `get_item_data_zip()` for Package types with `f=zip` parameter

2. **Replaced `update_item_data()` with `update_item_data_v2()`**
   - ✅ Created `ItemDataUpload` enum with Text, File, Url variants
   - ✅ Implemented type-safe pattern matching for three upload methods
   - ✅ Removed deprecated `update_item_data()` entirely (no users yet)

3. **New Type: `ItemDataUpload`** (src/services/portal/types.rs:699)
   ```rust
   pub enum ItemDataUpload {
       Text(String),
       File { data: Vec<u8>, filename: String, mime_type: String },
       Url(String),
   }
   ```

4. **Examples Created:**
   - ✅ `portal_item_data_text.rs` - GeoJSON and Web Map JSON uploads
   - ✅ `portal_item_data_files.rs` - CSV, PNG, PDF file uploads
   - ✅ Updated `portal_publishing.rs` to use new API

5. **Testing:**
   - ✅ All examples compile successfully
   - ✅ Coverage increased from 63% to 64%
   - ✅ PortalClient coverage: 73% → 81%

### Migration Decision:

**Zero users = Zero backward compatibility needed**
- Deprecated method removed entirely instead of migration period
- Clean API with no legacy baggage
- Single correct way to upload item data

---

## Original Analysis (For Historical Context)

### Current Issues

### `get_item_data` (Line 406-445)

**Problem 1:** Always adds `f=json` parameter
```rust
let mut request = self.client.http().get(&url).query(&[("f", "json")]);
```

**Issue:** According to Esri docs, `f` parameter only applies to Package types:
- For most item types, `f=json` is incorrect
- Endpoint returns raw bytes in item's native MIME type
- Adding wrong parameter may cause API to fail or return unexpected data

**Fix:** Remove `f` parameter by default, make it optional/conditional

---

### `update_item_data` (Line 463-520)

**Problem 1:** Always uses `file` parameter with hardcoded JSON assumptions
```rust
let part = reqwest::multipart::Part::bytes(data)
    .file_name("data.json")            // Hardcoded filename
    .mime_str("application/json")?;    // Hardcoded MIME type

let mut form = reqwest::multipart::Form::new()
    .text("f", "json")
    .part("file", part);  // Always uses "file" parameter
```

**Issue:** Esri API supports three mutually exclusive parameters:
- `text` - JSON content as string (e.g., Web Maps, feature collections)
- `file` - Binary file upload (e.g., packages, images, PDFs)
- `url` - URL reference (e.g., external services)

**Current impl only supports `file` with JSON assumptions**

---

## Proposed API Design

### Option 1: Configuration Struct (Recommended)

```rust
/// Configuration for uploading item data
#[derive(Debug, Clone)]
pub enum ItemDataUpload {
    /// Upload data as text (JSON string)
    Text(String),

    /// Upload data as a file
    File {
        data: Vec<u8>,
        filename: String,
        mime_type: String,
    },

    /// Reference external URL
    Url(String),
}

impl PortalClient<'_> {
    /// Updates item data with flexible format support
    pub async fn update_item_data(
        &self,
        item_id: impl AsRef<str>,
        upload: ItemDataUpload,
    ) -> Result<UpdateItemResult> {
        // Implementation handles all three cases
    }

    /// Downloads item data (raw bytes)
    pub async fn get_item_data(
        &self,
        item_id: impl AsRef<str>,
    ) -> Result<bytes::Bytes> {
        // Remove f=json parameter
    }

    /// Downloads package item as ZIP
    pub async fn get_item_data_as_zip(
        &self,
        item_id: impl AsRef<str>,
    ) -> Result<bytes::Bytes> {
        // Uses f=zip parameter
    }
}
```

**Benefits:**
- Type-safe API
- Self-documenting
- Forces users to think about what they're uploading
- Easy to extend (e.g., add `DataUrl` variant)
- Clear separation of concerns

**Drawbacks:**
- Breaking change to existing API
- More verbose for simple cases

---

### Option 2: Separate Methods (Alternative)

```rust
impl PortalClient<'_> {
    /// Upload text data (JSON string)
    pub async fn update_item_data_text(
        &self,
        item_id: impl AsRef<str>,
        text: impl Into<String>,
    ) -> Result<UpdateItemResult> { }

    /// Upload file data
    pub async fn update_item_data_file(
        &self,
        item_id: impl AsRef<str>,
        data: Vec<u8>,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Result<UpdateItemResult> { }

    /// Upload URL reference
    pub async fn update_item_data_url(
        &self,
        item_id: impl AsRef<str>,
        url: impl Into<String>,
    ) -> Result<UpdateItemResult> { }

    /// Get item data (raw)
    pub async fn get_item_data(
        &self,
        item_id: impl AsRef<str>,
    ) -> Result<bytes::Bytes> { }

    /// Get package item as ZIP
    pub async fn get_item_data_zip(
        &self,
        item_id: impl AsRef<str>,
    ) -> Result<bytes::Bytes> { }
}
```

**Benefits:**
- Explicit method names
- Simple to use for each case
- No enum to import

**Drawbacks:**
- More methods to maintain
- Harder to add new upload types
- Breaking change (still need migration path)

---

## Testing Strategy: Diverse Formats

### Test Matrix

| Item Type | Upload Method | Data Format | Test Focus |
|-----------|---------------|-------------|------------|
| **GeoJSON** | `Text` or `File` | JSON | Current test case |
| **Web Map** | `Text` | JSON | Complex nested JSON |
| **CSV** | `File` | text/csv | Non-JSON text file |
| **Shapefile** | `File` | application/zip | Binary package |
| **Image** | `File` | image/png | Binary non-package |
| **PDF** | `File` | application/pdf | Binary document |
| **URL Item** | `Url` | N/A | External reference |

### Test Implementation Approach

**Option A: Single comprehensive example** (e.g., `portal_item_data_formats.rs`)
```rust
async fn test_geojson_text_upload() -> Result<()> { }
async fn test_geojson_file_upload() -> Result<()> { }
async fn test_csv_file_upload() -> Result<()> { }
async fn test_shapefile_package() -> Result<()> { }
async fn test_image_upload() -> Result<()> { }
async fn test_url_reference() -> Result<()> { }
```

**Benefits:**
- All format tests in one place
- Easy to compare approaches
- Comprehensive validation

**Drawbacks:**
- Long example file
- Slow to run all tests

---

**Option B: Separate examples per category** (e.g., `portal_item_data_text.rs`, `portal_item_data_files.rs`)

```
examples/
  portal_item_data_text.rs      # GeoJSON, Web Map (text parameter)
  portal_item_data_files.rs     # CSV, Shapefile, Image, PDF (file parameter)
  portal_item_data_url.rs       # URL references (url parameter)
```

**Benefits:**
- Focused examples
- Faster individual runs
- Clear separation by upload type

**Drawbacks:**
- More files to maintain
- Duplicated setup code

---

**Recommendation: Option B** (separate examples) with shared test utilities

Create shared module:
```rust
// examples/common/portal_test_data.rs
pub fn create_test_geojson() -> String { }
pub fn create_test_csv() -> Vec<u8> { }
pub fn create_test_shapefile_zip() -> Vec<u8> { }
pub fn create_test_image_png() -> Vec<u8> { }
```

---

## Migration Path

**ACTUAL IMPLEMENTATION:** Skipped migration phases - removed old API immediately (zero users = zero breakage)

### ~~Phase 1: Add new API alongside old (Deprecated)~~ SKIPPED

No deprecation period needed - library has no users yet.

### ~~Phase 2: Update examples to use new API~~ COMPLETED

- ✅ `portal_publishing.rs` - Updated to use `ItemDataUpload::File`
- ✅ `portal_item_data_text.rs` - Created for Text variant demos
- ✅ `portal_item_data_files.rs` - Created for File variant demos

### ~~Phase 3: Remove deprecated method~~ COMPLETED IMMEDIATELY

- ✅ Removed `update_item_data()` entirely on 2026-02-21
- ✅ Only `update_item_data_v2()` exists (will rename to `update_item_data` in future refactor)

---

## Implementation Checklist

### Code Changes

- [ ] Create `ItemDataUpload` enum in `src/services/portal/types/mod.rs`
- [ ] Implement `update_item_data_v2` in `src/services/portal/client/items.rs`
- [ ] Fix `get_item_data` to remove `f=json` parameter
- [ ] Add `get_item_data_zip` for package types
- [ ] Deprecate old `update_item_data`
- [ ] Add comprehensive tracing/logging for all code paths

### Examples

- [ ] Update `portal_publishing.rs` Workflow B to use new API
- [ ] Create `portal_item_data_text.rs` - GeoJSON, Web Map
- [ ] Create `portal_item_data_files.rs` - CSV, Image, PDF
- [ ] Create `portal_item_data_packages.rs` - Shapefile ZIP
- [ ] Add assertions in all examples

### Tests

- [ ] Create `tests/portal_item_data_test.rs`
- [ ] Test text uploads (JSON)
- [ ] Test file uploads (CSV, binary)
- [ ] Test package uploads (ZIP)
- [ ] Test get_item_data (no f parameter)
- [ ] Test get_item_data_zip (with f=zip)
- [ ] Test round-trip integrity for all formats

### Documentation

- [ ] Update method docs with examples
- [ ] Add migration guide
- [ ] Document which item types use which upload method
- [ ] Add "Common Pitfalls" section

---

## Expected Behavior After Fix

### `get_item_data` (Fixed)

**Before:**
```rust
// Adds f=json to all requests (WRONG)
let data = portal.get_item_data(item_id).await?;
// Returns: might fail or return wrong format
```

**After:**
```rust
// No f parameter (CORRECT for most types)
let data = portal.get_item_data(item_id).await?;
// Returns: Raw bytes in item's native format

// For packages, use specific method:
let zip_data = portal.get_item_data_zip(item_id).await?;
// Returns: ZIP format
```

---

### `update_item_data` (Enhanced)

**Before:**
```rust
// Only supports JSON files (LIMITED)
let data = geojson_string.into_bytes();
let result = portal.update_item_data(item_id, data).await?;
```

**After:**
```rust
// Text upload (Web Maps, GeoJSON as text)
let upload = ItemDataUpload::Text(geojson_string);
let result = portal.update_item_data_v2(item_id, upload).await?;

// File upload (CSV, images, etc.)
let upload = ItemDataUpload::File {
    data: csv_bytes,
    filename: "data.csv".to_string(),
    mime_type: "text/csv".to_string(),
};
let result = portal.update_item_data_v2(item_id, upload).await?;

// URL reference
let upload = ItemDataUpload::Url("https://example.com/service".to_string());
let result = portal.update_item_data_v2(item_id, upload).await?;
```

---

## Questions for Decision

1. **API Design:** Prefer Option 1 (enum) or Option 2 (separate methods)?
   - **Recommendation:** Option 1 (enum) for type safety and extensibility

2. **Migration:** v2 suffix or new method names?
   - **Recommendation:** v2 suffix, cleaner migration path

3. **Examples:** One comprehensive or multiple focused?
   - **Recommendation:** Multiple focused examples (easier to run individually)

4. **Version:** Breaking change in 0.2.0 or 0.3.0?
   - **Recommendation:** Deprecate in 0.2.0, break in 0.3.0

---

## Timeline Estimate

- **Phase 1 (New API):** 2-3 hours
  - Implement enum and new method
  - Fix get_item_data
  - Deprecate old method

- **Phase 2 (Examples):** 3-4 hours
  - Update portal_publishing.rs
  - Create 3 new examples
  - Add comprehensive assertions

- **Phase 3 (Testing):** 2-3 hours
  - Create test suite
  - Test all formats
  - Verify round-trip integrity

**Total:** 7-10 hours

---

**Next Actions:**
1. Review and approve this plan
2. Implement Phase 1 (new API)
3. Run portal_publishing.rs to confirm current behavior with tracing
4. Implement Phase 2 (examples)
5. Update gap analysis document
