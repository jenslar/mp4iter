//! Composition offset atom `ctts`.

use binrw::BinRead;

/// Composition offset atom
#[derive(Debug, BinRead)]
#[br(big)]
pub struct Ctts {
    _version: u8,
    _flags: [u8; 3],
    entry_count: u32,
    #[br(count = entry_count)]
    offset_table: Vec<OffsetTableEntry>
}

/// Composition offset table
#[derive(Debug, BinRead)]
#[br(big)]
pub struct OffsetTableEntry {
    sample_count: u32,
    composition_offset: u32
}