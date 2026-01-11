//! Protocol Buffer (PBF) format support for ArcGIS Feature Services.
//!
//! This module provides efficient binary serialization for feature queries using
//! Protocol Buffers, offering 3-5x performance improvement over JSON for large datasets.
//!
//! The PBF format is supported by ArcGIS Enterprise 10.7+ and ArcGIS Online.

// Include the generated protocol buffer code
#[allow(clippy::all)]
#[allow(missing_docs)]
mod esri_p_buffer {
    include!("esri_p_buffer.rs");
}

pub use esri_p_buffer::*;

mod decoder;

pub use decoder::decode_feature_collection;
