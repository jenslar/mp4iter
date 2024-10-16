//! Iterate over MP4 atoms and find specific atoms via FourCC.
//! Does not and will not support any kind of video de/encoding.
//!
//! The implementation was mostly done with help from
//! <https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFPreface/qtffPreface.html>
//! (despite the warning on the front page above).
//!
//! ```rs
//! use mp4iter::Mp4;
//! use std::path::Path;
//!
//! fn main() -> std::io::Result<()> {
//!     let mp4 = Mp4::new(Path::new("VIDEO.MP4"))?;
//!
//!     for atom_header in mp4.into_iter() {
//!         println!("{atom_header:?}")
//!     }
//!
//!     // Derives duration for MP4 for longest track.
//!     println!("{:?}", mp4.duration());
//!
//!     // Extracts offsets for GoPro GPMF telemetry (handlre name 'GoPro MET')
//!     println!("{:#?}", mp4.offsets("GoPro MET"));
//!
//!     Ok(())
//! }
//! ```

pub mod mp4;
pub mod fourcc;
pub mod offset;
pub mod atom;
pub mod atom_types;
pub mod consts;
pub mod support;
pub mod track;
pub mod errors;
pub mod tests;

// Internal reader
pub(crate) mod reader;
pub(crate) use reader::{Mp4Reader, ReadOption, TargetReader};

pub mod iterator;

pub use mp4::Mp4;
pub use fourcc::FourCC;
pub use offset::{Offset, Offsets};
pub use atom::{Atom, AtomHeader};
pub use atom_types::{
    Co64,
    Dref,
    Elst,
    Ftyp,
    Sdtp,
    Smhd,
    Stts,
    Stss,
    Stsz,
    Stco,
    Hdlr,
    Tkhd,
    Mdhd,
    Mvhd,
    Stsd,
    Tmcd,
    Vmhd,
    AudioFormat, // stsd component
    VideoFormat, // stsd component
    SampleDescription, // stsd component
};
pub use consts::{CONTAINER, mp4_time_zero};
pub use errors::Mp4Error;
