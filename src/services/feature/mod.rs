//! Feature Service operations.
//!
//! The Feature Service provides access to feature data, allowing you to query
//! and edit features with full CRUD support, including attachment management.

mod attachment;
mod client;
mod edit;
mod geojson;
pub mod pbf;
mod query;
mod types;

pub use attachment::{
    AddAttachmentResult, AttachmentInfo, AttachmentInfosResponse, AttachmentSource,
    DeleteAttachmentResult, DeleteAttachmentsResponse, DownloadResult, DownloadTarget,
    UpdateAttachmentResult,
};
pub use client::FeatureServiceClient;
pub use edit::{EditError, EditOptions, EditResult, EditResultItem};
pub use query::QueryBuilder;
pub use types::{
    CodedValue, Domain, Feature, FeatureQueryParams, FeatureQueryParamsBuilder, FeatureSet,
    FeatureStatisticsResponse, FieldCalculation, LayerDomainInfo, QueryDomainsResponse,
    RelatedRecordGroup, RelatedRecordsParams, RelatedRecordsParamsBuilder, RelatedRecordsResponse,
    RelationshipClass, RelationshipRule, RelationshipsResponse, ResponseFormat,
    StatisticDefinition, StatisticType, Subtype, TopFeaturesParams, TopFeaturesParamsBuilder,
    TopFilter, TruncateResult,
};
