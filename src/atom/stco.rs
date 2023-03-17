//! Sample-to-offset atom for file sizes under the 32bit limit (`stco`) and above (`co64`).

use std::ops::Range;

/// Sample-to-offset (in bytes from start of MP4) atom
/// for file sizes under the 32bit limit.
/// Each value represents a byte offset for a data chunk
/// in the corresponding track.
#[derive(Debug, Default)]
pub struct Stco(Vec<u32>);
impl Stco {
    pub fn new(values: Vec<u32>) -> Self {
        Self(values)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn iter(&self) -> impl Iterator<Item = &u32> {
        self.0.iter()
    }
    /// Panics if out of bounds.
    pub fn slice(&self, range: Range<usize>) -> &[u32] {
        &self.0[range]
    }
}

/// Sample-to-offset (in bytes from start of MP4) atom
/// for file sizes above the 32bit limit.
/// Each value represents a byte offset for a data chunk
/// in the corresponding track.
#[derive(Debug, Default)]
pub struct Co64(Vec<u64>);
impl Co64 {
    pub fn new(values: Vec<u64>) -> Self {
        Self(values)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn iter(&self) -> impl Iterator<Item = &u64> {
        self.0.iter()
    }
    /// Panics if out of bounds.
    pub fn slice(&self, range: Range<usize>) -> &[u64] {
        &self.0[range]
    }
}

impl From<Stco> for Co64 {
    fn from(value: Stco) -> Self {
        Co64(value.0.iter().map(|n| *n as u64).collect())
    }
}