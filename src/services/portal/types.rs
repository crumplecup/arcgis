//! Portal service types for ArcGIS Online and Portal for ArcGIS.

use serde::{Deserialize, Serialize};

/// Information about a portal user.
///
/// Returned by the `getSelf` operation and other user-related queries.
/// When using OAuth2 client credentials, returns appInfo instead of user details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    /// Unique username (present with user authentication).
    #[serde(default)]
    username: Option<String>,

    /// Application info (present with OAuth2 client credentials).
    #[serde(default)]
    app_info: Option<AppInfo>,

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

/// Application information returned for OAuth2 client credentials.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// Application ID.
    app_id: String,

    /// Item ID of the application.
    item_id: String,

    /// Username of the application owner.
    app_owner: String,

    /// Organization ID.
    org_id: String,

    /// Application title.
    app_title: String,

    /// Privileges assigned to the application.
    #[serde(default)]
    privileges: Vec<String>,
}

impl UserInfo {
    /// Gets the effective username, checking both user and app contexts.
    ///
    /// With user authentication, returns the username field.
    /// With OAuth2 client credentials, returns appOwner from appInfo.
    pub fn effective_username(&self) -> Option<&str> {
        self.username
            .as_deref()
            .or_else(|| self.app_info.as_ref().map(|info| info.app_owner.as_str()))
    }
}

/// Group membership information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembership {
    /// Group ID.
    #[serde(default)]
    id: Option<String>,

    /// Group title.
    #[serde(default)]
    title: Option<String>,

    /// User's membership information in the group (can be string or object depending on API response).
    #[serde(default)]
    user_membership: Option<serde_json::Value>,

    /// Whether the group is invitation only.
    #[serde(default)]
    is_invitation_only: Option<bool>,
}

/// User's membership type in a group.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupMembershipType {
    /// Group owner.
    Owner,
    /// Group administrator.
    Admin,
    /// Regular member.
    Member,
    /// Unknown membership type (captures any other string value from API).
    #[serde(other)]
    Unknown,
}

/// User's membership information in a group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct UserMembershipInfo {
    /// Username of the member.
    username: String,

    /// Membership type (owner, admin, member).
    member_type: GroupMembershipType,

    /// Number of applications (purpose unclear in API docs).
    applications: i64,
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

/// Parameters for adding a new item to the portal.
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct AddItemParams {
    /// Item title (REQUIRED).
    title: String,

    /// Item type (REQUIRED).
    ///
    /// Examples: "Web Map", "Feature Service", "Web Mapping Application", "GeoJSON"
    #[setters(rename = "with_item_type")]
    item_type: String,

    /// Item description.
    #[setters(skip)]
    description: Option<String>,

    /// Tags (comma-separated or array).
    #[setters(skip)]
    tags: Option<Vec<String>>,

    /// Snippet (short summary).
    #[setters(skip)]
    snippet: Option<String>,

    /// Categories (paths like "/Categories/Imagery").
    #[setters(skip)]
    categories: Option<Vec<String>>,

    /// Type keywords.
    #[setters(skip)]
    type_keywords: Option<Vec<String>>,

    /// Item URL (for items referencing external resources).
    #[setters(skip)]
    url: Option<String>,

    /// Thumbnail file path (local file to upload).
    #[setters(skip)]
    thumbnail: Option<std::path::PathBuf>,

    /// Spatial reference WKID.
    #[setters(skip)]
    spatial_reference: Option<i32>,

    /// Extent as [[xmin, ymin], [xmax, ymax]].
    #[setters(skip)]
    extent: Option<Vec<Vec<f64>>>,

    /// Access level (private, shared, org, public).
    #[setters(skip)]
    access: Option<String>,

    /// Item properties (type-specific JSON).
    #[setters(skip)]
    properties: Option<serde_json::Value>,

    /// Folder ID where the item should be created.
    #[setters(skip)]
    folder: Option<String>,

    /// Text content for the item (for types like GeoJSON, CSV, etc.).
    #[setters(skip)]
    text: Option<String>,
}

