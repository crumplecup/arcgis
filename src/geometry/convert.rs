//! Conversion functions between ArcGIS JSON and geo-types.

use crate::{Error, Result};
use geo_types::Point;

// Placeholder conversions - to be implemented

/// Converts an ArcGIS point to a geo-types Point.
pub fn from_arcgis_point(_arcgis_point: &serde_json::Value) -> Result<Point> {
    // TODO: Implement actual conversion
    Err(Error::geometry("Not yet implemented"))
}

/// Converts a geo-types Point to ArcGIS JSON.
pub fn to_arcgis_point(_point: &Point) -> Result<serde_json::Value> {
    // TODO: Implement actual conversion
    Err(Error::geometry("Not yet implemented"))
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Not yet implemented"]
    fn test_point_conversion() {
        // TODO: Add tests once conversion is implemented
    }
}
