//! Chunk offset atom for file sizes below the 32bit limit (`stco`).
//!
//! Location: `moov/trak[multiple]/mdia/minf/stbl/stco`
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/chunk_offset_atom>

use binrw::BinRead;

/// Chunk offset atom for file sizes below the 32bit limit (`stco`).
///
/// Location: `moov/trak[multiple]/mdia/minf/stbl/stco`
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/chunk_offset_atom>
#[derive(Debug, Default, BinRead, Clone)]
pub struct Stco {
    pub(crate) version: u8,
    pub(crate) flags: [u8; 3],
    #[br(big)]
    pub(crate) no_of_entries: u32,
    /// Chunk offset table consisting of an array of offset values.
    #[br(big, count = no_of_entries)]
    pub(crate) offsets: Vec<u32>
}

impl Stco {
    pub fn len(&self) -> usize {
        self.no_of_entries as usize
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn flags(&self) -> &[u8] {
        self.flags.as_slice()
    }

    pub fn no_of_entries(&self) -> u32 {
        self.no_of_entries
    }

    pub fn offsets(&self) -> Vec<u32> {
        self.offsets.to_owned()
    }

    /// Returns chunk offset with specified ID.
    pub fn get(&self, chunk_id: usize) -> Option<&u32> {
        self.offsets.get(chunk_id)
    }
}
