//! Sample offsets consisting of byte offsets, extracted from `stco` (32bit) or `co64` (64bit) atom), size in bytes (extracted from `stsz` atom),
//! and duration (extracted from `stts` atom).

use std::{collections::HashMap, io::SeekFrom};

use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use time::Duration;

use crate::{atom_types::AtomType, reader::AtomReadOrigin, Co64, Mp4, Mp4Error, Stsd, TargetReader};

/// Sample offsets consisting of byte offsets,
/// (extracted from `stco` if 32bit or `co64` if 64bit atoms),
/// sizes in bytes (extracted from `stsz` atom),
/// and durations (extracted from `stts` atom).
#[derive(Debug, Default)]
pub struct SampleOffsets {
    pub(crate) stsd: Stsd,
    pub(crate) offsets: Vec<SampleOffset>
}

impl SampleOffsets {
    /// Internal. Returns a track's sample information.
    ///
    /// The optional `moov_position` must be within a track
    /// at the start of an atom header,
    /// but before or exactly  after the `stbl` header (sample table box)
    /// If set to `None` the position is assumed to be just after the
    /// `stbl` header.
    ///
    /// - Sample byte offset via `stsc` (samples per chunk) and `stco`/`co64` (chunk offsets)
    /// - Sample size via `stsz` (sample sizes)
    /// - Sample duration via `stts` (sample durations)
    ///
    /// Will fail or return incorrect data if reader position
    /// is not at or before the start of the `stbl` container atom
    /// for a track.
    ///
    /// This implementation is agnostic to the order of the sample atoms.
    /// Some MP4-files seem to move these around (specifically MP4 with
    /// `moov` atom before `mdat` atom?)
    ///
    /// The common atom order within a track, i.e. the `trak` container atom
    /// is usually, but critically, **not always**:
    /// ```
    /// trak -> tkhd -> mdhd -> hdlr -> stts -> stsc -> stsz -> stco/co64
    /// ```
    pub(crate) fn new(
        mp4: &mut Mp4,
        time_scale: u32,
        time_scale_zero_ok: bool,
        moov_position: Option<SeekFrom>
    ) -> Result<Self, Mp4Error> {
        // Have chunk offsets via stco atom, but offsets for
        // individual sample need to be calculated using
        // stsc atom.

        // Position must be at start of an atom,
        // within a `trak` before the sample atoms
        if let Some(seekfrom) = moov_position {
            mp4.seek_moov(seekfrom)?;
        }

        let mut stsd: Option<Stsd> = None;
        let mut offset_atoms: HashMap<&str, AtomType> = HashMap::new();

        loop {
            // Read "raw" atom at current position with moov reader
            let mut atom = mp4.atom(&TargetReader::Moov, AtomReadOrigin::None)?;

            let rel_pos_next = atom.header.next;

            // Match FourCC for each atom, ignore any that are not required
            // to derive track sample information.
            // Break if 4 of the below have been found, since only one of
            // each - and either stco or co64 - can exist for each track.
            match atom.header.name().to_str() {
                // stsd atom
                "stsd" => {stsd = Some(atom.stsd()?)},

                // Only one of stco or co64 can exist in a single track,
                // insert with same same key, only use 64bit offset values,
                // convert 32bit offsets (stco) to 64bit (co64)
                "stco" => {offset_atoms.insert("stco", AtomType::Co64(Co64::from(atom.stco()?)));},
                "co64" => {offset_atoms.insert("stco", AtomType::Co64(atom.co64()?));},

                // the following three are required
                "stsc" => {offset_atoms.insert("stsc", AtomType::Stsc(atom.stsc()?));},
                "stsz" => {offset_atoms.insert("stsz", AtomType::Stsz(atom.stsz()?));},
                "stts" => {offset_atoms.insert("stts", AtomType::Stts(atom.stts()?));},

                // If next track is encountered we've read too far so return error
                "trak" => return Err(Mp4Error::SampleOffsetError),

                // Not a relevant atom, seek to next
                _ => {mp4.seek_moov(SeekFrom::Current(i64::try_from(rel_pos_next)?))?;},
            }

            // if stco, stts, stsz or stco/co64 have been found break loop
            if offset_atoms.len() == 4 {
                break
            }
        }

        // Get sample durations from stts/sample to time atom.
        let stts = match offset_atoms.get("stts") {
            Some(AtomType::Stts(a)) => a,
            _ => return Err(Mp4Error::NoSuchAtom("stts".into()))
        };

        // Get sample to chunk atom (number of samples per chunk)
        let stsc = match offset_atoms.get("stsc") {
            Some(AtomType::Stsc(a)) => a,
            _ => return Err(Mp4Error::NoSuchAtom("stsc".into()))
        };

        // Get sample sizes from stsz/sample to size atom.
        let stsz = match offset_atoms.get("stsz") {
            Some(AtomType::Stsz(a)) => a,
            _ => return Err(Mp4Error::NoSuchAtom("stsz".into()))
        };

        // Get chunk offsets
        let co64 = match offset_atoms.get("stco") {
            Some(AtomType::Co64(a)) => a.to_owned(),
            _ => return Err(Mp4Error::NoSuchAtom("stco".into()))
        };

        let co_len = co64.offsets().len();

        // Convert chunk offsets to sample offsets by merging stsc, stco, stsz
        let sample_offsets: Vec<u64> = co64.offsets()
            .into_par_iter()
            .enumerate()
            .map(|(i, co)| {
                // Get number of samples in this chunk
                // 1-based indexing, i.e. first chunk in stsc's
                // sample-to-chunk table will have index = 1.
                let no_of_samples: u32 = match stsc.sample_count(i + 1) {
                    Some(n) => n,
                    None => panic!("stsc index does not exist\nlen    {}\ni+1    {}\nco len {}",
                        stsc.entry_count,
                        i+1,
                        co_len
                    ),
                };

                // Get sample sizes in this chunk
                // Panics on out of bounds... better to iter and use get for each sample?
                let smp_sizes: Vec<u32> = (&stsz.sizes()[i .. i + no_of_samples as usize]).to_vec();

                let mut delta = 0_u64;
                let smp_off: Vec<u64> = smp_sizes.into_iter()
                    .map(|s| {
                        let offset = co + delta;
                        delta += s as u64;
                        offset
                    })
                    .collect();

                Ok(smp_off)
            })
            .collect::<Result<Vec<Vec<u64>>, Mp4Error>>()?
            .into_iter()
            .flatten()
            .collect();

        let offsets: Vec<SampleOffset> = stts.durations()
            .iter()
            .zip(stsz.sizes().iter())
            .zip(sample_offsets.iter())
            .map(|((duration_ticks, size), position)| {
                SampleOffset::new(
                    *position,
                    *size,
                    *duration_ticks,
                    time_scale,
                    time_scale_zero_ok
                )
            })
            .collect();

        // return Ok(Self(offsets));
        return Ok(Self {
            stsd: stsd.ok_or_else(|| Mp4Error::NoSuchAtom("stsd".into()))?,
            offsets
        });
    }