impl AddItemParams {
    /// Creates a new AddItemParams with required fields.
    pub fn new(title: impl Into<String>, item_type: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            item_type: item_type.into(),
            ..Default::default()
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Sets the snippet.
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Sets categories.
    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.categories = Some(categories);
        self
    }

    /// Sets type keywords.
    pub fn with_type_keywords(mut self, keywords: Vec<String>) -> Self {
        self.type_keywords = Some(keywords);
        self
    }

    /// Sets the item URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the thumbnail file path.
    pub fn with_thumbnail(mut self, path: std::path::PathBuf) -> Self {
        self.thumbnail = Some(path);
        self
    }

    /// Sets the spatial reference.
    pub fn with_spatial_reference(mut self, wkid: i32) -> Self {
        self.spatial_reference = Some(wkid);
        self
    }

    /// Sets the extent.
    pub fn with_extent(mut self, extent: Vec<Vec<f64>>) -> Self {
        self.extent = Some(extent);
        self
    }

    /// Sets the access level.
    pub fn with_access(mut self, access: impl Into<String>) -> Self {
        self.access = Some(access.into());
        self
    }

    /// Sets item properties.
    pub fn with_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Sets the folder ID where the item should be created.
    pub fn with_folder(mut self, folder_id: impl Into<String>) -> Self {
        self.folder = Some(folder_id.into());
        self
    }

    /// Sets the text content for the item (for GeoJSON, CSV, etc.).
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
}

/// Parameters for updating an existing item.
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct UpdateItemParams {
    /// Updated title.
    #[setters(skip)]
    title: Option<String>,

    /// Updated description.
    #[setters(skip)]
    description: Option<String>,

    /// Updated tags.
    #[setters(skip)]
    tags: Option<Vec<String>>,

    /// Updated snippet.
    #[setters(skip)]
    snippet: Option<String>,

    /// Updated categories.
    #[setters(skip)]
    categories: Option<Vec<String>>,

    /// Updated type keywords.
    #[setters(skip)]
    type_keywords: Option<Vec<String>>,

    /// Updated URL.
    #[setters(skip)]
    url: Option<String>,

    /// Updated spatial reference.
    #[setters(skip)]
    spatial_reference: Option<i32>,

    /// Updated extent.
    #[setters(skip)]
    extent: Option<Vec<Vec<f64>>>,

    /// Updated access level.
    #[setters(skip)]
    access: Option<String>,
}

impl UpdateItemParams {
    /// Creates a new empty UpdateItemParams.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Sets the snippet.
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Sets categories.
    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.categories = Some(categories);
        self
    }

    /// Sets type keywords.
    pub fn with_type_keywords(mut self, keywords: Vec<String>) -> Self {
        self.type_keywords = Some(keywords);
        self
    }

    /// Sets the URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the spatial reference.
    pub fn with_spatial_reference(mut self, wkid: i32) -> Self {
        self.spatial_reference = Some(wkid);
        self
    }

    /// Sets the extent.
    pub fn with_extent(mut self, extent: Vec<Vec<f64>>) -> Self {
        self.extent = Some(extent);
        self
    }

    /// Sets the access level.
    pub fn with_access(mut self, access: impl Into<String>) -> Self {
        self.access = Some(access.into());
        self
    }
}

/// Result from adding an item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
pub struct AddItemResult {
    /// Whether the operation succeeded (defaults to true if not present).
    #[serde(default = "default_true")]
    success: bool,

    /// ID of the newly created item.
    id: String,

    /// Folder where the item was created.
    #[serde(default)]
    folder: Option<String>,
}

/// Helper function to provide default value of true for success field.
fn default_true() -> bool {
    true
}

/// Result from updating an item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
pub struct UpdateItemResult {
    /// Whether the operation succeeded (defaults to true if not present).
    #[serde(default = "default_true")]
    success: bool,

    /// ID of the updated item.
    #[serde(default)]
    id: Option<String>,
}

/// Result from deleting an item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct DeleteItemResult {
    /// Whether the operation succeeded.
    success: bool,

    /// ID of the deleted item.
    #[serde(default)]
    item_id: Option<String>,
}

