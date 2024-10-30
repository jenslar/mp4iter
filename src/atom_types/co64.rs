//! Chunk to offset atom for file sizes above the 32bit limit (`co64`).
//! The 64-bit equivalent of the `stco` atom.
//!
//! Path: `moov/trak[multiple]/mdia/minf/stbl/co64`

use binrw::BinRead;

use crate::Stco;

/// Chunk to offset atom for file sizes above the 32bit limit (`co64`).
/// The 64-bit equivalent of the `stco` atom.
///
/// Path: `moov/trak/mdia/minf/stbl/co64`
#[derive(Debug, Default, BinRead, Clone)]
pub struct Co64 {
    _version: u8,
    _flags: [u8; 3],
    #[br(big)]
    no_of_entries: u32,
    #[br(big, count = no_of_entries)]
    offsets: Vec<u64>
}

impl Co64 {
    /// Returns number of chunks.
    /// (each chunk correspinds to one or more samples).
    pub fn len(&self) -> usize {
        self.no_of_entries as usize
    }

    /// Returns chunk byte offsets.
    /// (each chunk correspinds to one or more samples).
    pub fn offsets(&self) -> &[u64] {
        &self.offsets
    }

    pub fn from_stco(stco: Stco) -> Self {
        Self::from(stco)
    }
}

impl From<Stco> for Co64 {
    fn from(value: Stco) -> Self {
        Self {
            _version: value.version,
            _flags: value.flags,
            no_of_entries: value.no_of_entries,
            offsets: value.offsets
                .iter()
                .map(|n| *n as u64)
                .collect()
        }
    }
}
