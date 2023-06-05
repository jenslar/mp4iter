use crate::{CONTAINER, FourCC};

/// Atom header.
/// 8 or 16 bytes in MP4, depending on whether
/// 32 or 64-bit sized.
/// ```
/// | [X X X X] [Y Y Y Y] [Z Z Z Z Z Z Z Z] |
///    |         |         |
///    |         |         64bit size (optional, only if 32 bit size == 1)
///    |         FourCC
///    32bit size
/// ```
#[derive(Debug, Clone)]
pub struct AtomHeader {
    /// Total size in bytes including 8/16 byte header.
    pub size: u64,
    /// FourCC
    pub name: FourCC,
    /// Byte offset for start of atom in MP4,
    /// i.e. byte offset where atom size is specified.
    pub offset: u64,
}

impl AtomHeader {
    /// Convenience method to check whether atom at current offset is
    /// a container or not.
    pub fn is_container(&self) -> bool {
        CONTAINER.contains(&self.name.to_str())
    }

    /// Determine header size in bytes in MP4.
    pub fn header_size(&self) -> u8 {
        match self.size > u32::MAX as u64 {
            true => 16,
            false => 8,
        }
    }

    /// Data offset, adjusted for header size.
    pub fn data_offset(&self) -> u64 {
        self.offset + self.header_size() as u64
    }

    /// Size of data load, adjusted for header size.
    /// (Not including header size)
    pub fn data_size(&self) -> u64 {
        self.size - self.header_size() as u64
    }

    /// Relative offset to next atom,
    /// adjusted for header size.
    pub fn offset_next(&self) -> u64 {
        // let mut adjust = 8;
        // if self.size > u32::MAX as u64 {
        //     adjust = 16;
        // }
        match self.is_container() {
            true => 0,
            false => self.size - self.header_size() as u64
        }
    }
}