/// Configuration for uploading item data.
///
/// Specifies how to upload data to a portal item. The three variants correspond to
/// the three mutually exclusive parameters supported by the ArcGIS REST API:
///
/// - `Text`: JSON content as a string (for Web Maps, GeoJSON text, etc.)
/// - `File`: Binary file upload with MIME type (for images, PDFs, CSVs, packages)
/// - `Url`: External URL reference (for remote services or resources)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemDataUpload {
    /// Upload data as JSON text content.
    ///
    /// Uses the `text` parameter in the REST API. Suitable for Web Maps,
    /// GeoJSON as text, and other JSON-based item types.
    Text(String),

    /// Upload data as a file with metadata.
    ///
    /// Uses the `file` parameter in the REST API. Suitable for binary files,
    /// images, PDFs, CSVs, shapefiles, and other file-based content.
    File {
        /// The raw file data as bytes.
        data: Vec<u8>,
        /// Filename (e.g., "data.csv", "map.png").
        filename: String,
        /// MIME type (e.g., "text/csv", "image/png", "application/json").
        mime_type: String,
    },

    /// Upload data as a URL reference.
    ///
    /// Uses the `url` parameter in the REST API. Suitable for referencing
    /// external services or web resources.
    Url(String),
}

/// Parameters for sharing an item.
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct SharingParameters {
    /// Share with everyone (make public).
    #[setters(skip)]
    everyone: Option<bool>,

    /// Share with organization.
    #[setters(skip)]
    org: Option<bool>,

    /// Group IDs to share with.
    #[setters(skip)]
    groups: Option<Vec<String>>,
}

impl SharingParameters {
    /// Creates a new empty SharingParameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to share with everyone (make public).
    pub fn with_everyone(mut self, everyone: bool) -> Self {
        self.everyone = Some(everyone);
        self
    }

    /// Sets whether to share with organization.
    pub fn with_org(mut self, org: bool) -> Self {
        self.org = Some(org);
        self
    }

    /// Sets group IDs to share with.
    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.groups = Some(groups);
        self
    }
}

/// Result from sharing an item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct ShareItemResult {
    /// Item ID that was shared.
    #[serde(default)]
    item_id: Option<String>,

    /// Not shared with (groups that failed).
    #[serde(default)]
    not_shared_with: Vec<String>,
}

impl ShareItemResult {
    /// Returns whether the operation succeeded (inferred from presence of item_id and empty not_shared_with).
    pub fn success(&self) -> bool {
        self.item_id.is_some() && self.not_shared_with.is_empty()
    }
}

/// Result from unsharing an item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct UnshareItemResult {
    /// Item ID that was unshared.
    #[serde(default)]
    item_id: Option<String>,

    /// Groups that the item was not unshared from.
    #[serde(default)]
    not_unshared_from: Vec<String>,
}

impl UnshareItemResult {
    /// Returns whether the operation succeeded (inferred from presence of item_id and empty not_unshared_from).
    pub fn success(&self) -> bool {
        self.item_id.is_some() && self.not_unshared_from.is_empty()
    }
}

/// Group information from portal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct GroupInfo {
    /// Unique group ID.
    id: String,

    /// Group title.
    title: String,

    /// Group description.
    #[serde(default)]
    description: Option<String>,

    /// Group snippet (short summary).
    #[serde(default)]
    snippet: Option<String>,

    /// Group owner username.
    owner: String,

    /// Tags associated with the group.
    #[serde(default)]
    tags: Vec<String>,

    /// Thumbnail URL.
    #[serde(default)]
    thumbnail: Option<String>,

    /// Date created (milliseconds since epoch).
    created: i64,

    /// Date modified (milliseconds since epoch).
    modified: i64,

    /// Access level (private, org, public).
    access: String,

    /// Whether group is invitation only.
    #[serde(default)]
    is_invitation_only: Option<bool>,

    /// Whether group is view only.
    #[serde(default)]
    is_view_only: Option<bool>,

    /// Sort field for group content.
    #[serde(default)]
    sort_field: Option<String>,

    /// Sort order for group content.
    #[serde(default)]
    sort_order: Option<String>,

    /// Protected status.
    #[serde(default)]
    protected: Option<bool>,

    /// Auto-join status.
    #[serde(default)]
    auto_join: Option<bool>,

    /// Current user's membership information in the group.
    #[serde(default)]
    user_membership: Option<UserMembershipInfo>,

    /// Provider group name (for federated groups).
    #[serde(default)]
    provider_group_name: Option<String>,
}

