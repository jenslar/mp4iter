//! Data reference atom (`dref`).
//! Declares source(s) os media data in track.
//!
//! Location: `moov/trak[multiple]/mdia/minf/dinf/dref`

use binrw::BinRead;

/// Data reference atom (`dref`).
/// Declares source(s) os media data in track.
///
/// Location: `moov/trak[multiple]/mdia/minf/dinf/dref`
#[derive(Debug, Default, BinRead)]
#[br(big)]
pub struct Dref {
    _version: u8,
    _flags: [u8; 3],
    _number_of_entries: u32,
    #[br(count = _number_of_entries)]
    table: Vec<DrefTable>
}

impl Dref {
    pub fn table(&self) -> &[DrefTable] {
        &self.table
    }
}

#[derive(Debug, Default, BinRead)]
#[br(big)]
pub struct DrefTable {
    pub size: u32,
    #[br(count = size)]
    pub data: Vec<u8>,
}
