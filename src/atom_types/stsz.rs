//! Sample size atom (`stsz`).
//!
//! Location: `moov/trak[multiple]/mdia/minf/stbl/stsz`
//!
//! Note that `stsz` lists sample size not chunk size.
//! `stco` or `co64` list chunk offsets, not offsets to individual samples.
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/sample_size_atom>

use binrw::BinRead;

/// Sample size atom (`stsz`).
///
/// Location: `moov/trak[multiple]/mdia/minf/stbl/stsz`
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/sample_size_atom>
#[derive(Debug, BinRead)]
#[br(big)]
pub struct Stsz {
    _version: u8,
    _flags: [u8; 3],
    /// Sample size.
    /// If 0 `no_of_entries` contains
    /// the number of u32 values that should be read,
    /// else all sample sizes should have this value.
    pub(crate) sample_size: u32,
    _no_of_entries: u32,
    #[br(count = _no_of_entries)]
    #[br(if(sample_size == 0, vec![sample_size; _no_of_entries as usize]))] // seems to work
    pub(crate) sizes: Vec<u32>
}

impl Stsz {
    pub fn len(&self) -> usize {
        self.sizes.len()
    }

    pub fn sample_size(&self) -> u32 {
        self.sample_size
    }

    /// Returns discrete list of sample sizes in bytes.
    pub fn sizes(&self) -> &[u32] {
        &self.sizes
    }
}