/// Parameters for searching groups.
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct GroupSearchParameters {
    /// Query string using Lucene syntax.
    query: String,

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

impl GroupSearchParameters {
    /// Creates a new GroupSearchParameters with the given query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
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

/// Search result for groups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct GroupSearchResult {
    /// Total number of results.
    total: i64,

    /// Starting index of this result set.
    start: i32,

    /// Number of results in this response.
    num: i32,

    /// Next start position for pagination.
    #[serde(default)]
    next_start: Option<i32>,

    /// Array of groups.
    #[serde(default)]
    results: Vec<GroupInfo>,
}

/// Parameters for creating a new group.
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct CreateGroupParams {
    /// Group title (REQUIRED).
    title: String,

    /// Group description.
    #[setters(skip)]
    description: Option<String>,

    /// Group snippet (short summary).
    #[setters(skip)]
    snippet: Option<String>,

    /// Tags.
    #[setters(skip)]
    tags: Option<Vec<String>>,

    /// Access level (private, org, public).
    #[setters(skip)]
    access: Option<String>,

    /// Whether group is invitation only.
    #[setters(skip)]
    is_invitation_only: Option<bool>,

    /// Whether group is view only.
    #[setters(skip)]
    is_view_only: Option<bool>,

    /// Auto-join setting.
    #[setters(skip)]
    auto_join: Option<bool>,

    /// Sort field for group content.
    #[setters(skip)]
    sort_field: Option<String>,

    /// Sort order for group content.
    #[setters(skip)]
    sort_order: Option<String>,
}

impl CreateGroupParams {
    /// Creates a new CreateGroupParams with required fields.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the snippet.
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Sets tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Sets the access level.
    pub fn with_access(mut self, access: impl Into<String>) -> Self {
        self.access = Some(access.into());
        self
    }

    /// Sets whether group is invitation only.
    pub fn with_invitation_only(mut self, invitation_only: bool) -> Self {
        self.is_invitation_only = Some(invitation_only);
        self
    }

    /// Sets whether group is view only.
    pub fn with_view_only(mut self, view_only: bool) -> Self {
        self.is_view_only = Some(view_only);
        self
    }

    /// Sets auto-join.
    pub fn with_auto_join(mut self, auto_join: bool) -> Self {
        self.auto_join = Some(auto_join);
        self
    }

    /// Sets sort field.
    pub fn with_sort_field(mut self, sort_field: impl Into<String>) -> Self {
        self.sort_field = Some(sort_field.into());
        self
    }

    /// Sets sort order.
    pub fn with_sort_order(mut self, sort_order: impl Into<String>) -> Self {
        self.sort_order = Some(sort_order.into());
        self
    }
}

/// Parameters for updating an existing group.
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct UpdateGroupParams {
    /// Updated title.
    #[setters(skip)]
    title: Option<String>,

    /// Updated description.
    #[setters(skip)]
    description: Option<String>,

    /// Updated snippet.
    #[setters(skip)]
    snippet: Option<String>,

    /// Updated tags.
    #[setters(skip)]
    tags: Option<Vec<String>>,

    /// Updated access level.
    #[setters(skip)]
    access: Option<String>,

    /// Updated invitation only setting.
    #[setters(skip)]
    is_invitation_only: Option<bool>,

    /// Updated view only setting.
    #[setters(skip)]
    is_view_only: Option<bool>,

    /// Updated sort field.
    #[setters(skip)]
    sort_field: Option<String>,

    /// Updated sort order.
    #[setters(skip)]
    sort_order: Option<String>,
}

impl UpdateGroupParams {
    /// Creates a new empty UpdateGroupParams.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the snippet.
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Sets tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Sets the access level.
    pub fn with_access(mut self, access: impl Into<String>) -> Self {
        self.access = Some(access.into());
        self
    }

    /// Sets invitation only.
    pub fn with_invitation_only(mut self, invitation_only: bool) -> Self {
        self.is_invitation_only = Some(invitation_only);
        self
    }

    /// Sets view only.
    pub fn with_view_only(mut self, view_only: bool) -> Self {
        self.is_view_only = Some(view_only);
        self
    }

    /// Sets sort field.
    pub fn with_sort_field(mut self, sort_field: impl Into<String>) -> Self {
        self.sort_field = Some(sort_field.into());
        self
    }

    /// Sets sort order.
    pub fn with_sort_order(mut self, sort_order: impl Into<String>) -> Self {
        self.sort_order = Some(sort_order.into());
        self
    }
}

/// Generic result for group operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct GroupResult {
    /// Whether the operation succeeded.
    success: bool,

    /// Group details (for create operations).
    #[serde(default)]
    group: Option<GroupSummary>,

    /// Group ID (for update/delete operations that return just success and groupId).
    #[serde(default)]
    group_id: Option<String>,
}

