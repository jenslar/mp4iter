use std::ops::{Range, RangeBounds};

use crate::{FourCC, Mp4Error, CONTAINER};

/// Atom header.
/// 8 or 16 bytes in MP4, depending on whether
/// 32 or 64-bit sized.
///
/// ```ignore
/// | [X X X X] [Y Y Y Y] [Z Z Z Z Z Z Z Z] |
///    |         |         |
///    |         |         64bit size (optional, only if 32 bit size == 1)
///    |         FourCC
///    32bit size
/// ```
#[derive(Debug, Clone, Default)]
pub struct AtomHeader {
    /// Total atom size in bytes including 8/16 byte header.
    pub(crate) atom_size: u64,
    /// FourCC
    pub(crate) name: FourCC,
    /// Absolute byte offset for start of atom in MP4,
    /// i.e. byte offset for its header,
    /// starting with 32-bit size.
    pub(crate) offset: u64,
    /// Relative offset to next atom
    /// counting from start of data load.
    pub(crate) next: u64,
    /// Set to `true` if atom size specified
    /// in 64 bit area. E.g. insta360 seems to
    /// specify all sizes as 64 bit regardless
    /// of actual atom size. I.e. 32bit size
    /// is set to `1`. But this info can not
    /// be derived post-parse.
    pub(crate) size_64bit: bool
}

impl AtomHeader {
    /// Convenience method to check whether atom at current offset is
    /// a container or not.
    pub fn is_container(&self) -> bool {
        CONTAINER.contains(&self.name.to_str())
    }

    pub fn start(&self) -> u64 {
        self.offset
    }

    pub fn end(&self) -> u64 {
        self.offset + self.atom_size
    }

    pub fn atom_size(&self) -> u64 {
        self.atom_size
    }

    pub fn name(&self) -> &FourCC {
        &self.name
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Determine header size in bytes in MP4.
    /// Returns 8 or 16 bytes.
    /// If a 64-bit atom header, size is increased
    /// 8 bytes to adjust for 64 bit value being read
    /// after FourCC.
    pub fn header_size(&self) -> u8 {
        // Size check will not work for cameras
        // that consistently store atoms with 64-bit size...
        // match self.atom_size > u32::MAX as u64 {
        //     true => 16,
        //     false => 8,
        // }
        // ...so instead a bool is stored to
        // be able to derive header size.
        // it would otherwise be impossible to
        // determine correct byte offsets without
        // re-reading the MP4.
        match self.size_64bit {
            true => 16,
            false => 8,
        }
    }

    // pub fn rem(&self, pos: u64) {

    // }

    /// Data load absolute offset,
    /// i.e. position after header
    /// adjusted for optional 64bit size value.
    pub fn data_offset(&self) -> u64 {
        self.offset + self.header_size() as u64
    }

    /// Size of data load, adjusted for header size
    /// (excludes header size).
    pub fn data_size(&self) -> u64 {
        self.atom_size - self.header_size() as u64
    }

    /// Relative offset to next atom,
    /// counted from start of data load,
    /// exluding the header itself.
    pub fn offset_next_rel(&self) -> u64 {
        match self.is_container() {
            true => 0,
            false => self.atom_size - self.header_size() as u64,
        }
    }

    /// Absolute offset to next atom.
    pub fn offset_next_abs(&self) -> u64 {
        self.offset + self.atom_size
    }

    /// Returns start, end offset range for atom.
    pub fn bounds(&self) -> Range<u64> {
        self.offset .. self.offset_next_abs()
    }

    pub fn in_bounds(&self, range: &Range<u64>) -> bool {
        self.bounds().contains(&range.start) && self.bounds().contains(&range.end)
    }

    /// Returns `true` is absolute offset `pos`
    /// is contained within atom span.
    ///
    /// Inclusive lower bound, exclusive upper bound
    /// when checking against atom start/end offset.
    /// I.e `start_of_atom <= pos < end_of_atom`,
    /// meaning if the upper bound is at an atom boundary,
    /// the position for the atom that starts at that offset
    /// will be returned.
    pub fn contains(&self, pos: u64) -> bool {
        self.offset <= pos && self.offset_next_abs() > pos
    }
}
