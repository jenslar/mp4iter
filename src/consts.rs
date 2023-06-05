//! Main "container" atoms, i.e. atoms that contain more atoms.
//! 
//! Note that `mp4iter` only supports container atoms where the child atom/s
//! follow immediately after the parent header. Try [AtomicParsley](https://atomicparsley.sourceforge.net)
//! for much better support in this regard.

use time::{self, PrimitiveDateTime, Month};

// If the atom is a "container",
// it's nested and contains more atoms,
// within its specified, total size.
pub const CONTAINER: [&'static str; 9] = [
    "moov", // offset tables, timing, metadata, telemetry
    "udta", // moov.udta, custom user data
    "trak", // moov.trak
    "tref", // moov.trak.tref
    "edts", // moov.trak.edts
    "mdia", // moov.trak.mdia
    "minf", // moov.trak.mdia.minf
    "dinf", // moov.trak.mdia.minf.dinf
    "stbl", // moov.trak.mdia.minf.stbl, contains timing (stts), offsets (stco)
    // both dref and stbl start with 8 bytes after fourcc that if interpreted
    // as two u32 parse as 0 and 1 respectively THEN followed by normal, embedded atom size + fourcc
    // "dref", // moov.trak.mdia.minf.dinf.dref
    // "stsd", // moov.trak.mdia.minf.stbl.stsd
];

/// Time zero for MP4 containers. January 1, 1904.
pub fn time_zero() -> PrimitiveDateTime {
    time::Date::from_calendar_date(1904, Month::January, 1).unwrap()
        .with_hms_milli(0, 0, 0, 0).unwrap()
}