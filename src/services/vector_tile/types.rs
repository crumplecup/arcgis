//! Vector tile types and structures.

use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A Mapbox GL style document.
///
/// This is the complete style specification for rendering vector tiles,
/// including layers, sources, sprite sheets, and glyphs.
///
/// Reference: https://docs.mapbox.com/mapbox-gl-js/style-spec/
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct VectorTileStyle {
    /// Style specification version (usually 8).
    version: u32,

    /// Style name.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// Style metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Value>,

    /// Data sources (vector tile URLs, etc.).
    sources: Value,

    /// Sprite sheet URLs for icons.
    #[serde(skip_serializing_if = "Option::is_none")]
    sprite: Option<String>,

    /// Glyph (font) URLs.
    #[serde(skip_serializing_if = "Option::is_none")]
    glyphs: Option<String>,

    /// Layer definitions (what to draw and how).
    layers: Vec<Value>,

    /// Initial view center coordinates [lon, lat].
    #[serde(skip_serializing_if = "Option::is_none")]
    center: Option<Vec<f64>>,

    /// Initial zoom level.
    #[serde(skip_serializing_if = "Option::is_none")]
    zoom: Option<f64>,

    /// Initial bearing (rotation) in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    bearing: Option<f64>,

    /// Initial pitch (tilt) in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pitch: Option<f64>,
}

/// A font stack name (e.g., "Arial Regular", "Noto Sans Bold").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontStack(pub String);

impl FontStack {
    /// Creates a new font stack.
    pub fn new(name: impl Into<String>) -> Self {
        FontStack(name.into())
    }
}

impl From<String> for FontStack {
    fn from(s: String) -> Self {
        FontStack(s)
    }
}

impl From<&str> for FontStack {
    fn from(s: &str) -> Self {
        FontStack(s.to_string())
    }
}

/// A glyph range for font characters (e.g., 0-255).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_getters::Getters)]
pub struct GlyphRange {
    /// Starting character code.
    start: u32,
    /// Ending character code.
    end: u32,
}

impl GlyphRange {
    /// Creates a new glyph range.
    pub fn new(start: u32, end: u32) -> Self {
        GlyphRange { start, end }
    }

    /// Standard ASCII range (0-255).
    pub fn ascii() -> Self {
        GlyphRange::new(0, 255)
    }

    /// Common Unicode ranges.
    pub fn basic_latin() -> Self {
        GlyphRange::new(0, 127)
    }

    /// Latin-1 Supplement.
    pub fn latin_1_supplement() -> Self {
        GlyphRange::new(128, 255)
    }

    /// Formats the range as a string (e.g., "0-255").
    pub fn format(&self) -> String {
        format!("{}-{}", self.start, self.end)
    }
}

impl std::fmt::Display for GlyphRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}
