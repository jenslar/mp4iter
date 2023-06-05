//! Time code entry (`tmcd`) in the Sample Description atom (`stsd`).
//! I.e. `tmcd` not an actual atom, but part of one. It contains
//! start time information that can be used to e.g. sort clips
//! from the same chronologically (making it independent of filename),
//! when the recording device split these up.

use binread::BinRead;
use time::{Time, ext::NumericalDuration};

use crate::Offset;

/// Time code entry in the Sample Description atom (`stsd`).
/// 
/// `tmcd` contains start time, which can be used to sort e.g. GoPro clips
/// chronologically when the camera splits the video during a longer recording session.
/// For sorting, start time doesn't have to be correct as long as each clip's start time
/// increments on the previus one if part of the same recording session.
/// 
/// See: <https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap3/qtff3.html#//apple_ref/doc/uid/TP40000939-CH205-91003>
#[derive(Debug, Default, BinRead)]
pub struct Tmcd {
    #[binread(big)]
    pub data_reference_index: u16,
    /// Should be set to 0. Currently unused.
    #[binread(big)]
    pub(crate) _reserved1: u32,
    #[binread(big)]
    pub flags: u32,
    #[binread(big)]
    pub time_scale: u32,
    #[binread(big)]
    pub frame_duration: u32,
    pub number_of_frames: u8,
    /// Should be set to 0. Currently unused.
    pub(crate) _reserved2: u8,
    /// Offsets in `mdat`
    #[binread(ignore)]
    pub offsets: Vec<Offset>
}

impl Tmcd {
    /// Returns start time counted from midnight.
    /// May not correspond to actual start time if device
    /// clock is not set correctly, but can still be used
    /// for sorting clips/splits belonging to the same recording session.
    pub fn seconds_since_midnight(&self, value: u32) -> Time {
        let t = time::Time::MIDNIGHT;

        return t + (value as f64 / self.time_scale as f64).seconds()
    }
}