impl GroupResult {
    /// Gets the group ID from either the group object or the group_id field.
    pub fn id(&self) -> Option<&str> {
        if let Some(ref g) = self.group {
            Some(g.id())
        } else {
            self.group_id.as_deref()
        }
    }
}

/// Summary of group details returned in create operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
pub struct GroupSummary {
    /// Group ID.
    id: String,

    /// Group title.
    title: String,

    /// Group owner.
    owner: String,

    /// Access level.
    access: String,
}

// Custom Deserialize to handle both response formats
impl<'de> Deserialize<'de> for GroupResult {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Helper {
            success: bool,
            #[serde(default)]
            group: Option<GroupSummary>,
            #[serde(default)]
            group_id: Option<String>,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(GroupResult {
            success: helper.success,
            group: helper.group,
            group_id: helper.group_id,
        })
    }
}

/// Parameters for publishing a hosted service.
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct PublishParameters {
    /// Service name (REQUIRED).
    name: String,

    /// Service description.
    #[setters(skip)]
    description: Option<String>,

    /// Copyright text.
    #[setters(skip)]
    copyright_text: Option<String>,

    /// Whether the data is static (vs dynamic).
    #[setters(skip)]
    has_static_data: Option<bool>,

    /// Maximum number of records returned by queries.
    #[setters(skip)]
    max_record_count: Option<i32>,

    /// Service capabilities (e.g., "Query,Create,Update,Delete").
    #[setters(skip)]
    capabilities: Option<String>,

    /// Default spatial reference WKID.
    #[setters(skip)]
    spatial_reference: Option<i32>,

    /// Initial extent as [[xmin, ymin], [xmax, ymax]].
    #[setters(skip)]
    initial_extent: Option<Vec<Vec<f64>>>,

    /// Full extent as [[xmin, ymin], [xmax, ymax]].
    #[setters(skip)]
    full_extent: Option<Vec<Vec<f64>>>,

    /// Allow geometry updates.
    #[setters(skip)]
    allow_geometry_updates: Option<bool>,

    /// Enable versioning.
    #[setters(skip)]
    enable_versioning: Option<bool>,

    /// Units (e.g., "esriMeters").
    #[setters(skip)]
    units: Option<String>,

    /// XSS prevention enabled.
    #[setters(skip)]
    xss_prevention_enabled: Option<bool>,

    /// Overwrite existing service.
    #[setters(skip)]
    overwrite: Option<bool>,

    /// Build initial cache.
    #[setters(skip)]
    build_initial_cache: Option<bool>,
}

impl PublishParameters {
    /// Creates a new PublishParameters with required fields.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the copyright text.
    pub fn with_copyright_text(mut self, copyright_text: impl Into<String>) -> Self {
        self.copyright_text = Some(copyright_text.into());
        self
    }

    /// Sets whether data is static.
    pub fn with_has_static_data(mut self, has_static_data: bool) -> Self {
        self.has_static_data = Some(has_static_data);
        self
    }

