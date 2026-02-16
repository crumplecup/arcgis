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

    /// Extracts an NALocation from a FeatureSet Feature.
    ///
    /// Infallible - missing fields are represented as None.
    pub(crate) fn from_feature(feature: &crate::Feature) -> Self {
        tracing::debug!("Converting FeatureSet feature to NALocation");

        let attrs = feature.attributes();

        let name = attrs
            .get("Name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let curb_approach =
            attrs
                .get("CurbApproach")
                .and_then(|v| v.as_i64())
                .and_then(|i| match i {
                    0 => Some(CurbApproach::EitherSide),
                    1 => Some(CurbApproach::RightSide),
                    2 => Some(CurbApproach::LeftSide),
                    3 => Some(CurbApproach::NoUTurn),
                    _ => None,
                });

        // Convert old geometry type to new via JSON (temporary during migration)
        let geometry = feature
            .geometry()
            .as_ref()
            .and_then(|old_geom| {
                serde_json::to_value(old_geom)
                    .ok()
                    .and_then(|v| serde_json::from_value(v).ok())
            })
            .unwrap_or(ArcGISGeometry::Point(crate::ArcGISPoint::new(0.0, 0.0)));

        Self {
            geometry,
            name,
            curb_approach,
            bearing: None,
            bearing_tolerance: None,
            nav_latency: None,
        }
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
#[derive(Debug, Clone, PartialEq, Serialize, Getters)]
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

impl<'de> serde::Deserialize<'de> for RouteResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct RouteResultVisitor;

        impl<'de> Visitor<'de> for RouteResultVisitor {
            type Value = RouteResult;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a RouteResult with FeatureSet routes and stops")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut routes_fs: Option<crate::FeatureSet> = None;
                let mut stops_fs: Option<crate::FeatureSet> = None;
                let mut barriers: Option<Vec<NALocation>> = None;
                let mut messages: Option<Vec<NAMessage>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "routes" => {
                            routes_fs = Some(map.next_value()?);
                        }
                        "stops" => {
                            stops_fs = Some(map.next_value()?);
                        }
                        "barriers" => {
                            barriers = Some(map.next_value()?);
                        }
                        "messages" => {
                            messages = Some(map.next_value()?);
                        }
                        _ => {
                            // Skip unknown fields
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let routes_fs = routes_fs.unwrap_or_default();
                let stops_fs = stops_fs.unwrap_or_default();
                let barriers = barriers.unwrap_or_default();
                let messages = messages.unwrap_or_default();

                tracing::debug!(
                    route_feature_count = routes_fs.features().len(),
                    stop_feature_count = stops_fs.features().len(),
                    "Deserializing RouteResult from FeatureSets"
                );

                // Convert FeatureSet features to Route objects (infallible)
                let routes: Vec<Route> = routes_fs
                    .features()
                    .iter()
                    .map(Route::from_feature)
                    .collect();

                // Convert FeatureSet features to Stop objects (infallible)
                let stops: Vec<Stop> = stops_fs.features().iter().map(Stop::from_feature).collect();

                tracing::debug!(
                    route_count = routes.len(),
                    stop_count = stops.len(),
                    "Successfully deserialized RouteResult"
                );

                Ok(RouteResult {
                    routes,
                    stops,
                    barriers,
                    messages,
                })
            }
        }

        deserializer.deserialize_map(RouteResultVisitor)
    }
}

/// A calculated route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    /// Route name.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// Total length (miles).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Total_Miles")]
    #[serde(alias = "total_length")] // Accept both for backwards compatibility
    total_length: Option<f64>,

    /// Total time (minutes).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Total_TravelTime")]
    #[serde(alias = "total_time")] // Accept both for backwards compatibility
    total_time: Option<f64>,

    /// Total drive time (minutes).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Total_DriveTime")]
    #[serde(alias = "total_drive_time")] // Accept both for backwards compatibility
    total_drive_time: Option<f64>,

    /// Total wait time (minutes).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Total_WaitTime")]
    #[serde(alias = "total_wait_time")] // Accept both for backwards compatibility
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

