//! Feature Service operations.
//!
//! The Feature Service provides access to feature data, allowing you to query
//! and edit features with full CRUD support, including attachment management.

mod attachment;
mod client;
mod edit;
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
    Feature, FeatureQueryParams, FeatureQueryParamsBuilder, FeatureSet, ResponseFormat,
};