    /// Sets the maximum record count.
    pub fn with_max_record_count(mut self, max_record_count: i32) -> Self {
        self.max_record_count = Some(max_record_count);
        self
    }

    /// Sets the service capabilities.
    pub fn with_capabilities(mut self, capabilities: impl Into<String>) -> Self {
        self.capabilities = Some(capabilities.into());
        self
    }

    /// Sets the spatial reference.
    pub fn with_spatial_reference(mut self, wkid: i32) -> Self {
        self.spatial_reference = Some(wkid);
        self
    }

    /// Sets the initial extent.
    pub fn with_initial_extent(mut self, extent: Vec<Vec<f64>>) -> Self {
        self.initial_extent = Some(extent);
        self
    }

    /// Sets the full extent.
    pub fn with_full_extent(mut self, extent: Vec<Vec<f64>>) -> Self {
        self.full_extent = Some(extent);
        self
    }

    /// Sets whether to allow geometry updates.
    pub fn with_allow_geometry_updates(mut self, allow: bool) -> Self {
        self.allow_geometry_updates = Some(allow);
        self
    }

    /// Sets whether to enable versioning.
    pub fn with_enable_versioning(mut self, enable: bool) -> Self {
        self.enable_versioning = Some(enable);
        self
    }

    /// Sets the units.
    pub fn with_units(mut self, units: impl Into<String>) -> Self {
        self.units = Some(units.into());
        self
    }

    /// Sets XSS prevention.
    pub fn with_xss_prevention_enabled(mut self, enabled: bool) -> Self {
        self.xss_prevention_enabled = Some(enabled);
        self
    }

    /// Sets whether to overwrite existing service.
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = Some(overwrite);
        self
    }

    /// Sets whether to build initial cache.
    pub fn with_build_initial_cache(mut self, build: bool) -> Self {
        self.build_initial_cache = Some(build);
        self
    }
}

/// Result from publishing a service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct PublishResult {
    /// Whether the operation succeeded (defaults to true if not present).
    #[serde(default = "default_true")]
    success: bool,

    /// Service item ID.
    #[serde(default)]
    service_item_id: Option<String>,

    /// Service URL.
    #[serde(default)]
    service_url: Option<String>,

    /// Job ID for tracking publish status.
    #[serde(default)]
    job_id: Option<String>,

    /// Error messages if any.
    #[serde(default)]
    error: Option<serde_json::Value>,
}

/// Status of a publishing job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct PublishStatus {
    /// Job ID.
    #[serde(default)]
    job_id: Option<String>,

    /// Job status (e.g., "esriJobSubmitted", "esriJobExecuting", "esriJobSucceeded", "esriJobFailed").
    #[serde(default)]
    job_status: Option<String>,

    /// Progress percentage (0-100).
    #[serde(default)]
    progress: Option<i32>,

    /// Status messages.
    #[serde(default)]
    messages: Vec<String>,

    /// Result parameter name.
    #[serde(default)]
    result_param_name: Option<String>,

    /// Result value.
    #[serde(default)]
    result_value: Option<serde_json::Value>,
}

/// Parameters for updating a service definition.
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct UpdateServiceDefinitionParams {
    /// Updated service definition (strongly-typed).
    #[setters(skip)]
    service_definition: Option<crate::ServiceDefinition>,

    /// Updated description.
    #[setters(skip)]
    description: Option<String>,

    /// Updated capabilities.
    #[setters(skip)]
    capabilities: Option<String>,

    /// Updated max record count.
    #[setters(skip)]
    max_record_count: Option<i32>,
}

