//! Composition offset atom (`ctts`).
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/composition_offset_atom>

use binrw::BinRead;

/// Composition offset atom (`ctts`).
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/composition_offset_atom>
#[derive(Debug, Default, BinRead)]
#[br(big)]
pub struct Ctts {
    _version: u8,
    _flags: [u8; 3],
    _entry_count: u32,
    #[br(count = _entry_count)]
    offset_table: Vec<OffsetTableEntry>
}

impl Ctts {
    pub fn offset_table(&self) -> &[OffsetTableEntry] {
        &self.offset_table
    }
}

/// Composition offset table
#[derive(Debug, BinRead)]
#[br(big)]
pub struct OffsetTableEntry {
    pub sample_count: u32,
    pub composition_offset: u32
}
