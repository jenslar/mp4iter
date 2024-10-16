//! Time code entry (`tmcd`). Part of the sample description atom (`stsd`).
//!
//! > Note: Distinguish from `tmcd` with the same FourCC located at `moov/tref/tmcd`.
//!
//! `tmcd` is not an atom in the main tree, but part of one. Contains
//! start time information that can be used to e.g. sort clips chronologically
//! if these belong to the same recording session (if a camera split the session into clips).
//!
//! Location: `moov/trak[multiple]/mdia/minf/stbl/stsd[tmcd]`

use binrw::BinRead;
use time::{Time, ext::NumericalDuration};

use crate::{Offset, Offsets};

/// Time code entry (`tmcd`). Part of the sample description atom (`stsd`).
///
/// > Note: Distinguish from `tmcd` with the same FourCC located at `moov/tref/tmcd`.
///
/// `tmcd` is not an atom in the main tree, but part of one. Contains
/// start time information that can be used to e.g. sort clips chronologically
/// if these belong to the same recording session (if a camera split the session into clips).
///
/// Location: `moov/trak[multiple]/mdia/minf/stbl/stsd[tmcd]`
#[derive(Debug, Default, BinRead)]
#[br(big)]
pub struct Tmcd {
    _reserved1: u32,
    _flags: u32,
    pub(crate) time_scale: u32,
    pub(crate) frame_duration: u32,
    pub(crate) number_of_frames: u8,
    /// Should be set to 0. Currently unused.
    pub(crate) _reserved2: u8,
    #[br(ignore)]
    // pub(crate) offsets: Vec<Offset>
    pub(crate) offsets: Offsets
}

impl Tmcd {
    pub fn time_scale(&self) -> u32 {
        self.time_scale
    }

    pub fn frame_duration(&self) -> u32 {
        self.frame_duration
    }

    pub fn number_of_frames(&self) -> u8 {
        self.number_of_frames
    }

    pub fn offsets(&self) -> impl Iterator<Item = &Offset> {
        self.offsets.iter()
    }

    /// Returns start time counted from midnight.
    /// May not correspond to actual start time if device
    /// clock is not set correctly, but can still be used
    /// for sorting clips/splits belonging to the same recording session.
    pub fn seconds_since_midnight(&self, value: u32) -> Time {
        let t = time::Time::MIDNIGHT;

        t + (value as f64 / self.time_scale as f64).seconds()
    }
}