impl UpdateServiceDefinitionParams {
    /// Creates a new empty UpdateServiceDefinitionParams.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the service definition using strongly-typed ServiceDefinition.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{UpdateServiceDefinitionParams, ServiceDefinitionBuilder};
    ///
    /// let params = UpdateServiceDefinitionParams::new()
    ///     .with_service_definition(
    ///         ServiceDefinitionBuilder::default()
    ///             .name("MyService")
    ///             .build()
    ///             .expect("Valid service definition")
    ///     );
    /// ```
    pub fn with_service_definition(mut self, definition: crate::ServiceDefinition) -> Self {
        self.service_definition = Some(definition);
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the capabilities.
    pub fn with_capabilities(mut self, capabilities: impl Into<String>) -> Self {
        self.capabilities = Some(capabilities.into());
        self
    }

    /// Sets the max record count.
    pub fn with_max_record_count(mut self, count: i32) -> Self {
        self.max_record_count = Some(count);
        self
    }
}

/// Result from updating a service definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServiceDefinitionResult {
    /// Whether the operation succeeded.
    success: bool,

    /// Service item ID.
    #[serde(default)]
    service_item_id: Option<String>,
}

/// Parameters for adding layers/tables to an existing hosted feature service.
///
/// Use this to add new layers or tables to a service created with [`create_service`](crate::PortalClient::create_service).
/// ESRI's createService creates an empty service container - you must call addToDefinition
/// to add the actual layer schemas before you can add features.
///
/// # Example
///
/// ```no_run
/// use arcgis::{
///     AddToDefinitionParams, FieldDefinitionBuilder, FieldType,
///     LayerDefinitionBuilder, GeometryTypeDefinition,
/// };
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Build a layer definition
/// let oid_field = FieldDefinitionBuilder::default()
///     .name("OBJECTID")
///     .field_type(FieldType::Oid)
///     .nullable(false)
///     .editable(false)
///     .build()?;
///
/// let layer = LayerDefinitionBuilder::default()
///     .id(0u32)
///     .name("Points")
///     .geometry_type(GeometryTypeDefinition::Point)
///     .fields(vec![oid_field])
///     .build()?;
///
/// // Add to an existing service
/// let params = AddToDefinitionParams::new()
///     .with_layers(vec![layer]);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct AddToDefinitionParams {
    /// Layers to add to the service.
    #[setters(skip)]
    layers: Option<Vec<crate::LayerDefinition>>,

    /// Tables to add to the service.
    #[setters(skip)]
    tables: Option<Vec<crate::TableDefinition>>,
}

impl AddToDefinitionParams {
    /// Creates a new empty AddToDefinitionParams.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the layers to add.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{AddToDefinitionParams, LayerDefinitionBuilder, GeometryTypeDefinition};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let layer = LayerDefinitionBuilder::default()
    ///     .id(0u32)
    ///     .name("Points")
    ///     .geometry_type(GeometryTypeDefinition::Point)
    ///     .build()?;
    ///
    /// let params = AddToDefinitionParams::new()
    ///     .with_layers(vec![layer]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_layers(mut self, layers: Vec<crate::LayerDefinition>) -> Self {
        self.layers = Some(layers);
        self
    }

    /// Sets the tables to add.
    pub fn with_tables(mut self, tables: Vec<crate::TableDefinition>) -> Self {
        self.tables = Some(tables);
        self
    }
}

/// Simple layer/table info returned from addToDefinition.
#[derive(Debug, Clone, Serialize, Deserialize, derive_getters::Getters)]
pub struct AddedLayerInfo {
    /// Layer/table name.
    name: String,

    /// Layer/table ID.
    id: u32,
}

/// Result from adding layers/tables to a service definition.
#[derive(Debug, Clone, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct AddToDefinitionResult {
    /// Whether the operation succeeded.
    success: bool,

    /// Layers that were added (if any).
    #[serde(default)]
    layers: Vec<AddedLayerInfo>,

    /// Tables that were added (if any).
    #[serde(default)]
    tables: Vec<AddedLayerInfo>,
}

/// Parameters for creating a new hosted feature service.
#[derive(Debug, Clone, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct CreateServiceParams {
    /// Service name.
    name: String,

    /// Service description.
    #[setters(skip)]
    description: Option<String>,

    /// Has static data (default: false for editable services).
    #[setters(skip)]
    has_static_data: Option<bool>,

    /// Max record count per query.
    #[setters(skip)]
    max_record_count: Option<i32>,

    /// Supported query formats (default: "JSON").
    #[setters(skip)]
    supported_query_formats: Option<String>,

    /// Capabilities (e.g., "Query,Create,Update,Delete,Editing").
    #[setters(skip)]
    capabilities: Option<String>,

    /// Service definition (strongly-typed, contains layers, tables, etc.).
    #[setters(skip)]
    service_definition: Option<crate::ServiceDefinition>,
}

