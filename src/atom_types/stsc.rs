//! Sample to chunk atom (`stsc`)
//!
//! Location:
//!
//! See:
//! - Sample to chunk atom: <https://developer.apple.com/documentation/quicktime-file-format/sample-to-chunk_atom>
//! - Sample to chunk table: <https://developer.apple.com/documentation/quicktime-file-format/sample-to-chunk_atom/sample-to-chunk_table>
//! - <https://github.com/essential61/mp4analyser/wiki/Understanding-The-Sample-Tables:-An-Example>

use binrw::BinRead;

/// Sample to chunk atom (`stsc`)
#[derive(Debug, BinRead)]
#[br(big)]
pub struct Stsc {
    pub(crate) version: u8,
    pub(crate) flags: [u8; 3],
    pub(crate) no_of_entries: u32,
    #[br(count = no_of_entries)]
    pub(crate) sample_to_chunk_table: Vec<SampleToChunk>,
}

impl Stsc {
    /// Returns number of samples for specified chunk,
    /// counting from start of MP4.
    ///
    /// > Important: The `first_chunk` field in an `stsc`
    /// > atom starts on 1,
    /// > so `chunk_index` is also a 1-based index,
    /// > exactly as the MP4 specification states.
    pub fn no_of_samples(&self, chunk_index: usize) -> Option<u32> {

        // Return early if only one entry, since this entry
        // is true for the entire track...
        if self.no_of_entries == 1 {
            if let Some(stc) = self.sample_to_chunk_table.first() {
                return Some(stc.samples_per_chunk)
            }
        }

        // instead of returning early,
        // add variable to be able to do last check
        let mut no_of_smp = None;

        // iterate one step at a time, but returning two tables,
        // then check if 'chunk_index' is within range of
        // 'first_chunk' value in chunk 2 from and
        // 'first_chunk' value in chunk 1, to derive how
        // many chunks contain specfied 'samples_per_chunk'
        // in chunk 1.
        // This will only check up until second to last
        // samples per chunk table.
        for s2chunks in self.sample_to_chunk_table.windows(2) {
            let s2c1 = &s2chunks[0];
            let s2c2 = &s2chunks[1];

            if (s2c1.first_chunk as usize .. s2c2.first_chunk as usize).contains(&chunk_index) {
                no_of_smp = Some(s2c1.samples_per_chunk);
            }
        }

        // Check if 'chunk_index' is larger than 'first_chunk'
        // in last sample to chunk table, since all remaining
        // chunk for the track chunks must have this number of samples
        if no_of_smp.is_none() {
            if let Some(last) = self.sample_to_chunk_table.last() {
                if chunk_index >= last.first_chunk as usize {
                    no_of_smp = Some(last.samples_per_chunk);
                }
            }
        }

        no_of_smp
    }

    /// Returns total number of samples
    pub fn len(
        &self,
    ) -> usize {
        // // If each chunk only contains a single sample
        // if self.sample_to_chunk_table.len() == 1 {
        //     return number_of_chunks
        // }

        // !!! not correct, missing information
        // !!! number of chunks required, since
        // !!! last chunk table stretches to the end,
        // !!! not just the first chunk specified.

        let mut sum = 0;
        for chunk in self.sample_to_chunk_table.windows(2) {
            // println!("{chunk:?}");
            let c1 = &chunk[0];
            let c2 = &chunk[1];

            // sum += (c2.first_chunk - c1.first_chunk) as usize;
            let delta = (c2.first_chunk - c1.first_chunk) as usize * c1.samples_per_chunk as usize;
            // println!("{delta}");
            sum += delta;
        }

        // Add final entry not caught by .windows()
        sum += self.sample_to_chunk_table
            .last()
            .unwrap()
            .samples_per_chunk as usize;

        sum // will be 0 for files with only 1 sample/chunk
    }

    /// Returns a list for individual sample offsets and sizes as
    /// a vector of tuples `(ABSOLUTE_BYTE_OFFSET, BYTE_SIZE)`.
    ///
    /// `stsz` (sample size atom) contains sample sizes for the track,
    /// `stco` (32 bit) or `co64` (64 bit) contain absolute chunk offsets.
    pub fn samples(
        &self,
        chunk_offsets: &[u64],
        sample_sizes: &[u32]
    ) -> Vec<(u64, u32)> {
        let len = self.sample_to_chunk_table.len();
        let mut sum = 0;
        let mut offsets: Vec<(u64, u32)> = Vec::new();
        for chunk in self.sample_to_chunk_table.windows(2) {
            println!("{chunk:?}");
            let c1 = &chunk[0];
            let c2 = &chunk[1];

            sum += c2.first_chunk - c1.first_chunk;
        }
        offsets
    }
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct SampleToChunk {
    /// 1-based index of first chunk
    /// that contains the number of
    /// samples specified in `samples_per_chunk`.
    /// The following chunks will all contain the
    /// same number of samples until the next
    /// sample to chunk entry.
    /// I.e. sub-tract `first_chunk` from the next
    /// sample to chunk entry, to calculate how
    /// many chunks that contain the same number of samples.
    pub(crate) first_chunk: u32,
    /// Number of samples for chunk number
    /// specified by `first_chunk` and on,
    /// until the next sample to chunk entry.
    pub(crate) samples_per_chunk: u32,
    pub(crate) sample_description_id: u32,
}