    pub fn iter(&self) -> impl Iterator<Item = &SampleOffset>{
        self.offsets.iter()
    }

    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    pub fn first(&self) -> Option<&SampleOffset> {
        self.offsets.first()
    }

    pub fn last(&self) -> Option<&SampleOffset> {
        self.offsets.last()
    }

    pub fn get(&self, index: usize) -> Option<&SampleOffset> {
        self.offsets.get(index)
    }
}

/// Sample offset consisting of byte offset,
/// (extracted from `stco` if 32bit or `co64` if 64bit atoms),
/// size in bytes (extracted from `stsz` atom),
/// and duration (extracted from `stts` atom).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleOffset {
    /// Offset in bytes from start of file
    /// (extracted from `stco` atom).
    // pub position: u32,
    pub position: u64,
    /// Size of chunk in bytes
    /// (extracted from `stsz` atom).
    pub size: u32,
    /// The sample's duration, scaled according
    /// to the track's time scale (located in `mdhd` atom for each track).
    pub duration: Duration
}

impl SampleOffset {
    /// Create new offset with the corresponding sample's
    /// position, size, and duration (as `time::Duration`).
    ///
    /// Panics if `time_scale` is 0 (invalid), but if `time_scale_zero_ok` is `true`
    /// `time_scale` is instead set to 1 to avoid division by 0.
    pub fn new(
        position: u64,
        size: u32,
        duration_ticks: u32,
        mut time_scale: u32,
        time_scale_zero_ok: bool
    ) -> Self {
        if time_scale_zero_ok && time_scale == 0 {
            time_scale = 1;
        }
        let duration = Duration::seconds_f64(duration_ticks as f64 / time_scale as f64);
        Self{position, size, duration}
    }
}