impl CreateServiceParams {
    /// Creates a new CreateServiceParams with the service name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            has_static_data: None,
            max_record_count: None,
            supported_query_formats: None,
            capabilities: None,
            service_definition: None,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets whether the service has static data.
    pub fn with_has_static_data(mut self, has_static: bool) -> Self {
        self.has_static_data = Some(has_static);
        self
    }

    /// Sets the max record count.
    pub fn with_max_record_count(mut self, count: i32) -> Self {
        self.max_record_count = Some(count);
        self
    }

    /// Sets the supported query formats.
    pub fn with_supported_query_formats(mut self, formats: impl Into<String>) -> Self {
        self.supported_query_formats = Some(formats.into());
        self
    }

    /// Sets the capabilities.
    pub fn with_capabilities(mut self, capabilities: impl Into<String>) -> Self {
        self.capabilities = Some(capabilities.into());
        self
    }

    /// Sets the full service definition using strongly-typed ServiceDefinition.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     CreateServiceParams, ServiceDefinitionBuilder, LayerDefinitionBuilder,
    ///     FieldDefinitionBuilder, FieldType, GeometryTypeDefinition
    /// };
    ///
    /// let layer = LayerDefinitionBuilder::default()
    ///     .name("Points")
    ///     .geometry_type(GeometryTypeDefinition::Point)
    ///     .build()
    ///     .expect("Valid layer");
    ///
    /// let mut svc_builder = ServiceDefinitionBuilder::default();
    /// svc_builder.name("MyService");
    /// let service_def = svc_builder.add_layer(layer).build().expect("Valid service definition");
    ///
    /// let params = CreateServiceParams::new("MyService")
    ///     .with_service_definition(service_def);
    /// ```
    pub fn with_service_definition(mut self, definition: crate::ServiceDefinition) -> Self {
        self.service_definition = Some(definition);
        self
    }
}

/// Result from creating a service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct CreateServiceResult {
    /// Whether the operation succeeded (defaults to true if not present).
    #[serde(default = "default_true")]
    success: bool,

    /// Service item ID.
    #[serde(default)]
    service_item_id: Option<String>,

    /// Service URL.
    #[serde(default, alias = "encodedServiceURL")]
    service_url: Option<String>,

    /// Service name.
    #[serde(default)]
    name: Option<String>,

    /// Whether the service is a view.
    #[serde(default)]
    is_view: Option<bool>,
}

/// Result from deleting a service.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters, derive_new::new,
)]
#[serde(rename_all = "camelCase")]
pub struct DeleteServiceResult {
    /// Whether the operation succeeded.
    success: bool,

    /// Service item ID that was deleted.
    #[serde(default)]
    service_item_id: Option<String>,
}

/// Parameters for overwriting a service.
#[derive(Debug, Clone, Default, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct OverwriteParameters {
    /// Source item ID (containing the data to publish).
    source_item_id: String,

    /// Target service item ID to overwrite.
    target_service_id: String,

    /// Preserve item ID.
    #[setters(skip)]
    preserve_item_id: Option<bool>,
}

impl OverwriteParameters {
    /// Creates a new OverwriteParameters with required fields.
    pub fn new(source_item_id: impl Into<String>, target_service_id: impl Into<String>) -> Self {
        Self {
            source_item_id: source_item_id.into(),
            target_service_id: target_service_id.into(),
            preserve_item_id: None,
        }
    }

    /// Sets whether to preserve item ID.
    pub fn with_preserve_item_id(mut self, preserve: bool) -> Self {
        self.preserve_item_id = Some(preserve);
        self
    }
}

/// Result from overwriting a service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct OverwriteResult {
    /// Whether the operation succeeded.
    success: bool,

    /// Service item ID.
    #[serde(default)]
    service_item_id: Option<String>,
}
