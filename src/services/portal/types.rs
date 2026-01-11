//! Portal service types for ArcGIS Online and Portal for ArcGIS.

use serde::{Deserialize, Serialize};

/// Information about a portal user.
///
/// Returned by the `getSelf` operation and other user-related queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    /// Unique username.
    username: String,

    /// User's full name.
    #[serde(default)]
    full_name: Option<String>,

    /// Email address.
    #[serde(default)]
    email: Option<String>,

    /// User role (e.g., "org_admin", "org_publisher", "org_user").
    #[serde(default)]
    role: Option<String>,

    /// Privileges assigned to the user.
    #[serde(default)]
    privileges: Vec<String>,

    /// Groups the user belongs to.
    #[serde(default)]
    groups: Vec<GroupMembership>,

    /// Storage quota in bytes.
    #[serde(default)]
    storage_quota: Option<i64>,

    /// Storage usage in bytes.
    #[serde(default)]
    storage_usage: Option<i64>,

    /// User's culture/locale (e.g., "en-US").
    #[serde(default)]
    culture: Option<String>,

    /// User's region.
    #[serde(default)]
    region: Option<String>,

    /// Thumbnail URL.
    #[serde(default)]
    thumbnail: Option<String>,

    /// User description.
    #[serde(default)]
    description: Option<String>,

    /// Tags associated with the user.
    #[serde(default)]
    tags: Vec<String>,

    /// Access level (private, org, public).
    #[serde(default)]
    access: Option<String>,

    /// User's preferred view (Web, GIS, null).
    #[serde(default)]
    preferred_view: Option<String>,

    /// Units preference (english, metric).
    #[serde(default)]
    units: Option<String>,

    /// User ID.
    #[serde(default)]
    id: Option<String>,

    /// Date created (milliseconds since epoch).
    #[serde(default)]
    created: Option<i64>,

    /// Date modified (milliseconds since epoch).
    #[serde(default)]
    modified: Option<i64>,

    /// Organization ID.
    #[serde(default)]
    org_id: Option<String>,
}

/// Group membership information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembership {
    /// Group ID.
    id: String,

    /// Group title.
    title: String,

    /// User's role in the group (owner, admin, member).
    #[serde(default)]
    user_membership: Option<GroupMembershipType>,

    /// Whether the group is invitation only.
    #[serde(default)]
    is_invitation_only: Option<bool>,
}

/// User's membership type in a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupMembershipType {
    /// Group owner.
    Owner,
    /// Group administrator.
    Admin,
    /// Regular member.
    Member,
}

/// Portal item information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct ItemInfo {
    /// Unique item ID.
    id: String,

    /// Item owner username.
    owner: String,

    /// Organization ID.
    #[serde(default)]
    org_id: Option<String>,

    /// Date created (milliseconds since epoch).
    created: i64,

    /// Date modified (milliseconds since epoch).
    modified: i64,

    /// Item name/title.
    title: String,

    /// Item type (e.g., "Feature Service", "Web Map", "Web Mapping Application").
    #[serde(rename = "type")]
    item_type: String,

    /// Type keywords.
    #[serde(default)]
    type_keywords: Vec<String>,

    /// Item description.
    #[serde(default)]
    description: Option<String>,

    /// Tags.
    #[serde(default)]
    tags: Vec<String>,

    /// Snippet (short summary).
    #[serde(default)]
    snippet: Option<String>,

    /// Thumbnail filename.
    #[serde(default)]
    thumbnail: Option<String>,

    /// Item extent (bounding box).
    #[serde(default)]
    extent: Option<Vec<Vec<f64>>>,

    /// Categories.
    #[serde(default)]
    categories: Vec<String>,

    /// Spatial reference.
    #[serde(default)]
    spatial_reference: Option<serde_json::Value>,

    /// Access level (private, shared, org, public).
    access: String,

    /// Number of comments.
    #[serde(default)]
    num_comments: Option<i32>,

    /// Number of ratings.
    #[serde(default)]
    num_ratings: Option<i32>,

    /// Average rating.
    #[serde(default)]
    avg_rating: Option<f64>,

    /// Number of views.
    #[serde(default)]
    num_views: Option<i64>,

    /// Item size in bytes.
    #[serde(default)]
    size: Option<i64>,

    /// Culture/locale.
    #[serde(default)]
    culture: Option<String>,

    /// Properties specific to item type.
    #[serde(default)]
    properties: Option<serde_json::Value>,

    /// Service URL (for service items).
    #[serde(default)]
    url: Option<String>,

    /// Groups the item is shared with.
    #[serde(default)]
    sharing_groups: Option<Vec<String>>,
}

/// Search parameters for portal items.
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct SearchParameters {
    /// Query string using Lucene syntax.
    ///
    /// Examples:
    /// - `"title:fire"` - Items with "fire" in title
    /// - `"type:\"Feature Service\""` - Feature services
    /// - `"owner:john AND tags:roads"` - Items owned by john tagged with roads
    query: String,

    /// Bounding box filter `[xmin, ymin, xmax, ymax]`.
    #[setters(skip)]
    bbox: Option<Vec<f64>>,

    /// Category filter (comma-separated).
    #[setters(skip)]
    categories: Option<String>,

    /// Field to sort by.
    #[setters(skip)]
    sort_field: Option<String>,

    /// Sort order (asc or desc).
    #[setters(skip)]
    sort_order: Option<SortOrder>,

    /// Starting index for pagination (default: 1).
    #[setters(skip)]
    start: Option<u32>,

    /// Number of results to return (default: 10, max: 100).
    #[setters(skip)]
    num: Option<u32>,
}

/// Sort order for search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortOrder {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

impl SearchParameters {
    /// Creates a new SearchParameters with the given query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    /// Sets the bounding box filter.
    ///
    /// Format: `[xmin, ymin, xmax, ymax]`
    pub fn with_bbox(mut self, bbox: Vec<f64>) -> Self {
        self.bbox = Some(bbox);
        self
    }

    /// Sets the category filter.
    pub fn with_categories(mut self, categories: impl Into<String>) -> Self {
        self.categories = Some(categories.into());
        self
    }

    /// Sets the sort field and order.
    pub fn with_sort(mut self, field: impl Into<String>, order: SortOrder) -> Self {
        self.sort_field = Some(field.into());
        self.sort_order = Some(order);
        self
    }

    /// Sets pagination parameters.
    pub fn with_pagination(mut self, start: u32, num: u32) -> Self {
        self.start = Some(start);
        self.num = Some(num);
        self
    }
}

/// Search result from portal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// Total number of results.
    total: i64,

    /// Starting index of this result set.
    start: i32,

    /// Number of results in this response.
    num: i32,

    /// Next start position for pagination.
    #[serde(default)]
    next_start: Option<i32>,

    /// Array of items.
    #[serde(default)]
    results: Vec<ItemInfo>,
}
