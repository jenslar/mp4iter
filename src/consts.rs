use time::{self, PrimitiveDateTime, Month};

/// FourCC:s for known "container" atoms.
/// These are nested and contains more atoms,
/// within its specified, total size.
///
/// Only container atoms in the main MP4 tree are listed.
///
/// - `moov`: offset tables, timing, metadata, telemetry
/// - `trak`: moov.trak (multiple)
/// - `tref`: moov.trak.tref
/// - `edts`: moov.trak.edts
/// - `mdia`: moov.trak.mdia
/// - `minf`: moov.trak.mdia.minf
/// - `dinf`: moov.trak.mdia.minf.dinf
/// - `stbl`: moov.trak.mdia.minf.stbl, contains timing (stts), offsets (stco)
/// - `udta`: moov.udta, may contain custom data, specific to the device
pub const CONTAINER: [&'static str; 9] = [
    "moov",
    "trak",
    "tref",
    "edts",
    "mdia",
    "minf",
    "dinf",
    "stbl",
    "udta",
];

/// Time zero for MP4 containers. Midnight January 1, 1904.
pub fn mp4_time_zero() -> PrimitiveDateTime {
    time::Date::from_calendar_date(1904, Month::January, 1).unwrap()
        .with_hms_milli(0, 0, 0, 0).unwrap()
}