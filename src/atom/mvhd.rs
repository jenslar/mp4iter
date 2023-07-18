use binrw::BinRead;
use time::{Duration, ext::NumericalDuration};

use crate::mp4_time_zero;

/// Movie header atom (`mvhd`).
/// 
/// See: <https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-BBCGFGJG>
#[derive(Debug, BinRead)]
#[br(big)]
pub struct Mvhd {
    _version: u8,
    _flags: [u8; 3],
    pub creation_time: u32, // should be UTC
    pub modification_time: u32, // should be UTC
    pub time_scale: u32, // 4 bytes, supposedly int, but no data type in apple docs
    pub duration: u32, // 4 bytes, supposedly int, but no data type in apple docs
    pub preferred_rate: u32, // actually fixed point number, "float", 1.0 normal rate
    pub preferred_volume: u16, // actually fixed point number, "float", 1.0 normal rate
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
    pub fn creation_time(&self) -> time::PrimitiveDateTime {
        mp4_time_zero() + Duration::seconds(self.creation_time as i64)
    }

    /// Modification time as UTC datetime.
    pub fn modification_time(&self) -> time::PrimitiveDateTime {
        mp4_time_zero() + Duration::seconds(self.modification_time as i64)
    }

    /// Duration of the longest track.
    pub fn duration(&self) -> Duration {
        (self.duration as f64 / self.time_scale as f64).seconds()
    }
}