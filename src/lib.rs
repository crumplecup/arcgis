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
mod config;
mod error;
mod geometry;
mod services;
mod types;
mod util;

// Re-exports
pub use auth::{ApiKeyAuth, ApiKeyTier, AuthProvider, ClientCredentialsAuth, NoAuth};
pub use client::ArcGISClient;
pub use config::EnvConfig;
pub use error::{
    BuilderError, EnvError, Error, ErrorKind, HttpError, IoError, JsonError, UrlEncodedError,
    UrlError,
};
pub use geometry::{
    ArcGISEnvelope, ArcGISGeometry, ArcGISGeometryError, ArcGISGeometryErrorKind, ArcGISMultipoint,
    ArcGISPoint, ArcGISPolygon, ArcGISPolyline, GeoError, GeometryJsonError, GeometryType,
    SpatialReference, SpatialRel,
};
pub use services::{
    AddAttachmentResult, AddItemParams, AddItemResult, AddToDefinitionParams,
    AddToDefinitionResult, AddedLayerInfo, AddressCandidate, AlterResponse, AlterVersionParams,
    AreaUnit, AreasAndLengthsParameters, AreasAndLengthsParametersBuilder, AreasAndLengthsResult,
    AttachmentInfo, AttachmentInfosResponse, AttachmentSource, BarrierType, BatchGeocodeRecord,
    BatchGeocodeResponse, BatchLocation, BufferParameters, BufferParametersBuilder, BufferResult,
    CalculateResult, CalculationType, CategoriesResult, Category, CategoryInfo, ClassBreakInfo,
    ClosestFacilityParameters, ClosestFacilityParametersBuilder, ClosestFacilityResult, CodedValue,
    CodedValueCode, CodedValueDomain, CodedValueDomainBuilder, ConflictDetection, ConflictEntry,
    ConflictFeature, ConflictsResponse, CreateGroupParams, CreateServiceParams,
    CreateServiceResult, CreateVersionParams, CreateVersionResponse, CurbApproach, DayHours,
    DeleteAttachmentResult, DeleteAttachmentsResponse, DeleteForwardEditsResponse,
    DeleteItemResult, DeleteResponse, DeleteServiceResult, DemResolution, DifferenceFeature,
    DifferenceResultType, DifferencesResponse, DirectionsLength, DirectionsStyle,
    DirectionsTimeAttribute, DistanceParameters, DistanceParametersBuilder, DistanceResult, Domain,
    DomainCodedValue, DownloadResult, DownloadTarget, DrawingTool, EditError, EditFieldsInfo,
    EditFieldsInfoBuilder, EditOptions, EditResult, EditResultItem, EditSessionError,
    EditorTrackingInfo, ElevationClient, ElevationPoint, ExportExtent, ExportImageParameters,
    ExportImageParametersBuilder, ExportImageResult, ExportMapBuilder, ExportMapParams,
    ExportMapParamsBuilder, ExportMapResponse, ExportResult, ExportTarget, Extent, Feature,
    FeatureQueryParams, FeatureQueryParamsBuilder, FeatureServiceClient, FeatureSet,
    FeatureStatisticsResponse, FeatureTemplate, FeatureTemplateBuilder, FieldCalculation,
    FieldDefinition, FieldDefinitionBuilder, FieldType, FindParams, FindParamsBuilder,
    FindResponse, FindResult, FontStack, GPBoolean, GPDataFile, GPDate, GPDouble, GPExecuteResult,
    GPFeatureRecordSetLayer, GPJobInfo, GPJobStatus, GPLinearUnit, GPLong, GPMessage,
    GPMessageType, GPParameter, GPProgress, GPRasterDataLayer, GPResultParameter, GPString,
    GenerateKmlParams, GenerateKmlParamsBuilder, GenerateRendererParams,
    GenerateRendererParamsBuilder, GeocodeAddress, GeocodeResponse, GeocodeServiceClient,
    GeometryServiceClient, GeometryTypeDefinition, GeoprocessingServiceClient, GlyphRange,
    GroupInfo, GroupMembership, GroupMembershipType, GroupResult, GroupSearchParameters,
    GroupSearchResult, HistogramParameters, HistogramParametersBuilder, HistogramResult,
    IdentifyParameters, IdentifyParametersBuilder, IdentifyParams, IdentifyParamsBuilder,
    IdentifyResponse, IdentifyResult, ImageFormat, ImageIdentifyResult, ImageServiceClient,
    ImpedanceAttribute, Index, IndexBuilder, InspectConflictFeature, InspectConflictLayer,
    InspectConflictsResponse, InterpolationType, ItemDataUpload, ItemInfo, LayerConflicts, LayerDefinition,
    LayerDefinitionBuilder, LayerDefinitions, LayerDomainInfo, LayerFeatureDifferences,
    LayerLegend, LayerObjectIdDifferences, LayerOperation, LayerRelationship,
    LayerRelationshipBuilder, LayerSelection, LegendResponse, LegendSymbol, LevelOfDetail,
    LinearUnit, LocationType, MapServiceClient, MapServiceMetadata, MergePolicy, MosaicRule,
    NALocation, ODCostMatrixParameters, ODCostMatrixParametersBuilder, ODCostMatrixResult,
    OutputLine, OverwriteParameters, OverwriteResult, PartialPostRow, PixelType, PlaceAddress,
    PlaceCategory, PlaceContactInfo, PlaceDetailsResult, PlaceHours, PlaceInfo, PlaceRating,
    PlaceSearchParameters, PlaceSearchParametersBuilder, PlaceSearchResult, PlacesClient,
    PortalClient, PostResponse, ProfileParameters, ProfileParametersBuilder, ProfileResult,
    ProjectParameters, ProjectParametersBuilder, ProjectResult, PublishParameters, PublishResult,
    PublishStatus, QueryBuilder, QueryDomainsResponse, RangeDomain, RangeDomainBuilder, RasterInfo,
    ReconcileResponse, RelatedRecordGroup, RelatedRecordsParams, RelatedRecordsParamsBuilder,
    RelatedRecordsResponse, RelationshipCardinality, RelationshipClass, RelationshipRole,
    RelationshipRule, RelationshipsResponse, RendererResponse, RenderingRule, ResponseFormat,
    RestoreRowsLayer, RestoreRowsResponse, RestrictionAttribute, ReverseGeocodeResponse,
    RouteParameters, RouteParametersBuilder, RouteResult, RouteShape, RoutingServiceClient,
    SampleParameters, SampleParametersBuilder, SampleResult, SearchParameters, SearchResult,
    ServiceAreaParameters, ServiceAreaParametersBuilder, ServiceAreaResult, ServiceDefinition,
    ServiceDefinitionBuilder, ServiceDefinitionValidationError, ServiceLayer, SessionId,
    ShareItemResult, SharingParameters, SimplifyParameters, SimplifyParametersBuilder,
    SimplifyResult, SortOrder, SpatialReferenceDefinition, SplitPolicy, StartEditingResponse,
    StartReadingResponse, StatisticDefinition, StatisticType, StopEditingResponse,
    StopReadingResponse, Subtype, SuggestResponse, Suggestion, SummarizeElevationParameters,
    SummarizeElevationParametersBuilder, SummarizeElevationResult, TableDefinition,
    TableDefinitionBuilder, TemplatePrototype, TemplatePrototypeBuilder, TileCoordinate, TileInfo,
    TimeRelation, TopFeaturesParams, TopFeaturesParamsBuilder, TopFilter, Transformation,
    TravelDirection, TravelMode, TruncateResult, UTurnPolicy, UnionParameters,
    UnionParametersBuilder, UnionResult, UniqueValueInfo, UnshareItemResult,
    UpdateAttachmentResult, UpdateGroupParams, UpdateItemParams, UpdateItemResult,
    UpdateServiceDefinitionParams, UpdateServiceDefinitionResult, UserInfo,
    VectorTileServiceClient, VectorTileStyle, VersionGuid, VersionInfo, VersionInfosResponse,
    VersionManagementClient, VersionPermission, VersioningType, ViewshedParameters,
    ViewshedParametersBuilder, ViewshedResult,
};
pub use types::{AttachmentId, LayerId, ObjectId};
pub use util::check_esri_error;

/// Result type alias using this crate's [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;
