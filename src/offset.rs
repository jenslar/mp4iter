//! MP4 byte offset (extracted from `stco` atom), size in bytes (extracted from `stsz` atom),
//! and duration (extracted from `stts`atom) in milliseconds
//! for a chunk of interleaved data.

use std::{collections::HashMap, io::{Seek, SeekFrom}};

use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use time::Duration;

use crate::{atom_types::AtomType, reader::AtomReadOrigin, Co64, FourCC, Mp4, Mp4Error, TargetReader};

#[derive(Debug, Default)]
pub struct Offsets(pub(crate) Vec<Offset>);

impl Offsets {
    /// Internal. Returns a track's sample information if reader
    /// position is at the start of a track or before the track's
    /// `stts` atom.
    ///
    /// - Sample byte offset via `stsc` (samples per chunk) and `stco` (chunk offsets)
    /// - Sample size via `stsz` (sample sizes)
    /// - Sample duration via `stts` (sample durations)
    ///
    /// Will fail or return incorrect data if reader position
    /// is not at or before the start of the `stts` atom
    /// (sample to time) for a track.
    ///
    /// Atom order within a track, i.e. the `trak` container atom
    /// (intermediary atoms and levels ignored):
    /// ```
    /// trak -> tkhd -> mdhd -> hdlr -> stts -> stsc -> stsz -> stco/co64
    /// ```
    pub(crate) fn new_old(
        mp4: &mut Mp4,
        time_scale: u32,
        time_scale_zero_ok: bool
    ) -> Result<Self, Mp4Error> {
        // atom order within a track (intermediary atoms/levels ignored):
        // tkhd -> mdhd -> hdlr -> stts -> stsc -> stsz -> stco

        // Have chunk offsets via stco atom, but offsets for
        // individual sample need to be calculated using
        // stsc atom.

        // Get sample durations from stts/sample to time atom.
        let stts = mp4.stts(false)?; // .durations();

        // Get sample to chunk atom (number of samples per chunk)
        let stsc = mp4.stsc(false)?;

        // Get sample sizes from stsz/sample to size atom.
        let stsz = mp4.stsz(false)?; // .sizes();

        // Get chunk offsets.
        // Chunk offset atom may be 32 bit (stco)
        // or 64 bit (co64) version, and some MP4
        // seem to mix them through out the file,
        // even for file sizes > 4GB (e.g. DJI Osmo Action 4).
        // Solution is to always output 64 bit offsets...
        let chunk_offsets = match mp4.len() > u32::MAX as u64 {
            // 64-bit size
            true => {
                // Before reading, store current moov reader position,
                // Search for co64, then if no co64 rewind to stored pos
                // and look for stco instead...
                let pos = mp4.pos_moov()?;

                // !!! POTENTIAL ISSUE: only works if this is the final track?
                // !!! I.e. What if the current track has stco
                // !!! and the co64 for the following track is found instead...?
                // !!! Instead: Only work at track/trak level and check trak
                // !!! bounds? at least for user facing functions
                match mp4.co64(false) {
                    Ok(co64) => co64.offsets().to_owned(),
                    // For files mixing stco and co64 the raised error
                    // is probably "no co64 found", but other errors
                    // are still discarded...
                    Err(_e) => {
                        mp4.seek_moov(SeekFrom::Start(pos))?;
                        Co64::from(mp4.stco(false)?).offsets().to_owned()
                    },
                }
            },
            // 32-bit size
            false => Co64::from(mp4.stco(false)?).offsets().to_owned(),
        };

        let co_len = chunk_offsets.len();

        // Convert chunk offsets to sample offsets by merging stsc, stco, stsz
        let sample_offsets: Vec<u64> = chunk_offsets
            // .into_iter()
            .into_par_iter()
            .enumerate()
            .map(|(i, co)| {
                // Get number of samples in this chunk
                // 1-based indexing, i.e. first chunk in stsc's
                // sample-to-chunk table will have index = 1.
                let no_of_samples = match stsc.no_of_samples(i + 1) {
                    Some(n) => n,
                    None => panic!("stsc index does not exist\nlen {}\ni+1 {}\nco len {}",
                        stsc.no_of_entries,
                        i+1,
                        co_len
                    ),
                };

                // Get sample sizes in this chunk
                // Panics on out of bounds... better to iter and use get for each sample?
                let smp_sizes = (&stsz.sizes()[i .. i + no_of_samples as usize]).to_vec();

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

        let offsets: Vec<Offset> = stts.durations()
            .iter()
            .zip(stsz.sizes().iter())
            .zip(sample_offsets.iter())
            .map(|((duration_ticks, size), position)| {
                Offset::new(
                    *position,
                    *size,
                    *duration_ticks,
                    time_scale,
                    time_scale_zero_ok
                )
            })
            .collect();

        return Ok(Self(offsets));
    }

    /// Internal. Returns a track's sample information if reader
    /// position is at the start of a track or before the track's
    /// `stts` atom.
    ///
    /// - Sample byte offset via `stsc` (samples per chunk) and `stco`/`co64` (chunk offsets)
    /// - Sample size via `stsz` (sample sizes)
    /// - Sample duration via `stts` (sample durations)
    ///
    /// Will fail or return incorrect data if reader position
    /// is not at or before the start of the `stts` atom
    /// (sample to time) for a track.
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
        time_scale_zero_ok: bool
    ) -> Result<Self, Mp4Error> {
        // Have chunk offsets via stco atom, but offsets for
        // individual sample need to be calculated using
        // stsc atom.

        // 1. Find stbl container atom, container so no need to seek to next when found
        let stbl_header = match mp4.reader.find_header(&TargetReader::Moov, "stbl", false)? {
            Some(hdr) => hdr,
            None => return Err(Mp4Error::NoSuchAtom("stbl".to_string())),
        };

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
                // Only one of stco or co64 can exist in a single track, insert with same same key
                "stco" => {offset_atoms.insert("stco", AtomType::Stco(atom.stco()?));},
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
            _ => return Err(Mp4Error::NoSuchAtom("stts".into())) // .durations();
        };

        // Get sample to chunk atom (number of samples per chunk)
        let stsc = match offset_atoms.get("stsc") {
            Some(AtomType::Stsc(a)) => a,
            _ => return Err(Mp4Error::NoSuchAtom("stsc".into())) // .durations();
        };

        // Get sample sizes from stsz/sample to size atom.
        let stsz = match offset_atoms.get("stsz") {
            Some(AtomType::Stsz(a)) => a,
            _ => return Err(Mp4Error::NoSuchAtom("stsz".into())) // .durations();
        };

        // Get either stco or co64 (whichever present), but return only co64
        let co64 = match offset_atoms.get("stco") {
            Some(AtomType::Stco(a)) => Co64::from(a.to_owned()),
            Some(AtomType::Co64(a)) => a.to_owned(),
            _ => return Err(Mp4Error::NoSuchAtom("stco".into())) // .durations();
        };

        let co_len = co64.offsets().len();

        // Convert chunk offsets to sample offsets by merging stsc, stco, stsz
        // let sample_offsets: Vec<u64> = chunk_offsets
        let sample_offsets: Vec<u64> = co64.offsets()
            // .into_iter()
            .into_par_iter()
            .enumerate()
            .map(|(i, co)| {
                // Get number of samples in this chunk
                // 1-based indexing, i.e. first chunk in stsc's
                // sample-to-chunk table will have index = 1.
                let no_of_samples = match stsc.no_of_samples(i + 1) {
                    Some(n) => n,
                    None => panic!("stsc index does not exist\nlen    {}\ni+1    {}\nco len {}",
                        stsc.no_of_entries,
                        i+1,
                        co_len
                    ),
                };

                // Get sample sizes in this chunk
                // Panics on out of bounds... better to iter and use get for each sample?
                let smp_sizes = (&stsz.sizes()[i .. i + no_of_samples as usize]).to_vec();

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

        let offsets: Vec<Offset> = stts.durations()
            .iter()
            .zip(stsz.sizes().iter())
            .zip(sample_offsets.iter())
            .map(|((duration_ticks, size), position)| {
                Offset::new(
                    *position,
                    *size,
                    *duration_ticks,
                    time_scale,
                    time_scale_zero_ok
                )
            })
            .collect();

        return Ok(Self(offsets));
    }

    pub fn iter(&self) -> impl Iterator<Item = &Offset>{
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> Option<&Offset> {
        self.0.first()
    }

    pub fn last(&self) -> Option<&Offset> {
        self.0.last()
    }

    pub fn get(&self, index: usize) -> Option<&Offset> {
        self.0.get(index)
    }
}

/// MP4 byte offset (from `stco` atom), size in bytes (from `stsz` atom),
/// and duration (from `stts`atom) in milliseconds
/// for a chunk of data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offset {
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

impl Offset {
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
