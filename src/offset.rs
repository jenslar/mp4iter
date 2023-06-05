//! MP4 byte offset (derived from `stco` atom), size in bytes (derived from `stsz` atom),
//! and duration (derived from `stts`atom) in milliseconds
//! for a chunk of data.

/// MP4 byte offset (from `stco` atom), size in bytes (from `stsz` atom),
/// and duration (from `stts`atom) in milliseconds
/// for a chunk of data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offset {
    /// Offset in bytes from start of file.
    // pub position: u32,
    pub position: u64,
    /// Size of GPMF-chunk in bytes.
    pub size: u32,
    /// Duration in milliseconds,
    /// equal to the GPMF-chunk's "duration"
    /// within the `mdat` atom.
    pub duration: u32
}
impl Offset {
    // pub fn new(position: u43, size: u32, duration: u32) -> Self {
    pub fn new(position: u64, size: u32, duration: u32) -> Self {
        Self{position, size, duration}
    }
}
