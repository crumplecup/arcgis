//! Types for routing and network analysis operations.

use crate::ArcGISGeometry;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// A location for network analysis operations.
///
/// Can represent a stop, facility, incident, or barrier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Getters)]
#[serde(rename_all = "camelCase")]
pub struct NALocation {
    /// Geometry of the location (typically a point).
    geometry: ArcGISGeometry,

    /// Optional name for the location.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// Optional curb approach.
    #[serde(skip_serializing_if = "Option::is_none")]
    curb_approach: Option<CurbApproach>,

    /// Optional bearing angle (0-360 degrees).
    #[serde(skip_serializing_if = "Option::is_none")]
    bearing: Option<f64>,

    /// Optional bearing tolerance (0-180 degrees).
    #[serde(skip_serializing_if = "Option::is_none")]
    bearing_tolerance: Option<f64>,

    /// Optional navigation latency (seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    nav_latency: Option<f64>,
}

impl NALocation {
    /// Creates a new NALocation from a geometry.
    pub fn new(geometry: ArcGISGeometry) -> Self {
        Self {
            geometry,
            name: None,
            curb_approach: None,
            bearing: None,
            bearing_tolerance: None,
            nav_latency: None,
        }
    }

    /// Sets the name of the location.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the curb approach.
    pub fn with_curb_approach(mut self, approach: CurbApproach) -> Self {
        self.curb_approach = Some(approach);
        self
    }
}

/// Curb approach for navigating to a location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurbApproach {
    /// Either side of the vehicle.
    #[serde(rename = "esriNAEitherSideOfVehicle")]
    EitherSide = 0,
    /// Right side of the vehicle.
    #[serde(rename = "esriNARightSideOfVehicle")]
    RightSide = 1,
    /// Left side of the vehicle.
    #[serde(rename = "esriNALeftSideOfVehicle")]
    LeftSide = 2,
    /// No U-turn.
    #[serde(rename = "esriNANoUTurn")]
    NoUTurn = 3,
}

/// Travel mode for routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TravelMode {
    /// Driving time.
    DrivingTime,
    /// Driving distance.
    DrivingDistance,
    /// Trucking time.
    TruckingTime,
    /// Trucking distance.
    TruckingDistance,
    /// Walking time.
    WalkingTime,
    /// Walking distance.
    WalkingDistance,
    /// Rural driving time.
    RuralDrivingTime,
    /// Rural driving distance.
    RuralDrivingDistance,
}

/// Impedance attribute for cost calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImpedanceAttribute {
    /// Travel time (minutes).
    #[serde(rename = "TravelTime")]
    TravelTime,
    /// Miles.
    #[serde(rename = "Miles")]
    Miles,
    /// Kilometers.
    #[serde(rename = "Kilometers")]
    Kilometers,
    /// Time at one kilometer per hour.
    #[serde(rename = "TimeAt1KPH")]
    TimeAt1KPH,
    /// Walk time.
    #[serde(rename = "WalkTime")]
    WalkTime,
    /// Truck travel time.
    #[serde(rename = "TruckTravelTime")]
    TruckTravelTime,
}

/// Restriction attribute for routing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestrictionAttribute {
    /// Avoid toll roads.
    #[serde(rename = "Avoid Toll Roads")]
    AvoidTollRoads,
    /// Avoid highways.
    #[serde(rename = "Avoid Limited Access Roads")]
    AvoidHighways,
    /// Avoid unpaved roads.
    #[serde(rename = "Avoid Unpaved Roads")]
    AvoidUnpavedRoads,
    /// Avoid ferries.
    #[serde(rename = "Avoid Ferries")]
    AvoidFerries,
    /// Avoid gates.
    #[serde(rename = "Avoid Gates")]
    AvoidGates,
    /// One way restriction.
    #[serde(rename = "Oneway")]
    Oneway,
    /// Height restriction.
    #[serde(rename = "Height Restriction")]
    HeightRestriction,
    /// Weight restriction.
    #[serde(rename = "Weight Restriction")]
    WeightRestriction,
    /// Weight per axle restriction.
    #[serde(rename = "Weight per Axle Restriction")]
    WeightPerAxleRestriction,
    /// Length restriction.
    #[serde(rename = "Length Restriction")]
    LengthRestriction,
    /// Width restriction.
    #[serde(rename = "Width Restriction")]
    WidthRestriction,
}

