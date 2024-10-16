//! Track header atom (`tkhd`).
//!
//! Location: `moov/trak[multiple]/tkhd`
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/track_header_atom>

use binrw::BinRead;
use time::{Duration, PrimitiveDateTime};

/// Track header atom (`tkhd`).
///
/// Location: `moov/trak[multiple]/tkhd`
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/track_header_atom>
#[derive(Debug, BinRead)]
#[br(big)]
pub struct Tkhd {
    _version: u8,
    _flags: [u8; 3],
    /// Indicates the creation calendar date and time for the track header.
    /// Represents the calendar date and time in seconds since midnight,
    /// January 1, 1904, preferably using coordinated universal time (UTC).
    pub(crate) creation_time: u32,
    /// Indicates the last change date for the track header.
    /// Represents the calendar date and time in seconds since midnight,
    /// January 1, 1904, preferably using coordinated universal time (UTC).
    pub(crate) modification_time: u32,
    /// Uniquely identifies the track.
    /// Value 0 cannot be used.
    pub(crate) track_id: u32,
    /// Reserved. Should be set to 0.
    _reserved1: [u8; 4],
    /// Indicates the duration of this track,
    /// in the movie’s time coordinate system.
    /// Derived from the track’s edits.
    /// The value of this field is equal to the sum of the durations
    /// of all of the track’s edits.
    /// If there is no edit list, then the duration is the sum of the sample durations,
    /// converted into the movie timescale.
    pub(crate) duration: u32,
    _reserved2: [u8; 8],
    /// This track’s spatial priority in its movie.
    layer: u16,
    /// Identifies a collection of movie tracks that contain alternate data for one another.
    pub(crate) alternate_group: u16,
    /// 16-bit fixed-point value that indicates how loudly to play this track’s sound.
    /// 1.0 indicates normal volume.
    pub(crate) volume: u16,
    /// Reserved. Should be set to 0.
    _reserved3: [u8; 2],
    /// The matrix structure associated with this track.
    pub(crate) matrix_structure: [u8; 36],
    /// 32-bit fixed-point number
    /// that specifies the width of this track in pixels.
    pub(crate) track_width: u32,
    /// 32-bit fixed-point number
    /// that specifies the height of this track in pixels.
    pub(crate) track_height: u32,
}

impl Tkhd {
    pub fn track_id(&self) -> u32 {
        self.track_id
    }

    /// Track width in pixels (video tracks only).
    pub fn width(&self) -> f64 {
        self.track_width as f64 / 2_u32.pow(16) as f64
    }

    /// Track height in pixels (video tracks only).
    pub fn height(&self) -> f64 {
        self.track_height as f64 / 2_u32.pow(16) as f64
    }

    pub fn layer(&self) -> u16 {
        self.layer
    }

    pub fn alternate_group(&self) -> u16 {
        self.alternate_group
    }

    /// Volume "level", indicating if adjustments to volume
    /// is suggested. 1.0 is normal volume.
    pub fn volume(&self) -> f64 {
        self.volume as f64 / 2_u16.pow(8) as f64
    }

    /// This track's unscaled duration.
    pub fn duration(&self) -> u32 {
        self.duration
    }

    /// This track's duration in seconds.
    pub fn duration_sec(&self, time_scale: u32) -> f64 {
        self.duration as f64 / time_scale as f64
    }

    pub fn matrix_structure(&self) -> &[u8] {
        self.matrix_structure.as_slice()
    }

    /// Creation time as duration.
    fn creation_duration(&self) -> Duration {
        Duration::seconds(self.creation_time as i64)
    }

    /// Creation datetime for this track.
    pub fn creation_time(&self) -> PrimitiveDateTime {
        crate::consts::mp4_time_zero() + self.creation_duration()
    }

    /// Modification time as duration.
    fn modification_duration(&self) -> Duration {
        Duration::seconds(self.modification_time as i64)
    }

    /// Modification datetime for this track.
    pub fn modification_time(&self) -> PrimitiveDateTime {
        crate::consts::mp4_time_zero() + self.modification_duration()
    }
}