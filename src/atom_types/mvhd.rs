//! Movie header atom (`mvhd`).
//! 
//! Location: `moov/mvhd`
//! 
//! See: <https://developer.apple.com/documentation/quicktime-file-format/movie_header_atom>

use binrw::BinRead;
use time::{Duration, ext::NumericalDuration};

use crate::mp4_time_zero;

/// Movie header atom (`mvhd`).
/// 
/// Location: `moov/mvhd`
/// 
/// See: <https://developer.apple.com/documentation/quicktime-file-format/movie_header_atom>
#[derive(Debug, BinRead)]
#[br(big)]
pub struct Mvhd {
    _version: u8,
    _flags: [u8; 3],
    /// Seconds since midnight, 1904-01-01 UTC
    pub creation_time: u32, // should be UTC
    /// Seconds since midnight, 1904-01-01 UTC
    pub modification_time: u32, // should be UTC
    /// Number of time units that pass in one second
    pub time_scale: u32,
    /// Unscaled duration. I.e. "time units"
    /// that require dividing by time scale
    /// to derive a value in seconds.
    /// 
    /// Corresponds to the longest track.
    pub duration: u32,
    /// Fixed point number (16.16)
    /// representing preferred play rate
    /// (1.0 = normal playback).
    pub preferred_rate: u32,
    /// Fixed point number (8.8)
    /// representing preferred volume
    /// (1.0 = full volume).
    pub preferred_volume: u16,
    pub reserved: [u8; 10],
    pub matrix: [u8; 36], // row-major matrix
    pub preview_time: u32,
    pub preview_duration: u32,
    pub poster_time: u32,
    pub selection_time: u32,
    pub selection_duration: u32,
    pub current_time: u32,
    pub next_track_id: u32,
}

impl Mvhd {
    /// Creation time as UTC datetime.
    /// May default to MP4 default time
    /// `1904-01-01 00:00:00` depending on device and settings.
    pub fn creation_time(&self) -> time::PrimitiveDateTime {
        mp4_time_zero() + Duration::seconds(self.creation_time as i64)
    }

    /// Modification time as UTC datetime.
    pub fn modification_time(&self) -> time::PrimitiveDateTime {
        mp4_time_zero() + Duration::seconds(self.modification_time as i64)
    }

    /// Duration of the longest track in seconds.
    pub fn duration(&self) -> Duration {
        (self.duration as f64 / self.time_scale as f64).seconds()
    }
}