//! ArcGIS service modules.

mod elevation;
mod feature;
mod geocode;
mod geometry;
mod geoprocessing;
mod image;
mod map;
mod places;
mod portal;
mod routing;
mod vector_tile;
mod version_management;

pub use elevation::{
    DemResolution, ElevationClient, ProfileParameters, ProfileParametersBuilder, ProfileResult,
    SummarizeElevationParameters, SummarizeElevationParametersBuilder, SummarizeElevationResult,
    ViewshedParameters, ViewshedParametersBuilder, ViewshedResult,
};
pub use feature::{
    AddAttachmentResult, AttachmentInfo, AttachmentInfosResponse, AttachmentSource, CodedValue,
    DeleteAttachmentResult, DeleteAttachmentsResponse, Domain, DownloadResult, DownloadTarget,
    EditError, EditOptions, EditResult, EditResultItem, Feature, FeatureQueryParams,
    FeatureQueryParamsBuilder, FeatureServiceClient, FeatureSet, FeatureStatisticsResponse,
    FieldCalculation, LayerDomainInfo, QueryBuilder, QueryDomainsResponse, RelatedRecordGroup,
    RelatedRecordsParams, RelatedRecordsParamsBuilder, RelatedRecordsResponse, ResponseFormat,
    StatisticDefinition, StatisticType, Subtype, TopFeaturesParams, TopFeaturesParamsBuilder,
    TopFilter, TruncateResult, UpdateAttachmentResult,
};
pub use geocode::{
    AddressCandidate, BatchCandidateResult, BatchCandidatesResponse, BatchGeocodeResponse,
    BatchLocation, Category, Extent, GeocodeAddress, GeocodeResponse, GeocodeServiceClient,
    LocationType, ReverseGeocodeResponse, SuggestResponse, Suggestion,
};
pub use geometry::{
    AreaUnit, AreasAndLengthsParameters, AreasAndLengthsParametersBuilder, AreasAndLengthsResult,
    BufferParameters, BufferParametersBuilder, BufferResult, CalculationType, DistanceParameters,
    DistanceParametersBuilder, DistanceResult, GeometryServiceClient, LinearUnit,
    ProjectParameters, ProjectParametersBuilder, ProjectResult, SimplifyParameters,
    SimplifyParametersBuilder, SimplifyResult, Transformation, UnionParameters,
    UnionParametersBuilder, UnionResult,
};
pub use geoprocessing::{
    GPBoolean, GPDataFile, GPDate, GPDouble, GPExecuteResult, GPFeatureRecordSetLayer, GPJobInfo,
    GPJobStatus, GPLinearUnit, GPLong, GPMessage, GPMessageType, GPParameter, GPRasterDataLayer,
    GPResultParameter, GPString, GeoprocessingServiceClient,
};
pub use image::{
    ExportImageParameters, ExportImageParametersBuilder, ExportImageResult, HistogramParameters,
    HistogramParametersBuilder, HistogramResult, IdentifyParameters, IdentifyParametersBuilder,
    ImageIdentifyResult, ImageServiceClient, InterpolationType, MosaicRule, PixelType, RasterInfo,
    RenderingRule, SampleParameters, SampleParametersBuilder, SampleResult,
};
pub use map::{
    ClassBreakInfo, ExportExtent, ExportMapBuilder, ExportMapParams, ExportMapParamsBuilder,
    ExportMapResponse, ExportResult, ExportTarget, FindParams, FindParamsBuilder, FindResponse,
    FindResult, GenerateKmlParams, GenerateKmlParamsBuilder, GenerateRendererParams,
    GenerateRendererParamsBuilder, IdentifyParams, IdentifyParamsBuilder, IdentifyResponse,
    IdentifyResult, ImageFormat, LayerDefinitions, LayerLegend, LayerOperation, LayerSelection,
    LegendResponse, LegendSymbol, LevelOfDetail, MapServiceClient, MapServiceMetadata,
    RendererResponse, ServiceLayer, SpatialReference, TileCoordinate, TileInfo, TimeRelation,
    UniqueValueInfo,
};
pub use places::{
    CategoriesResult, CategoryInfo, DayHours, PlaceAddress, PlaceCategory, PlaceContactInfo,
    PlaceDetailsResult, PlaceHours, PlaceInfo, PlaceRating, PlaceSearchParameters,
    PlaceSearchParametersBuilder, PlaceSearchResult, PlacesClient,
};
pub use portal::{
    GroupMembership, GroupMembershipType, ItemInfo, PortalClient, SearchParameters,
    SearchResult, SortOrder, UserInfo,
};
pub use routing::{
    BarrierType, ClosestFacilityParameters, ClosestFacilityParametersBuilder,
    ClosestFacilityResult, CurbApproach, DirectionsLength, DirectionsStyle,
    DirectionsTimeAttribute, ImpedanceAttribute, NALocation, ODCostMatrixParameters,
    ODCostMatrixParametersBuilder, ODCostMatrixResult, OutputLine, RestrictionAttribute,
    RouteParameters, RouteParametersBuilder, RouteResult, RouteShape, RoutingServiceClient,
    ServiceAreaParameters, ServiceAreaParametersBuilder, ServiceAreaResult, TravelDirection,
    TravelMode, UTurnPolicy,
};
pub use vector_tile::{FontStack, GlyphRange, VectorTileServiceClient, VectorTileStyle};
pub use version_management::{
    AlterResponse, AlterVersionParams, ConflictDetection, ConflictEntry, ConflictFeature,
    ConflictsResponse, CreateVersionParams, CreateVersionResponse, DeleteForwardEditsResponse,
    DeleteResponse, DifferenceFeature, DifferenceResultType, DifferencesResponse, EditSessionError,
    InspectConflictFeature, InspectConflictLayer, InspectConflictsResponse, LayerConflicts,
    LayerFeatureDifferences, LayerObjectIdDifferences, PartialPostRow, PostResponse,
    ReconcileResponse, RestoreRowsLayer, RestoreRowsResponse, SessionId, StartEditingResponse,
    StartReadingResponse, StopEditingResponse, StopReadingResponse, VersionGuid, VersionInfo,
    VersionInfosResponse, VersionManagementClient, VersionPermission, VersioningType,
};
