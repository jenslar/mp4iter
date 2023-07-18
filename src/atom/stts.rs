//! Time-to-sample atom (`stts`). Time-to-sample table only.

use binrw::BinRead;

#[derive(Debug, BinRead)]
pub struct TimeToSample {
    #[br(big)]
    sample_count: u32,
    #[br(big)]
    sample_duration: u32,
}

/// Time-to-sample atom (`stts`).
/// 
/// See <https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html>
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
    /// Expand time to sample table into
    /// discrete list of values.
    pub fn expand(&self) -> Vec<u32> {
        self.table.iter()
            .flat_map(|t| vec![t.sample_duration; t.sample_count as usize])
            .collect()
    }

    pub fn len(&self) -> usize {
        self.table.iter()
            .map(|t| t.sample_count as usize)
            .sum()
    }

    pub fn table(&self) -> Vec<(u32, u32)> {
        self.table.iter()
            .map(|t| (t.sample_count, t.sample_duration))
            .collect()
    }
}