/// U-turn policy at junctions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UTurnPolicy {
    /// Allow U-turns anywhere.
    #[serde(rename = "esriNFSBAllowBacktrack")]
    AllowBacktrack,
    /// Allow U-turns at dead ends only.
    #[serde(rename = "esriNFSBAtDeadEndsOnly")]
    AtDeadEndsOnly,
    /// Allow U-turns at dead ends and intersections.
    #[serde(rename = "esriNFSBAtDeadEndsAndIntersections")]
    AtDeadEndsAndIntersections,
    /// No U-turns.
    #[serde(rename = "esriNFSBNoBacktrack")]
    NoBacktrack,
}

/// Output line type for routes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputLine {
    /// No line output (coordinates only).
    #[serde(rename = "esriNAOutputLineNone")]
    None,
    /// Straight lines between stops.
    #[serde(rename = "esriNAOutputLineStraight")]
    Straight,
    /// True shape with measures.
    #[serde(rename = "esriNAOutputLineTrueShapeWithMeasure")]
    TrueShapeWithMeasure,
}

/// Travel direction for analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TravelDirection {
    /// From facility/origin.
    #[serde(rename = "esriNATravelDirectionFromFacility")]
    FromFacility,
    /// To facility/destination.
    #[serde(rename = "esriNATravelDirectionToFacility")]
    ToFacility,
}

/// Type of barrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BarrierType {
    /// Point barrier.
    Point,
    /// Line barrier.
    Line,
    /// Polygon barrier.
    Polygon,
}

/// Directions length units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DirectionsLength {
    /// Miles.
    #[serde(rename = "esriNAMiles")]
    Miles,
    /// Kilometers.
    #[serde(rename = "esriNAKilometers")]
    Kilometers,
    /// Meters.
    #[serde(rename = "esriNAMeters")]
    Meters,
    /// Feet.
    #[serde(rename = "esriNAFeet")]
    Feet,
    /// Yards.
    #[serde(rename = "esriNAYards")]
    Yards,
    /// Nautical miles.
    #[serde(rename = "esriNANauticalMiles")]
    NauticalMiles,
}

/// Directions time attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DirectionsTimeAttribute {
    /// Travel time.
    #[serde(rename = "TravelTime")]
    TravelTime,
    /// Walk time.
    #[serde(rename = "WalkTime")]
    WalkTime,
    /// Truck travel time.
    #[serde(rename = "TruckTravelTime")]
    TruckTravelTime,
}

/// Directions style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DirectionsStyle {
    /// Complete directions.
    #[serde(rename = "esriDMTStandard")]
    Standard,
    /// Directions suitable for printing.
    #[serde(rename = "esriDMTPrint")]
    Print,
    /// Directions for desktop applications.
    #[serde(rename = "esriDMTDesktop")]
    Desktop,
    /// Directions for navigation devices.
    #[serde(rename = "esriDMTNavigation")]
    Navigation,
}

/// Shape type for route geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RouteShape {
    /// No shape.
    #[serde(rename = "none")]
    None,
    /// Straight line.
    #[serde(rename = "straight")]
    Straight,
    /// True route shape.
    #[serde(rename = "true")]
    True,
    /// True shape with measures.
    #[serde(rename = "trueShapeWithMeasures")]
    TrueShapeWithMeasures,
}

