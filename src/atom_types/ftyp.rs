//! File type compatibility atom (`ftyp`).
//!
//! Location: `ftyp` (the very first atom in an MP4 file)
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/file_type_compatibility_atom>

use binrw::BinRead;

use crate::support::chars_from_bytes;

/// File type compatibility atom (`ftyp`).
///
/// Location: `ftyp` (the very first atom in an MP4 file)
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/file_type_compatibility_atom>
#[derive(Debug, Default, BinRead)]
#[br(big, import {data_size: u32})]
pub struct Ftyp {
    pub(crate) major_brand: [u8; 4],
    /// MP4: seems to be set to 0.
    /// QuickTime: Four binary-coded decimal values, indicating the century, year, and month of format spec.
    pub(crate) minor_version: [u8; 4],
    #[br(count = (data_size - 8) / 4)]
    pub(crate) compatible_brands: Vec<[u8; 4]>
}

impl Ftyp {
    pub fn major_brand(&self) -> String {
        chars_from_bytes(self.major_brand)
            .iter()
            .collect()
    }

    pub fn minor_version(&self) -> &[u8; 4] {
        &self.minor_version
    }

    pub fn compatible_brands(&self) -> Vec<String> {
        self.compatible_brands
            .iter()
            .map(|c| chars_from_bytes(*c).iter().collect::<String>())
            .collect()
    }
}