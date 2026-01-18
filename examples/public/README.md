# Public Examples

These examples use **Tier 1 (Public)** services that do not require authentication or consume API credits.

## Running Public Examples

Public examples can be run without any API key or credentials:

```bash
cargo run --example query_features
cargo run --example spatial_query
```

## Available Examples

### query_features.rs

Comprehensive Feature Service query operations using ESRI's public World Cities service.

**Demonstrates:**
- Basic WHERE clause queries
- Field filtering with out_fields
- Geometry inclusion control
- Pagination (manual and automatic)
- Count-only queries
- Object ID queries
- Alternative formats (GeoJSON, PBF)

**Credit Usage:** None (uses NoAuth with public service)

**Runtime:** ~5-10 seconds

---

### spatial_query.rs

Advanced spatial relationship queries demonstrating geographic filters.

**Demonstrates:**
- Bounding box queries (Envelope)
- Polygon queries (custom shapes)
- Combining spatial and attribute queries
- Different spatial relationships (Intersects, Contains, Within)
- Large area queries with auto-pagination

**Credit Usage:** None (uses NoAuth with public service)

**Runtime:** ~5-10 seconds

---

## Benefits of Public Examples

- **No API Key Required:** Run immediately without any setup
- **No Credit Cost:** Perfect for learning and testing
- **Fast Feedback:** See results instantly
- **Safe for CI/CD:** Can be included in automated testing without credential management

## Limitations

Public services may have:
- Rate limits (typically generous for learning/testing)
- Limited functionality (no editing, no portal operations)
- Public data only (World Cities, World Imagery basemaps, etc.)

For production applications or advanced features, see the [enterprise examples](../enterprise/).