/// Parameters for route calculation.
///
/// Use [`RouteParameters::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct RouteParameters {
    /// Stops for the route (REQUIRED).
    /// Minimum 2 stops required.
    stops: Vec<NALocation>,

    /// Point barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    barriers: Option<Vec<NALocation>>,

    /// Polyline barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    polyline_barriers: Option<Vec<ArcGISGeometry>>,

    /// Polygon barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    polygon_barriers: Option<Vec<ArcGISGeometry>>,

    /// Whether to return directions.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_directions: Option<bool>,

    /// Whether to return routes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_routes: Option<bool>,

    /// Whether to return stops.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_stops: Option<bool>,

    /// Whether to return barriers.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_barriers: Option<bool>,

    /// Output spatial reference WKID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    out_sr: Option<i32>,

    /// Impedance attribute for cost.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    impedance_attribute: Option<ImpedanceAttribute>,

    /// Restriction attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    restriction_attribute_names: Option<Vec<RestrictionAttribute>>,

    /// Attribute parameter values (JSON object).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    attribute_parameter_values: Option<serde_json::Value>,

    /// Whether to use hierarchy in solving.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    use_hierarchy: Option<bool>,

    /// Time of day for traffic-aware routing (epoch milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    start_time: Option<i64>,

    /// U-turn policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    uturn_policy: Option<UTurnPolicy>,

    /// Directions length units.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    directions_length_units: Option<DirectionsLength>,

    /// Directions time attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    directions_time_attribute: Option<DirectionsTimeAttribute>,

    /// Directions style.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    directions_style: Option<DirectionsStyle>,

    /// Directions language (e.g., "en", "es", "fr").
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    directions_language: Option<String>,

    /// Whether to preserve first stop.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    preserve_first_stop: Option<bool>,

    /// Whether to preserve last stop.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    preserve_last_stop: Option<bool>,

    /// Whether to find best sequence.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    find_best_sequence: Option<bool>,

    /// Whether to return to start.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_to_start: Option<bool>,

    /// Whether to use time windows.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    use_time_windows: Option<bool>,

    /// Accumulate attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    accumulate_attribute_names: Option<Vec<String>>,

    /// Output line type.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    output_lines: Option<OutputLine>,

    /// Travel mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    travel_mode: Option<TravelMode>,
}

impl RouteParameters {
    /// Creates a builder for RouteParameters.
    pub fn builder() -> RouteParametersBuilder {
        RouteParametersBuilder::default()
    }
}

/// Result from route calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct RouteResult {
    /// Calculated routes.
    #[serde(default)]
    routes: Vec<Route>,

    /// Stops with arrival/departure times.
    #[serde(default)]
    stops: Vec<Stop>,

    /// Barriers that were used.
    #[serde(default)]
    barriers: Vec<NALocation>,

    /// Messages from the solve operation.
    #[serde(default)]
    messages: Vec<NAMessage>,
}

/// A calculated route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    /// Route name.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// Total length.
    #[serde(skip_serializing_if = "Option::is_none")]
    total_length: Option<f64>,

    /// Total time (minutes).
    #[serde(skip_serializing_if = "Option::is_none")]
    total_time: Option<f64>,

    /// Total drive time (minutes).
    #[serde(skip_serializing_if = "Option::is_none")]
    total_drive_time: Option<f64>,

    /// Total wait time (minutes).
    #[serde(skip_serializing_if = "Option::is_none")]
    total_wait_time: Option<f64>,

    /// Route geometry (polyline).
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry: Option<ArcGISGeometry>,

    /// Turn-by-turn directions.
    #[serde(default)]
    directions: Vec<Direction>,

    /// Start time (epoch milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    start_time: Option<i64>,

    /// End time (epoch milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    end_time: Option<i64>,
}

/// A stop on a route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    /// Stop name.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// Stop geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry: Option<ArcGISGeometry>,

    /// Arrival time (epoch milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    arrival_time: Option<i64>,

    /// Departure time (epoch milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    departure_time: Option<i64>,

    /// Wait time (minutes).
    #[serde(skip_serializing_if = "Option::is_none")]
    wait_time: Option<f64>,

    /// Cumulative length to this stop.
    #[serde(skip_serializing_if = "Option::is_none")]
    cumulative_length: Option<f64>,

    /// Sequence number in optimized route.
    #[serde(skip_serializing_if = "Option::is_none")]
    sequence: Option<i32>,
}

/// A direction instruction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Direction {
    /// Instruction text.
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,

    /// Length of this segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    length: Option<f64>,

    /// Time for this segment (minutes).
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<f64>,

    /// Direction geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry: Option<ArcGISGeometry>,

    /// Maneuver type.
    #[serde(skip_serializing_if = "Option::is_none")]
    maneuver_type: Option<String>,
}

/// Network Analyst message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct NAMessage {
    /// Message type (0=informative, 1=warning, 2=error).
    #[serde(rename = "type")]
    message_type: i32,

    /// Message description.
    description: String,
}

