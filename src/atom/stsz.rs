//! Sample-to-size (in bytes) atom (`stsz`).

use binrw::BinRead;

/// Sample-to-size (in bytes) atom. FourCC `stsz`.
#[derive(Debug, BinRead)]
#[br(big)]
pub struct Stsz {
    _version: u8,
    _flags: [u8; 3],
    /// Sample size.
    /// If 0 `no_of_entries` contains
    /// the number of u32 values that should be read,
    /// else all sample sizes should have this value.
    _sample_size: u32,
    _no_of_entries: u32,
    #[br(count = _no_of_entries)]
    #[br(if(_sample_size == 0, vec![_sample_size; _no_of_entries as usize]))] // seems to work
    sizes: Vec<u32>
}

impl Stsz {
    pub fn len(&self) -> usize {
        self.sizes.len()
    }

    pub fn expand(&self) -> Vec<u32> {
        self.sizes.to_owned()
    }
}