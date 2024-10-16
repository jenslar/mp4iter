//! Sync sample atom (`stss`).
//!
//! Location: `moov/trak[multiple]/mdia/minf/stbl/stss`
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/sync_sample_atom>

use binrw::BinRead;

/// Sync sample atom (`stss`).
///
/// Location:
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/sync_sample_atom>
#[derive(Debug, Default, BinRead)]
#[br(big)]
pub struct Stss {
    _version: u8,
    _flags: [u8; 3],
    _number_of_entries: u32,
    #[br(count = _number_of_entries)]
    pub(crate) sync_sample_table: Vec<SyncSample>
}

impl Stss {
    pub fn sync_sample_table(&self) -> &[SyncSample] {
        &self.sync_sample_table
    }
}

/// Table of sample numbers.
///
/// Note that field sizes are not specifed in Apple's QuickTime documentation,
/// and that 2 x `u16` is pure guess based on example file where
/// number of entries is 88 over 352 bytes = 4 bytes / sync sample,
/// possibly meaning two 16-bit integers (or one u8 + [u8; 3], or...).
#[derive(Debug, Default, BinRead)]
pub struct SyncSample {
    pub(crate) entry_number: u16,
    pub(crate) sample: u16
}

impl SyncSample {
    pub fn entry_number(&self) -> u16 {
        self.entry_number
    }

    pub fn sample(&self) -> u16 {
        self.sample
    }
}