impl Route {
    /// Extracts a Route from a FeatureSet Feature.
    ///
    /// Infallible - missing fields are represented as None.
    fn from_feature(feature: &crate::Feature) -> Self {
        tracing::debug!("Converting FeatureSet feature to Route");

        let attrs = feature.attributes();

        let name = attrs
            .get("Name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let total_length = attrs.get("Total_Miles").and_then(|v| v.as_f64());

        let total_time = attrs.get("Total_TravelTime").and_then(|v| v.as_f64());

        let total_drive_time = attrs.get("Total_DriveTime").and_then(|v| v.as_f64());

        let total_wait_time = attrs.get("Total_WaitTime").and_then(|v| v.as_f64());

        // Convert old geometry type to new via JSON (temporary during migration)
        let geometry = feature
            .geometry()
            .as_ref()
            .and_then(|old_geom| {
                serde_json::to_value(old_geom)
                    .ok()
                    .and_then(|v| serde_json::from_value(v).ok())
            })
            .unwrap_or_else(|| {
                // Default to empty point if conversion fails
                ArcGISGeometry::Point(crate::ArcGISPoint::new(0.0, 0.0))
            });

        tracing::debug!(
            name = ?name,
            total_miles = ?total_length,
            total_time_minutes = ?total_time,
            "Extracted route data from feature"
        );

        Route {
            name,
            total_length,
            total_time,
            total_drive_time,
            total_wait_time,
            geometry: Some(geometry),
            directions: Vec::new(), // Directions come from separate array in response
            start_time: None,       // Not in route attributes
            end_time: None,         // Not in route attributes
        }
    }
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

impl Stop {
    /// Extracts a Stop from a FeatureSet Feature.
    ///
    /// Infallible - missing fields are represented as None.
    fn from_feature(feature: &crate::Feature) -> Self {
        tracing::debug!("Converting FeatureSet feature to Stop");

        let attrs = feature.attributes();

        let name = attrs
            .get("Name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Convert old geometry type to new via JSON (temporary during migration)
        let geometry = feature
            .geometry()
            .as_ref()
            .and_then(|old_geom| {
                serde_json::to_value(old_geom)
                    .ok()
                    .and_then(|v| serde_json::from_value(v).ok())
            })
            .unwrap_or_else(|| {
                // Default to empty point if conversion fails
                ArcGISGeometry::Point(crate::ArcGISPoint::new(0.0, 0.0))
            });

        let arrival_time = None; // Not directly in attributes

        let departure_time = None; // Not directly in attributes

        let wait_time = None; // Not directly in attributes

        let cumulative_length = attrs.get("Cumul_Miles").and_then(|v| v.as_f64());

        let sequence = attrs
            .get("Sequence")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32);

        tracing::debug!(
            name = ?name,
            sequence = ?sequence,
            cumul_miles = ?cumulative_length,
            "Extracted stop data from feature"
        );

        Stop {
            name,
            geometry: Some(geometry),
            arrival_time,
            departure_time,
            wait_time,
            cumulative_length,
            sequence,
        }
    }
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

    /// Whether to return service area polygons.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_polygons: Option<bool>,
}

impl ServiceAreaParameters {
    /// Creates a builder for ServiceAreaParameters.
    pub fn builder() -> ServiceAreaParametersBuilder {
        ServiceAreaParametersBuilder::default()
    }
}

/// Helper to deserialize service area feature sets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: serde::Deserialize<'de>"))]
struct ServiceAreaFeatureSet<T> {
    #[serde(default)]
    features: Vec<T>,
}

impl<T> Default for ServiceAreaFeatureSet<T> {
    fn default() -> Self {
        Self {
            features: Vec::new(),
        }
    }
}

/// Result from service area calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceAreaResult {
    /// Service area polygons.
    #[serde(default, rename = "saPolygons")]
    sapolygons_raw: ServiceAreaFeatureSet<ServiceAreaPolygon>,

    /// Service area polylines (network edges).
    #[serde(default, rename = "saPolylines")]
    sapolylines_raw: ServiceAreaFeatureSet<ServiceAreaPolyline>,

    /// Facilities that were used.
    #[serde(default)]
    facilities: Vec<NALocation>,

    /// Messages from the solve operation.
    #[serde(default)]
    messages: Vec<NAMessage>,
}

impl ServiceAreaResult {
    /// Gets the service area polygons.
    pub fn service_area_polygons(&self) -> &[ServiceAreaPolygon] {
        &self.sapolygons_raw.features
    }

    /// Gets the service area polylines.
    pub fn service_area_polylines(&self) -> &[ServiceAreaPolyline] {
        &self.sapolylines_raw.features
    }

    /// Gets the facilities.
    pub fn facilities(&self) -> &[NALocation] {
        &self.facilities
    }

    /// Gets the messages.
    pub fn messages(&self) -> &[NAMessage] {
        &self.messages
    }
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

/// Helper to deserialize closest facility feature sets.
/// Result from closest facility calculation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClosestFacilityResult {
    /// Routes from incidents to facilities.
    routes: Vec<Route>,

    /// Facilities that were analyzed (returned as feature set).
    facilities: Vec<NALocation>,

    /// Incidents that were analyzed (returned as feature set).
    incidents: Vec<NALocation>,

    /// Messages from the solve operation.
    messages: Vec<NAMessage>,
}

impl ClosestFacilityResult {
    /// Gets the routes.
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// Gets the facilities.
    pub fn facilities(&self) -> &[NALocation] {
        &self.facilities
    }

    /// Gets the incidents.
    pub fn incidents(&self) -> &[NALocation] {
        &self.incidents
    }

