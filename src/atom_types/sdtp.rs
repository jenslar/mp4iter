//! Sample dependency flags atom (`sdtp`)
//!
//! Location:
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/sample_dependency_flags_atom>

use binrw::BinRead;

/// Sample dependency flags atom (`sdtp`).
///
/// Note that number of entries is derived from stsz atoms entry number,
/// since sdtp precedes stsz, atom size is used to derive this value instead.
/// If necessary, verify with the associated stsz (the one in the same track/`trak`)
/// that follows the sdtp atom.
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/sample_dependency_flags_atom>
#[derive(Debug, Default, BinRead)]
#[br(big, import {data_size: u32})] // attempt to use size for deriving sample flags table size instead.
pub struct Sdtp {
    _version: u8,
    _flags: [u8; 3],
    /// Sample dependency flags table.
    /// A table of 8-bit values indicating the sample flag settings.
    #[br(count = (data_size - 4) / 2)]
    pub(crate) sample_flags_table: Vec<SampleFlagsTable>
}

impl Sdtp {
    pub fn sample_flags_table(&self) -> &[SampleFlagsTable] {
        &self.sample_flags_table
    }
}

#[derive(Debug, Default, BinRead)]
pub struct SampleFlagsTable {
    pub(crate) sample_dependency_flag: u8,
    pub(crate) sample: u8,
}

impl SampleFlagsTable {
    pub fn sample_dependency_flag(&self) -> u8 {
        self.sample_dependency_flag
    }

    pub fn sample(&self) -> u8 {
        self.sample
    }
}