/// Parameters for service area calculation.
///
/// Use [`ServiceAreaParameters::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct ServiceAreaParameters {
    /// Facilities (starting points) for service areas (REQUIRED).
    facilities: Vec<NALocation>,

    /// Break values (time or distance) for service areas (REQUIRED).
    /// For example: [5, 10, 15] for 5, 10, and 15 minute service areas.
    default_breaks: Vec<f64>,

    /// Point barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    barriers: Option<Vec<NALocation>>,

    /// Polyline barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    polyline_barriers: Option<Vec<ArcGISGeometry>>,

    /// Polygon barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    polygon_barriers: Option<Vec<ArcGISGeometry>>,

    /// Travel direction (from or to facilities).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    travel_direction: Option<TravelDirection>,

    /// Output spatial reference WKID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    out_sr: Option<i32>,

    /// Impedance attribute for cost.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    impedance_attribute: Option<ImpedanceAttribute>,

    /// Whether to merge similar polygons.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    merge_similar_polygon_ranges: Option<bool>,

    /// Whether to split polygons at breaks.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    split_polygons_at_breaks: Option<bool>,

    /// Whether to trim outer polygon.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    trim_outer_polygon: Option<bool>,

    /// Trim distance (in units matching impedance).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    trim_polygon_distance: Option<f64>,

    /// Time of day for traffic-aware analysis (epoch milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    time_of_day: Option<i64>,

    /// Whether to use hierarchy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    use_hierarchy: Option<bool>,

    /// U-turn policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    uturn_policy: Option<UTurnPolicy>,

    /// Restriction attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    restriction_attribute_names: Option<Vec<RestrictionAttribute>>,

    /// Travel mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    travel_mode: Option<TravelMode>,

    /// Whether to return facilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_facilities: Option<bool>,

    /// Whether to return barriers.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_barriers: Option<bool>,
}

impl ServiceAreaParameters {
    /// Creates a builder for ServiceAreaParameters.
    pub fn builder() -> ServiceAreaParametersBuilder {
        ServiceAreaParametersBuilder::default()
    }
}

/// Result from service area calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ServiceAreaResult {
    /// Service area polygons.
    #[serde(default)]
    service_area_polygons: Vec<ServiceAreaPolygon>,

    /// Service area polylines (network edges).
    #[serde(default)]
    service_area_polylines: Vec<ServiceAreaPolyline>,

    /// Facilities that were used.
    #[serde(default)]
    facilities: Vec<NALocation>,

    /// Messages from the solve operation.
    #[serde(default)]
    messages: Vec<NAMessage>,
}

/// A service area polygon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ServiceAreaPolygon {
    /// Facility ID this area belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    facility_id: Option<i32>,

    /// Break value (time or distance).
    #[serde(skip_serializing_if = "Option::is_none")]
    from_break: Option<f64>,

    /// End break value.
    #[serde(skip_serializing_if = "Option::is_none")]
    to_break: Option<f64>,

    /// Polygon geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry: Option<ArcGISGeometry>,
}

/// A service area polyline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ServiceAreaPolyline {
    /// Facility ID this line belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    facility_id: Option<i32>,

    /// Break value (time or distance).
    #[serde(skip_serializing_if = "Option::is_none")]
    from_break: Option<f64>,

    /// End break value.
    #[serde(skip_serializing_if = "Option::is_none")]
    to_break: Option<f64>,

    /// Polyline geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry: Option<ArcGISGeometry>,
}

/// Parameters for closest facility calculation.
///
/// Use [`ClosestFacilityParameters::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct ClosestFacilityParameters {
    /// Incidents (demand points) to analyze (REQUIRED).
    incidents: Vec<NALocation>,

    /// Facilities (supply points) to analyze (REQUIRED).
    facilities: Vec<NALocation>,

    /// Number of closest facilities to find per incident.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    default_target_facility_count: Option<i32>,

    /// Point barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    barriers: Option<Vec<NALocation>>,

    /// Polyline barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    polyline_barriers: Option<Vec<ArcGISGeometry>>,

    /// Polygon barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    polygon_barriers: Option<Vec<ArcGISGeometry>>,

    /// Travel direction (from incidents or to facilities).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    travel_direction: Option<TravelDirection>,

    /// Output spatial reference WKID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    out_sr: Option<i32>,

    /// Impedance attribute for cost.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    impedance_attribute: Option<ImpedanceAttribute>,

    /// Accumulate attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    accumulate_attribute_names: Option<Vec<String>>,

    /// Whether to return directions.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_directions: Option<bool>,

    /// Whether to return routes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_routes: Option<bool>,

    /// Whether to return facilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_facilities: Option<bool>,

    /// Whether to return incidents.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_incidents: Option<bool>,

    /// Whether to return barriers.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_barriers: Option<bool>,

    /// Output line type.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    output_lines: Option<OutputLine>,

    /// Time of day for traffic-aware routing (epoch milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    time_of_day: Option<i64>,

    /// Whether to use hierarchy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    use_hierarchy: Option<bool>,

    /// U-turn policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    uturn_policy: Option<UTurnPolicy>,

    /// Restriction attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    restriction_attribute_names: Option<Vec<RestrictionAttribute>>,

    /// Travel mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    travel_mode: Option<TravelMode>,
}

