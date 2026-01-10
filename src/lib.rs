//! # arcgis
//!
//! A type-safe Rust SDK for the [ArcGIS REST API](https://developers.arcgis.com/rest/).
//!
//! This library provides strongly-typed interfaces to ArcGIS services with compile-time
//! guarantees. Instead of error-prone string constants, it uses Rust enums and newtypes
//! to make invalid states unrepresentable.
//!
//! ## Features
//!
//! - 🔒 **Type-safe**: Enums instead of strings - compile-time validation
//! - 🌍 **GeoRust integration**: Native `geo-types` support
//! - 🔐 **Authentication**: API Key and OAuth 2.0
//! - ⚡ **Async/await**: Built on `tokio` and `reqwest`
//! - 🎯 **Modular**: Optional services via Cargo features
//!
//! ## Quick Start
//!
//! ```no_run
//! use arcgis::{ApiKeyAuth, ArcGISClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), arcgis::Error> {
//!     let auth = ApiKeyAuth::new("YOUR_API_KEY");
//!     let client = ArcGISClient::new(auth);
//!
//!     // Use the client to access ArcGIS services
//!     Ok(())
//! }
//! ```
//!
//! ## Type Safety
//!
//! This SDK enforces type safety throughout:
//!
//! ```rust
//! use arcgis::{GeometryType, SpatialRel};
//!
//! // ✅ Compile-time validated
//! let geom_type = GeometryType::Point;
//! let spatial_rel = SpatialRel::Intersects;
//!
//! // ❌ Won't compile
//! // let geom_type = "esriGeometryPoint";  // Wrong type!
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

// Re-export major dependencies for user convenience
pub use geo_types;
pub use geojson;

// Core modules
mod auth;
mod client;
mod error;
mod geometry;
mod services;
mod types;
mod util;

// Re-exports
pub use auth::{ApiKeyAuth, AuthProvider, ClientCredentialsAuth};
pub use client::ArcGISClient;
pub use error::{
    BuilderError, Error, ErrorKind, HttpError, IoError, JsonError, UrlEncodedError, UrlError,
};
pub use geometry::{
    ArcGISEnvelope, ArcGISGeometry, ArcGISMultipoint, ArcGISPoint, ArcGISPolygon, ArcGISPolyline,
};
pub use services::{
    AddAttachmentResult, AddressCandidate, AlterResponse, AlterVersionParams, AreaUnit,
    AreasAndLengthsParameters, AreasAndLengthsParametersBuilder, AreasAndLengthsResult,
    AttachmentInfo, AttachmentInfosResponse, AttachmentSource, BarrierType, BatchCandidateResult,
    BatchCandidatesResponse, BatchGeocodeResponse, BatchLocation, BufferParameters,
    BufferParametersBuilder, BufferResult, CalculationType, Category, ClassBreakInfo, CodedValue,
    ConflictDetection, ConflictEntry, ConflictFeature, ConflictsResponse, CreateVersionParams,
    CreateVersionResponse, DeleteAttachmentResult, DeleteAttachmentsResponse,
    DeleteForwardEditsResponse, DeleteResponse, DifferenceFeature, DifferenceResultType,
    DifferencesResponse, DirectionsLength, DirectionsStyle, DirectionsTimeAttribute,
    DistanceParameters, DistanceParametersBuilder, DistanceResult, Domain, DownloadResult,
    DownloadTarget, EditError, EditOptions, EditResult, EditResultItem, EditSessionError,
    ExportExtent, ExportMapBuilder, ExportMapParams, ExportMapParamsBuilder, ExportMapResponse,
    ExportResult, ExportTarget, Extent, Feature, FeatureQueryParams, FeatureQueryParamsBuilder,
    FeatureServiceClient, FeatureSet, FeatureStatisticsResponse, FieldCalculation, FindParams,
    FindParamsBuilder, FindResponse, FindResult, GenerateKmlParams, GenerateKmlParamsBuilder,
    GenerateRendererParams, GenerateRendererParamsBuilder, GeocodeAddress, GeocodeResponse,
    GeocodeServiceClient, GeoprocessingServiceClient, GeometryServiceClient, GPBoolean,
    GPDataFile, GPDate, GPDouble, GPExecuteResult, GPFeatureRecordSetLayer, GPJobInfo, GPJobStatus,
    GPLinearUnit, GPLong, GPMessage, GPMessageType, GPParameter, GPRasterDataLayer,
    GPResultParameter, GPString, IdentifyParams, IdentifyParamsBuilder,
    IdentifyResponse, IdentifyResult, ImageFormat, ImpedanceAttribute, InspectConflictFeature,
    InspectConflictLayer, InspectConflictsResponse, LayerConflicts, LayerDefinitions,
    LayerDomainInfo, LayerFeatureDifferences, LayerLegend, LayerObjectIdDifferences,
    LayerOperation, LayerSelection, LegendResponse, LegendSymbol, LevelOfDetail, LinearUnit,
    LocationType, MapServiceClient, MapServiceMetadata, NALocation, OutputLine, PartialPostRow,
    PostResponse, ProjectParameters, ProjectParametersBuilder, ProjectResult, QueryBuilder,
    QueryDomainsResponse, ReconcileResponse, RelatedRecordGroup, RelatedRecordsParams,
    RelatedRecordsParamsBuilder, RelatedRecordsResponse, RendererResponse, ResponseFormat,
    RestoreRowsLayer, RestoreRowsResponse, RestrictionAttribute, ReverseGeocodeResponse,
    RouteParameters, RouteParametersBuilder, RouteResult, RouteShape, RoutingServiceClient,
    ServiceAreaParameters, ServiceAreaParametersBuilder, ServiceAreaResult,
    ClosestFacilityParameters, ClosestFacilityParametersBuilder, ClosestFacilityResult,
    CurbApproach, ODCostMatrixParameters, ODCostMatrixParametersBuilder, ODCostMatrixResult,
    ServiceLayer, SessionId, SimplifyParameters, SimplifyParametersBuilder, SimplifyResult,
    SpatialReference, StartEditingResponse, StartReadingResponse, StatisticDefinition,
    StatisticType, StopEditingResponse, StopReadingResponse, Subtype, SuggestResponse, Suggestion,
    TileCoordinate, TileInfo, TimeRelation, TopFeaturesParams, TopFeaturesParamsBuilder, TopFilter,
    Transformation, TravelDirection, TravelMode, TruncateResult, UnionParameters,
    UnionParametersBuilder, UnionResult, UniqueValueInfo, UpdateAttachmentResult, UTurnPolicy,
    VectorTileServiceClient, VectorTileStyle, VersionGuid, VersionInfo, VersionInfosResponse,
    VersionManagementClient, VersionPermission, VersioningType, FontStack, GlyphRange,
};
pub use types::{AttachmentId, GeometryType, LayerId, ObjectId, SpatialRel};

/// Result type alias using this crate's [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;