    /// Gets the messages.
    pub fn messages(&self) -> &[NAMessage] {
        &self.messages
    }
}

impl<'de> serde::Deserialize<'de> for ClosestFacilityResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct ClosestFacilityResultVisitor;

        impl<'de> Visitor<'de> for ClosestFacilityResultVisitor {
            type Value = ClosestFacilityResult;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a ClosestFacilityResult with FeatureSet routes, facilities, and incidents",
                )
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut routes_fs: Option<crate::FeatureSet> = None;
                let mut facilities_fs: Option<crate::FeatureSet> = None;
                let mut incidents_fs: Option<crate::FeatureSet> = None;
                let mut messages: Option<Vec<NAMessage>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "routes" => {
                            routes_fs = Some(map.next_value()?);
                        }
                        "facilities" => {
                            facilities_fs = Some(map.next_value()?);
                        }
                        "incidents" => {
                            incidents_fs = Some(map.next_value()?);
                        }
                        "messages" => {
                            messages = Some(map.next_value()?);
                        }
                        _ => {
                            // Skip unknown fields
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let routes_fs = routes_fs.unwrap_or_default();
                let facilities_fs = facilities_fs.unwrap_or_default();
                let incidents_fs = incidents_fs.unwrap_or_default();
                let messages = messages.unwrap_or_default();

                tracing::debug!(
                    route_feature_count = routes_fs.features().len(),
                    facility_feature_count = facilities_fs.features().len(),
                    incident_feature_count = incidents_fs.features().len(),
                    "Deserializing ClosestFacilityResult from FeatureSets"
                );

                // Convert FeatureSet features to Route objects using from_feature
                let routes: Vec<Route> = routes_fs
                    .features()
                    .iter()
                    .map(Route::from_feature)
                    .collect();

                // Convert FeatureSet features to NALocation objects
                let facilities: Vec<NALocation> = facilities_fs
                    .features()
                    .iter()
                    .map(NALocation::from_feature)
                    .collect();

                let incidents: Vec<NALocation> = incidents_fs
                    .features()
                    .iter()
                    .map(NALocation::from_feature)
                    .collect();

                tracing::debug!(
                    route_count = routes.len(),
                    facility_count = facilities.len(),
                    incident_count = incidents.len(),
                    "Successfully deserialized ClosestFacilityResult"
                );

                Ok(ClosestFacilityResult {
                    routes,
                    facilities,
                    incidents,
                    messages,
                })
            }
        }

        deserializer.deserialize_map(ClosestFacilityResultVisitor)
    }
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

// Internal structure matching the API response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ODCostMatrixResponse {
    #[serde(default, rename = "odCostMatrix")]
    od_cost_matrix: ODCostMatrixData,
    #[serde(default)]
    messages: Vec<NAMessage>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ODCostMatrixData {
    #[serde(default)]
    cost_attribute_names: Vec<String>,
    #[serde(flatten)]
    matrix: std::collections::HashMap<String, std::collections::HashMap<String, Vec<f64>>>,
}

/// Result from origin-destination cost matrix calculation.
#[derive(Debug, Clone, PartialEq, Getters)]
pub struct ODCostMatrixResult {
    /// Origin-destination cost matrix lines.
    od_lines: Vec<ODLine>,

    /// Origins that were analyzed.
    origins: Vec<NALocation>,

    /// Destinations that were analyzed.
    destinations: Vec<NALocation>,

    /// Messages from the solve operation.
    messages: Vec<NAMessage>,
}

impl<'de> serde::Deserialize<'de> for ODCostMatrixResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let response = ODCostMatrixResponse::deserialize(deserializer)?;

        // Convert nested map to Vec<ODLine>
        let mut od_lines = Vec::new();
        for (origin_id_str, destinations) in response.od_cost_matrix.matrix {
            // Skip costAttributeNames entry
            if origin_id_str == "costAttributeNames" {
                continue;
            }

            let origin_id = origin_id_str.parse::<i32>().ok();
            for (dest_id_str, costs) in destinations {
                let destination_id = dest_id_str.parse::<i32>().ok();

                // Map cost values by name
                let mut total_time = None;
                let mut total_distance = None;

                for (i, name) in response
                    .od_cost_matrix
                    .cost_attribute_names
                    .iter()
                    .enumerate()
                {
                    if i < costs.len() {
                        match name.as_str() {
                            "TravelTime" | "Minutes" => total_time = Some(costs[i]),
                            "Miles" | "Kilometers" => total_distance = Some(costs[i]),
                            _ => {}
                        }
                    }
                }

                od_lines.push(ODLine {
                    origin_id,
                    destination_id,
                    total_time,
                    total_distance,
                    origin_name: None,
                    destination_name: None,
                });
            }
        }

        tracing::debug!(
            od_line_count = od_lines.len(),
            "Deserialized OD cost matrix result"
        );

        Ok(ODCostMatrixResult {
            od_lines,
            origins: Vec::new(),
            destinations: Vec::new(),
            messages: response.messages,
        })
    }
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
