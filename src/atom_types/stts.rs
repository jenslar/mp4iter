//! Time-to-sample atom (`stts`).
//!
//! Location: `moov/trak[multiple]/mdia/minf/stbl/stts`
//!
//! See <https://developer.apple.com/documentation/quicktime-file-format/time-to-sample_atom>

use binrw::BinRead;

#[derive(Debug, BinRead)]
pub struct TimeToSample {
    #[br(big)]
    pub(crate) sample_count: u32,
    #[br(big)]
    pub(crate) sample_duration: u32,
}

impl TimeToSample {
    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn sample_duration(&self) -> u32 {
        self.sample_duration
    }
}

/// Time to sample atom (`stts`).
///
/// Path: `moov/trak[multiple]/mdia/minf/stbl/stts`
///
/// See <https://developer.apple.com/documentation/quicktime-file-format/time-to-sample_atom>
#[derive(Debug, BinRead)]
#[br(big)]
pub struct Stts {
    _version: u8,
    _flags: [u8; 3],
    _no_of_entries: u32,
    #[br(count = _no_of_entries)]
    table: Vec<TimeToSample>
}

impl Stts {
    /// Returns total number of samples.
    ///
    /// If an entry lists a duration for four samples,
    /// it counts as four entries towards the total.
    pub fn len(&self) -> usize {
        self.table.iter()
            .map(|t| t.sample_count as usize)
            .sum()
    }

    /// Returns the time to sample table as tuples,
    /// `(SAMPLE_COUNT, SAMPLE_DURATION)`.
    pub fn table(&self) -> Vec<(u32, u32)> {
        self.table.iter()
            .map(|t| (t.sample_count, t.sample_duration))
            .collect()
    }

    /// Returns discrete list of unscaled duration values.
    pub fn durations(&self) -> Vec<u32> {
        self.table.iter()
            .flat_map(|t| vec![t.sample_duration; t.sample_count as usize])
            .collect()
    }

    /// Returns unscaled sample duration, when the time to sample
    /// table only has a single entry. Returns `None` if it is empty,
    /// or contains multiple entries.
    pub fn duration(&self) -> Option<u32> {
        if self.table.len() == 1 {
            return Some(self.table.first()?.sample_duration)
        }
        None
    }

    pub fn duration_sum(&self) -> u32 { // return u64 instead?
        self.table.iter()
            .map(|t| t.sample_duration * t.sample_count) // as u64?
            .sum()
    }

    pub fn sample_sum(&self) -> u32 { // return u64 instead?
        self.table.iter()
            .map(|t| t.sample_count) // as u64?
            .sum()
    }
}
