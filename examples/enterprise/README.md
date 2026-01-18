# Enterprise Examples

These examples use **Tier 2+ (Location/Portal/Publishing)** services that require authentication and may consume API credits from your ArcGIS subscription.

## Prerequisites

1. **ArcGIS Account:** Sign up at [developers.arcgis.com](https://developers.arcgis.com)
2. **API Key or OAuth Credentials:** Generate from your ArcGIS Dashboard
3. **Environment Setup:** Create a `.env` file in the project root:

```env
ARCGIS_API_KEY=your_api_key_here
# Or for OAuth:
CLIENT_ID=your_client_id
CLIENT_SECRET=your_client_secret
```

## Credit Usage

Enterprise users have access to a pool of credits associated with their subscription. The marginal cost of running these examples is typically **near zero** for testing purposes, as credits are shared across your organization's usage.

### Estimated Credits per Example

| Example | Tier | Est. Credits | Runtime | Notes |
|---------|------|--------------|---------|-------|
| `basic_client.rs` | N/A | 0 | <1s | Auth setup only, no API calls |
| `client_credentials_flow.rs` | N/A | 0 | <1s | OAuth flow demo, no service calls |
| `geocode_addresses.rs` | 2 (Location) | 0.04 | 5-10s | 10 geocodes × 0.004 credits each |
| `routing_navigation.rs` | 2 (Location) | 0.50 | 10-15s | Route + directions (~0.5 credits) |
| `geometry_operations.rs` | 2 (Location) | 0.10 | 5-10s | Buffer/union operations |
| `edit_session.rs` | 3 (Portal) | 0.01 | 5-10s | Feature edits (storage credits) |
| `feature_attachments.rs` | 3 (Portal) | 0.02 | 5-10s | Attachment uploads (storage credits) |
| `portal_content_management.rs` | 3 (Portal) | 0.01 | 10-15s | Portal searches and metadata |

**Total for all enterprise examples:** ~0.68 credits (~$0.07 at standard rates)

For enterprise users with credit pools, this cost is typically negligible compared to production workloads.

## Running Enterprise Examples

```bash
# Basic authentication setup
cargo run --example basic_client

# OAuth flow (requires CLIENT_ID/CLIENT_SECRET)
cargo run --example client_credentials_flow

# Location services (Tier 2)
cargo run --example geocode_addresses
cargo run --example routing_navigation
cargo run --example geometry_operations

# Portal operations (Tier 3)
cargo run --example edit_session
cargo run --example feature_attachments
cargo run --example portal_content_management
```

## Available Examples

### Authentication & Setup

#### basic_client.rs
**Tier:** N/A
**Credits:** 0
**Demonstrates:** Creating an ArcGIS client with API key authentication

#### client_credentials_flow.rs
**Tier:** N/A
**Credits:** 0
**Demonstrates:** OAuth 2.0 client credentials flow for service-to-service authentication

---

### Location Services (Tier 2)

#### geocode_addresses.rs
**Tier:** 2 (Location)
**Est. Credits:** 0.04
**Demonstrates:**
- Forward geocoding (address → coordinates)
- Batch geocoding
- Reverse geocoding (coordinates → address)
- Geocoding with custom parameters

#### routing_navigation.rs
**Tier:** 2 (Location)
**Est. Credits:** 0.50
**Demonstrates:**
- Route finding between waypoints
- Turn-by-turn directions
- Route optimization
- Travel time calculations

#### geometry_operations.rs
**Tier:** 2 (Location)
**Est. Credits:** 0.10
**Demonstrates:**
- Buffer operations
- Geometry unions
- Intersections
- Simplification
- Area/length calculations

---

### Portal Operations (Tier 3)

#### edit_session.rs
**Tier:** 3 (Portal/Publishing)
**Est. Credits:** 0.01
**Demonstrates:**
- Creating edit sessions
- Adding features
- Updating attributes
- Deleting features
- Transaction management

#### feature_attachments.rs
**Tier:** 3 (Portal/Publishing)
**Est. Credits:** 0.02
**Demonstrates:**
- Querying attachments
- Adding photo/document attachments
- Downloading attachments
- Updating attachments
- Deleting attachments
- Bulk operations

#### portal_content_management.rs
**Tier:** 3 (Portal)
**Est. Credits:** 0.01
**Demonstrates:**
- Searching portal content
- Querying item metadata
- Managing groups
- Pagination through results
- Portal item operations

---

## Credit Management

### For Individual Developers

Free tier includes:
- 2,000,000 basemap tiles/month
- 20,000 geocodes/month
- 5,000 routing requests/month
- 2 GB feature storage

Running all examples monthly would consume ~0.68 credits = well within free tier limits.

### For Enterprise Users

Enterprise subscriptions typically include:
- Pooled credits across organization
- Higher rate limits
- Premium services access
- Priority support

The marginal cost of running examples is negligible compared to production usage. Use these examples freely to learn and test your integrations.

## Rate Limiting

To avoid hitting rate limits, examples include automatic delays between requests. If you encounter rate limit errors:

1. Increase delays in example code
2. Use your own feature services (deployed to your portal)
3. Contact ArcGIS support for rate limit increases

## Next Steps

After running these examples:

1. **Explore the SDK:** Check out the [API documentation](https://docs.rs/arcgis)
2. **Build Your Application:** Use these examples as templates
3. **Deploy to Production:** Review [security best practices](../../docs/security.md)
4. **Monitor Usage:** Track credit consumption in your ArcGIS Dashboard

## Support

- **Documentation:** [developers.arcgis.com/documentation](https://developers.arcgis.com/documentation)
- **Community:** [community.esri.com](https://community.esri.com)
- **Issues:** [GitHub Issues](https://github.com/user/arcgis/issues)
