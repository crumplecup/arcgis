//! Type-safe projected coordinate types.
//!
//! This module provides types that encode spatial reference information in the type system,
//! enabling compile-time guarantees about coordinate system usage.

use serde::{Deserialize, Serialize};

/// Trait for types representing points in a specific spatial reference system.
///
/// This allows the type system to encode spatial reference information,
/// enabling compile-time guarantees about coordinate system usage.
pub trait ProjectedPoint: Sized {
    /// The Well-Known ID (WKID) of this spatial reference system.
    const WKID: u32;

    /// The human-readable name of this spatial reference system.
    const NAME: &'static str;

    /// X coordinate (easting/longitude depending on projection).
    fn x(&self) -> f64;

    /// Y coordinate (northing/latitude depending on projection).
    fn y(&self) -> f64;

    /// Formats the point as JSON with spatial reference for ArcGIS REST API.
    ///
    /// This is used for operations that require explicit spatial reference specification.
    fn to_location_json(&self) -> String {
        format!(
            r#"{{"x":{},"y":{},"spatialReference":{{"wkid":{}}}}}"#,
            self.x(),
            self.y(),
            Self::WKID
        )
    }
}

/// A point in WGS84 (EPSG:4326) geographic coordinate system.
///
/// This is the standard GPS coordinate system using decimal degrees.
/// Longitude ranges from -180 to 180, latitude from -90 to 90.
///
/// # Examples
///
/// ```
/// use arcgis::{Wgs84Point, ProjectedPoint};
///
/// let point = Wgs84Point::new(-117.195, 34.056);
/// assert_eq!(point.lon(), -117.195);
/// assert_eq!(point.lat(), 34.056);
/// assert_eq!(Wgs84Point::WKID, 4326);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Wgs84Point {
    /// Longitude in decimal degrees.
    #[serde(rename = "x")]
    lon: f64,

    /// Latitude in decimal degrees.
    #[serde(rename = "y")]
    lat: f64,
}

impl Wgs84Point {
    /// Creates a new WGS84 point from longitude and latitude.
    ///
    /// # Arguments
    ///
    /// * `lon` - Longitude in decimal degrees (-180 to 180)
    /// * `lat` - Latitude in decimal degrees (-90 to 90)
    ///
    /// # Examples
    ///
    /// ```
    /// use arcgis::Wgs84Point;
    ///
    /// let point = Wgs84Point::new(-117.195, 34.056);
    /// ```
    pub fn new(lon: f64, lat: f64) -> Self {
        Self { lon, lat }
    }

    /// Longitude in decimal degrees (-180 to 180).
    pub fn lon(&self) -> f64 {
        self.lon
    }

    /// Latitude in decimal degrees (-90 to 90).
    pub fn lat(&self) -> f64 {
        self.lat
    }
}

impl ProjectedPoint for Wgs84Point {
    const WKID: u32 = 4326;
    const NAME: &'static str = "WGS84";

    fn x(&self) -> f64 {
        self.lon
    }

    fn y(&self) -> f64 {
        self.lat
    }
}

/// A point in Web Mercator (EPSG:3857) projection.
///
/// This is the projection used by most web mapping applications
/// (Google Maps, OpenStreetMap, ArcGIS Online).
///
/// Coordinates are in meters, with valid ranges approximately:
/// - X (easting): -20037508 to 20037508
/// - Y (northing): -20037508 to 20037508
///
/// # Examples
///
/// ```
/// use arcgis::{WebMercatorPoint, ProjectedPoint};
///
/// // Esri Redlands campus in Web Mercator
/// let point = WebMercatorPoint::new(-13046213.0, 4036389.0);
/// assert_eq!(WebMercatorPoint::WKID, 3857);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WebMercatorPoint {
    /// X coordinate (easting) in meters.
    x: f64,

    /// Y coordinate (northing) in meters.
    y: f64,
}

impl WebMercatorPoint {
    /// Creates a new Web Mercator point from X and Y coordinates in meters.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (easting) in meters
    /// * `y` - Y coordinate (northing) in meters
    ///
    /// # Examples
    ///
    /// ```
    /// use arcgis::WebMercatorPoint;
    ///
    /// let point = WebMercatorPoint::new(-13046213.0, 4036389.0);
    /// ```
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// X coordinate (easting) in meters.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Y coordinate (northing) in meters.
    pub fn y(&self) -> f64 {
        self.y
    }
}

impl ProjectedPoint for WebMercatorPoint {
    const WKID: u32 = 3857;
    const NAME: &'static str = "Web Mercator";

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}

/// A point in a State Plane coordinate system.
///
/// The type parameter `ZONE` encodes the specific State Plane zone WKID,
/// allowing the type system to distinguish between different zones at compile time.
///
/// # Examples
///
/// ```
/// use arcgis::{StatePlanePoint, ProjectedPoint};
///
/// // California Zone 5 (feet)
/// type CalZone5 = StatePlanePoint<2229>;
/// let point = CalZone5::new(6389000.0, 2093000.0);
/// assert_eq!(CalZone5::WKID, 2229);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StatePlanePoint<const ZONE: u32> {
    /// X coordinate (easting).
    x: f64,

    /// Y coordinate (northing).
    y: f64,
}

impl<const ZONE: u32> StatePlanePoint<ZONE> {
    /// Creates a new State Plane point.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// X coordinate (easting).
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Y coordinate (northing).
    pub fn y(&self) -> f64 {
        self.y
    }
}

impl<const ZONE: u32> ProjectedPoint for StatePlanePoint<ZONE> {
    const WKID: u32 = ZONE;
    const NAME: &'static str = "State Plane";

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}

// Type aliases for common State Plane zones

/// California Zone 5 (US Survey Feet) - WKID 2229.
pub type CaliforniaZone5 = StatePlanePoint<2229>;

/// California Zone 5 (meters) - WKID 2771.
pub type CaliforniaZone5Meters = StatePlanePoint<2771>;

/// Oregon North (US Survey Feet) - WKID 2269.
pub type OregonNorth = StatePlanePoint<2269>;

/// Oregon North (meters) - WKID 2913.
pub type OregonNorthMeters = StatePlanePoint<2913>;

/// Washington North (US Survey Feet) - WKID 2285.
pub type WashingtonNorth = StatePlanePoint<2285>;

/// Washington North (meters) - WKID 2926.
pub type WashingtonNorthMeters = StatePlanePoint<2926>;
