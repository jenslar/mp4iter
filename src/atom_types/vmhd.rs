//! Video media information header atom (`vmhd`).
//!
//! Location: `moov/trak[multiple]/mdia/minf/vmhd`
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/video_media_information_header_atom>

use binrw::BinRead;

/// Video media information header atom (`vmhd`).
///
/// Location: `moov/trak[multiple]/mdia/minf/vmhd`
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/video_media_information_header_atom>
#[derive(Debug, Default, BinRead)]
pub struct Vmhd {
    _version: u8,
    _flags: [u8; 3],
    /// Specified transfer mode.
    pub(crate) graphics_mode: u16,
    /// Specifes the red, green, and blue colours
    /// for the transfer mode operation.
    pub(crate) op_color: [u16; 3]
}

impl Vmhd {
    pub fn graphics_mode(&self) -> u16 {
        self.graphics_mode
    }
    pub fn op_color(&self) -> &[u16; 3] {
        &self.op_color
    }
}