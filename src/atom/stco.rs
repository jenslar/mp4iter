//! Sample-to-offset atom for file sizes below the 32bit limit (`stco`) and above (`co64`).

use binrw::BinRead;

/// Sample-to-offset atom.
#[derive(Debug, Default, BinRead)]
pub struct Stco {
    version: u8,
    flags: [u8; 3],
    #[br(big)]
    no_of_entries: u32,
    #[br(big, count = no_of_entries)]
    pub offsets: Vec<u32>
}

impl Stco {
    pub fn len(&self) -> usize {
        self.no_of_entries as usize
    }

    pub fn expand(&self) -> Vec<u32> {
        self.offsets.to_owned()
    }
}

/// 64bit sample-to-offset atom.
#[derive(Debug, Default, BinRead)]
pub struct Co64 {
    version: u8,
    flags: [u8; 3],
    #[br(big)]
    no_of_entries: u32,
    #[br(big, count = no_of_entries)]
    pub offsets: Vec<u64>
}

impl Co64 {
    pub fn len(&self) -> usize {
        self.no_of_entries as usize
    }

    pub fn expand(&self) -> Vec<u64> {
        self.offsets.to_owned()
    }
}

impl From<Stco> for Co64 {
    fn from(value: Stco) -> Self {
        let mut co64 = Self::default();
        co64.version = value.version;
        co64.flags = value.flags;
        co64.no_of_entries = value.no_of_entries;
        co64.offsets = value.offsets.iter().map(|n| *n as u64).collect();

        co64
    }
}