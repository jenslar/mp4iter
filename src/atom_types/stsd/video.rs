use binrw::BinRead;

use crate::support::counted_string;

#[derive(Debug, BinRead)]
#[br(import {size: u32})]
pub struct Video {
    /// A 16-bit integer that holds the sample description version.
    _version: u16,
    /// A 16-bit integer.
    _revision_level: u16,
    /// A 32-bit integer that specifies the developer of the compressor that generated the compressed data.
    vendor: u32,
    /// A 32-bit integer that indicates the degree of temporal compression.
    temporal_quality: u32,
    /// A 32-bit integer that indicates the degree of spatial compression.
    spatial_quality: u32,
    /// A 16-bit integer that specifies the width of the source image in pixels.
    width: u16,
    /// A 16-bit integer that specifies the height of the source image in pixels.
    height: u16,
    /// A 32-bit fixed-point number containing the horizontal resolution of the image in pixels per inch.
    /// I.e. the interpreted result is `horizontal_resolution_u32 / 2^16`.
    horizontal_resolution: u32,
    /// A 32-bit fixed-point number containing the vertical resolution of the image in pixels per inch.
    /// I.e. the interpreted result is `horizontal_resolution_u32 / 2^16`.
    vertical_resolution: u32,
    /// A 32-bit integer.
    data_size: u32,
    /// A 16-bit integer that indicates how many frames of compressed data are stored in each sample.
    frame_count: u16, // byte count so far 36
    // frame_count: u32, // byte count so far 36
    /// A 32-byte Pascal string containing the name of the compressor that created the image, such as “jpeg”.
    /// NB: first byte = char count, remaining bytes are set to 0 so could possibly ignore
    /// length and just filter out null bytes.
    #[br(map = |data: [u8; 32]| counted_string(&data, true))]
    compressor_name: String,
    /// A 16-bit integer that indicates the pixel depth of the compressed image.
    depth: i16,
    /// A 16-bit integer that identifies which color table to use.
    color_table_id: i16,

    /// Video extensions as bytes (currently unsupported)
    #[br(count = size - 16 - 70)] // 16 bytes in FormatType, 70 in preceding fields
    extensions: Vec<u8>
}

impl Video {
    pub fn vendor(&self) -> u32 {
        self.vendor
    }

    pub fn temporal_quality(&self) -> u32 {
        self.temporal_quality
    }

    pub fn spatial_quality(&self) -> u32 {
        self.spatial_quality
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn horizontal_resolution(&self) -> f64 {
        self.horizontal_resolution as f64 / 2_u32.pow(16) as f64
    }

    pub fn vertical_resolution(&self) -> f64 {
        self.vertical_resolution as f64 / 2_u32.pow(16) as f64
    }

    pub fn data_size(&self) -> u32 {
        self.data_size
    }
    pub fn frame_count(&self) -> u16 {
        self.frame_count
    }
    pub fn compressor_name(&self) -> &str {
        self.compressor_name.as_str()
    }
    pub fn depth(&self) -> i16 {
        self.depth
    }
    pub fn color_table_id(&self) -> i16 {
        self.color_table_id
    }
    pub fn extensions(&self) -> &[u8] {
        self.extensions.as_slice()
    }
}
