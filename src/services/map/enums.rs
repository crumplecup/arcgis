//! Enumerations for map service operations.

use serde::{Deserialize, Serialize};

/// Image format for exported maps.
///
/// Different formats support different capabilities:
/// - PNG formats support transparency
/// - GIF supports transparency and animation
/// - JPEG is compact but lossy
/// - PDF and SVG are vector formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    /// PNG format (default) - supports transparency
    #[default]
    Png,

    /// PNG with 8-bit color depth
    #[serde(rename = "png8")]
    Png8,

    /// PNG with 24-bit color depth
    #[serde(rename = "png24")]
    Png24,

    /// PNG with 32-bit color depth (includes alpha channel)
    #[serde(rename = "png32")]
    Png32,

    /// JPEG format - compact, lossy compression
    #[serde(rename = "jpg")]
    Jpg,

    /// PDF format - vector format
    Pdf,

    /// BMP format - uncompressed bitmap
    Bmp,

    /// GIF format - supports transparency
    Gif,

    /// SVG format - scalable vector graphics
    Svg,

    /// Compressed SVG
    Svgz,

    /// Enhanced Metafile Format
    Emf,

    /// PostScript
    Ps,
}

/// Layer visibility operations.
///
/// Controls which layers are visible in the exported map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerOperation {
    /// Show only the specified layers
    Show,

    /// Hide the specified layers (show all others)
    Hide,

    /// Include the specified layers (add to default visibility)
    Include,

    /// Exclude the specified layers (remove from default visibility)
    Exclude,
}

impl LayerOperation {
    /// Returns the string representation for the API.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Show => "show",
            Self::Hide => "hide",
            Self::Include => "include",
            Self::Exclude => "exclude",
        }
    }
}

/// Time relationship for temporal queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TimeRelation {
    /// Features that overlap the time window (default)
    #[default]
    #[serde(rename = "esriTimeRelationOverlaps")]
    Overlaps,

    /// Features that occur after the time window
    #[serde(rename = "esriTimeRelationAfter")]
    After,

    /// Features that occur before the time window
    #[serde(rename = "esriTimeRelationBefore")]
    Before,
}

/// Response format for map service operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    /// HTML format - for browser display
    Html,

    /// JSON format (default for programmatic access)
    #[default]
    Json,

    /// Pretty-printed JSON
    #[serde(rename = "pjson")]
    PJson,

    /// Direct image format - returns image bytes directly (no JSON wrapper)
    Image,

    /// KMZ format - compressed KML
    Kmz,
}

/// Which layers to identify in an identify operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LayerSelection {
    /// Top-most visible layer only
    Top,

    /// All visible layers (default)
    #[default]
    Visible,

    /// All layers regardless of visibility
    All,
}
