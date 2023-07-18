use crate::{CONTAINER, FourCC, Mp4Error, Mp4};

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
#[derive(Debug, Clone, Default)]
pub struct AtomHeader {
    /// Total size in bytes including 8/16 byte header.
    pub size: u64,
    /// FourCC
    pub name: FourCC,
    /// Absolute byte offset for start of atom in MP4,
    /// i.e. byte offset where atom size is specified.
    pub offset: u64,
    // Relative offset to next atom
    // counting from start of data load.
    pub next: u64,
}

impl AtomHeader {
    pub(crate) fn new(mp4: &mut Mp4) -> Result<Self, Mp4Error> {
        // Get offset for header
        let pos = mp4.pos()?;

        let mut hdr = Self::default();

        hdr.offset = pos;

        // Read 32bit atom size
        hdr.size = mp4.read_be::<u32>()? as u64;

        let string = mp4.read_string(4)?;
        hdr.name = FourCC::from_str(&string);

        // Check if atom size is 64bit and read the 8 bytes
        // following directly after FourCC as new size if so
        if hdr.size == 1 {
            hdr.size = mp4.read_be::<u64>()?;
        }

        hdr.next = match CONTAINER.contains(&hdr.name.to_str()) {
            true => 0,
            false => hdr.size - hdr.header_size() as u64
        };

        Ok(hdr)
    }

    /// Convenience method to check whether atom at current offset is
    /// a container or not.
    pub fn is_container(&self) -> bool {
        CONTAINER.contains(&self.name.to_str())
    }

    /// Determine header size in bytes in MP4.
    /// Returns 8 or 16 bytes.
    /// If a 64-bit atom header, size is increased
    /// 8 bytes to adjust for 64 bit value being read
    /// after FourCC.
    pub fn header_size(&self) -> u8 {
        match self.size > u32::MAX as u64 {
            true => 16,
            false => 8,
        }
    }

    /// Data load absolute offset
    /// (excludes header).
    pub fn data_offset(&self) -> u64 {
        self.offset + self.header_size() as u64
    }

    /// Size of data load, adjusted for header size
    /// (excludes header size).
    pub fn data_size(&self) -> u64 {
        self.size - self.header_size() as u64
    }

    /// Relative offset to next atom,
    /// counted from start of data load,
    /// exluding the header itself.
    pub fn offset_next_rel(&self) -> u64 {
        match self.is_container() {
            true => 0,
            false => self.size - self.header_size() as u64,
        }
    }

    /// Absolute offset to next atom,
    /// counted from start of data load,
    /// excluding the header itself.
    pub fn offset_next_abs(&self) -> u64 {
        self.offset + self.size
    }
}