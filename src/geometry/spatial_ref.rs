//! ESRI Spatial Reference System types.

use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// ESRI Spatial Reference System.
///
/// Defines the coordinate system for geometry coordinates.
///
/// # Examples
///
/// ```
/// # use arcgis::SpatialReference;
/// // Create WGS84 (GPS coordinates)
/// let wgs84 = SpatialReference::wgs84();
/// assert!(wgs84.is_geographic());
///
/// // Create Web Mercator (web mapping)
/// let web_mercator = SpatialReference::web_mercator();
/// assert!(web_mercator.is_projected());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Getters, derive_builder::Builder)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct SpatialReference {
    /// Well-Known ID (e.g., 4326 for WGS84, 3857 for Web Mercator).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    wkid: Option<u32>,

    /// Latest WKID (for compatibility with updated definitions).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    latest_wkid: Option<u32>,

    /// Well-Known Text representation (alternative to WKID).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    wkt: Option<String>,
}

impl SpatialReference {
    /// Creates WGS84 (EPSG:4326) spatial reference.
    ///
    /// This is the standard geographic coordinate system used by GPS.
    ///
    /// # Examples
    ///
    /// ```
    /// # use arcgis::SpatialReference;
    /// let wgs84 = SpatialReference::wgs84();
    /// assert_eq!(*wgs84.wkid(), Some(4326));
    /// ```
    pub const fn wgs84() -> Self {
        Self {
            wkid: Some(4326),
            latest_wkid: Some(4326),
            wkt: None,
        }
    }

    /// Creates Web Mercator (EPSG:3857) spatial reference.
    ///
    /// This is the standard projection used by web mapping applications.
    ///
    /// # Examples
    ///
    /// ```
    /// # use arcgis::SpatialReference;
    /// let web_mercator = SpatialReference::web_mercator();
    /// assert_eq!(*web_mercator.wkid(), Some(3857));
    /// ```
    pub const fn web_mercator() -> Self {
        Self {
            wkid: Some(3857),
            latest_wkid: Some(3857),
            wkt: None,
        }
    }

    /// Checks if this is a geographic coordinate system (lat/lon).
    ///
    /// Geographic systems use angular units (degrees).
    ///
    /// # Examples
    ///
    /// ```
    /// # use arcgis::SpatialReference;
    /// assert!(SpatialReference::wgs84().is_geographic());
    /// assert!(!SpatialReference::web_mercator().is_geographic());
    /// ```
    #[instrument(skip(self), fields(wkid = ?self.wkid))]
    pub fn is_geographic(&self) -> bool {
        let result = matches!(self.wkid, Some(4326) | Some(4269) | Some(4267));
        tracing::debug!(is_geographic = result, "Checked coordinate system type");
        result
    }

    /// Checks if this is a projected coordinate system.
    ///
    /// Projected systems use linear units (meters, feet).
    ///
    /// # Examples
    ///
    /// ```
    /// # use arcgis::SpatialReference;
    /// assert!(!SpatialReference::wgs84().is_projected());
    /// assert!(SpatialReference::web_mercator().is_projected());
    /// ```
    #[instrument(skip(self), fields(wkid = ?self.wkid))]
    pub fn is_projected(&self) -> bool {
        let result = self.wkid.is_some() && !self.is_geographic();
        tracing::debug!(is_projected = result, "Checked coordinate system type");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
            )
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn test_wgs84() {
        init_tracing();
        let sr = SpatialReference::wgs84();
        assert_eq!(*sr.wkid(), Some(4326));
        assert_eq!(*sr.latest_wkid(), Some(4326));
        assert!(sr.is_geographic());
        assert!(!sr.is_projected());
    }

    #[test]
    fn test_web_mercator() {
        init_tracing();
        let sr = SpatialReference::web_mercator();
        assert_eq!(*sr.wkid(), Some(3857));
        assert_eq!(*sr.latest_wkid(), Some(3857));
        assert!(!sr.is_geographic());
        assert!(sr.is_projected());
    }

    #[test]
    fn test_builder() -> anyhow::Result<()> {
        init_tracing();
        let sr = SpatialReferenceBuilder::default()
            .wkid(2263)
            .latest_wkid(2263)
            .build()?;

        assert_eq!(*sr.wkid(), Some(2263));
        assert!(sr.is_projected());
        Ok(())
    }
}