impl ClosestFacilityParameters {
    /// Creates a builder for ClosestFacilityParameters.
    pub fn builder() -> ClosestFacilityParametersBuilder {
        ClosestFacilityParametersBuilder::default()
    }
}

/// Result from closest facility calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ClosestFacilityResult {
    /// Routes from incidents to facilities.
    #[serde(default)]
    routes: Vec<Route>,

    /// Facilities that were analyzed.
    #[serde(default)]
    facilities: Vec<NALocation>,

    /// Incidents that were analyzed.
    #[serde(default)]
    incidents: Vec<NALocation>,

    /// Messages from the solve operation.
    #[serde(default)]
    messages: Vec<NAMessage>,
}

/// Parameters for origin-destination cost matrix calculation.
///
/// Use [`ODCostMatrixParameters::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct ODCostMatrixParameters {
    /// Origins for the matrix (REQUIRED).
    origins: Vec<NALocation>,

    /// Destinations for the matrix (REQUIRED).
    destinations: Vec<NALocation>,

    /// Point barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    barriers: Option<Vec<NALocation>>,

    /// Polyline barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    polyline_barriers: Option<Vec<ArcGISGeometry>>,

    /// Polygon barriers to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    polygon_barriers: Option<Vec<ArcGISGeometry>>,

    /// Travel direction (origins to destinations or reverse).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    travel_direction: Option<TravelDirection>,

    /// Output spatial reference WKID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    out_sr: Option<i32>,

    /// Impedance attribute for cost.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    impedance_attribute: Option<ImpedanceAttribute>,

    /// Accumulate attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    accumulate_attribute_names: Option<Vec<String>>,

    /// Time of day for traffic-aware analysis (epoch milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    time_of_day: Option<i64>,

    /// Whether to use hierarchy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    use_hierarchy: Option<bool>,

    /// U-turn policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    uturn_policy: Option<UTurnPolicy>,

    /// Restriction attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    restriction_attribute_names: Option<Vec<RestrictionAttribute>>,

    /// Travel mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    travel_mode: Option<TravelMode>,
}

impl ODCostMatrixParameters {
    /// Creates a builder for ODCostMatrixParameters.
    pub fn builder() -> ODCostMatrixParametersBuilder {
        ODCostMatrixParametersBuilder::default()
    }
}

/// Result from origin-destination cost matrix calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ODCostMatrixResult {
    /// Origin-destination cost matrix lines.
    #[serde(default, rename = "odLines")]
    od_lines: Vec<ODLine>,

    /// Origins that were analyzed.
    #[serde(default)]
    origins: Vec<NALocation>,

    /// Destinations that were analyzed.
    #[serde(default)]
    destinations: Vec<NALocation>,

    /// Messages from the solve operation.
    #[serde(default)]
    messages: Vec<NAMessage>,
}

/// An origin-destination cost matrix line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ODLine {
    /// Origin ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    origin_id: Option<i32>,

    /// Destination ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    destination_id: Option<i32>,

    /// Total travel time (minutes).
    #[serde(skip_serializing_if = "Option::is_none")]
    total_time: Option<f64>,

    /// Total distance.
    #[serde(skip_serializing_if = "Option::is_none")]
    total_distance: Option<f64>,

    /// Origin name.
    #[serde(skip_serializing_if = "Option::is_none")]
    origin_name: Option<String>,

    /// Destination name.
    #[serde(skip_serializing_if = "Option::is_none")]
    destination_name: Option<String>,
}
