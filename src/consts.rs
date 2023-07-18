//! Main "container" atoms, i.e. atoms that contain more atoms.
//! 
//! Note that `mp4iter` only supports container atoms where the child atom/s
//! follow immediately after the parent header.
//! Try [AtomicParsley](https://atomicparsley.sourceforge.net)
//! for much better support in this regard.

use time::{self, PrimitiveDateTime, Month};

/// FourCC:s for known "container" atoms.
/// If the atom is a "container",
/// it's nested and contains more atoms,
/// within its specified, total size.
/// - `moov`: offset tables, timing, metadata, telemetry
/// - `trak`: moov.trak
/// - `tref`: moov.trak.tref
/// - `edts`: moov.trak.edts
/// - `mdia`: moov.trak.mdia
/// - `minf`: moov.trak.mdia.minf
/// - `dinf`: moov.trak.mdia.minf.dinf
/// - `stbl`: moov.trak.mdia.minf.stbl, contains timing (stts), offsets (stco)
pub const CONTAINER: [&'static str; 8] = [
    "moov",
    "trak",
    "tref",
    "edts",
    "mdia",
    "minf",
    "dinf",
    "stbl",
];

/// Containers that hold further atoms-like structure,
/// but are not branched according to the general MP4 structure.
pub const SUB_CONTAINER: [&'static str; 1] = [
    "hdlr", // moov.trak.mdia.hdlr -> mhlr
];

/// Time zero for MP4 containers. January 1, 1904.
pub fn mp4_time_zero() -> PrimitiveDateTime {
    time::Date::from_calendar_date(1904, Month::January, 1).unwrap()
        .with_hms_milli(0, 0, 0, 0).unwrap()
}