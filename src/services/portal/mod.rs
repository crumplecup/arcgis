//! Portal Service operations for ArcGIS Online and Portal for ArcGIS.
//!
//! This module provides access to portal functionality including:
//! - User management and information
//! - Content search and discovery
//! - Item operations (CRUD)
//! - Sharing and permissions
//! - Group management
//!
//! # Example
//!
//! ```no_run
//! use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, SearchParameters};
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//! let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &client);
//!
//! // Get current user
//! let user = portal.get_self().await?;
//! if let Some(name) = user.effective_username() {
//!     println!("User: {}", name);
//! }
//!
//! // Search for items
//! let results = portal
//!     .search(SearchParameters::new("type:\"Web Map\""))
//!     .await?;
//!
//! for item in results.results() {
//!     println!("{}: {}", item.title(), item.id());
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod service_definition;
mod types;

pub use client::PortalClient;
pub use service_definition::{
    CodedValue as DomainCodedValue, CodedValueCode, CodedValueDomain, CodedValueDomainBuilder,
    DrawingTool, EditFieldsInfo, EditFieldsInfoBuilder, EditorTrackingInfo, FeatureTemplate,
    FeatureTemplateBuilder, FieldDefinition, FieldDefinitionBuilder, FieldType,
    GeometryTypeDefinition, Index, IndexBuilder, LayerDefinition, LayerDefinitionBuilder,
    LayerRelationship, LayerRelationshipBuilder, MergePolicy, RangeDomain, RangeDomainBuilder,
    RelationshipCardinality, RelationshipRole, ServiceDefinition, ServiceDefinitionBuilder,
    SpatialReferenceDefinition, SplitPolicy, TableDefinition, TableDefinitionBuilder,
    TemplatePrototype, TemplatePrototypeBuilder,
};
pub use types::{
    AddItemParams, AddItemResult, CreateGroupParams, CreateServiceParams, CreateServiceResult,
    DeleteItemResult, DeleteServiceResult, GroupInfo, GroupMembership, GroupMembershipType,
    GroupResult, GroupSearchParameters, GroupSearchResult, ItemInfo, OverwriteParameters,
    OverwriteResult, PublishParameters, PublishResult, PublishStatus, SearchParameters,
    SearchResult, ShareItemResult, SharingParameters, SortOrder, UnshareItemResult,
    UpdateGroupParams, UpdateItemParams, UpdateItemResult, UpdateServiceDefinitionParams,
    UpdateServiceDefinitionResult, UserInfo,
};
