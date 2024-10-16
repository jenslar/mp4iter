//! An MP4 "track", containing compiled information such as time scale, byte offsets for data load in `mdat` etc.
//!
//! ```rust
//! # Locate and iterate over GoPro telemetry
//! let path = PathBuf::from ("PATH/TO/GOPROVIDEO.MP4");
//! let gopro_gpmf_track = Mp4::new(&path).unwrap().track("GoPro MET");
//! ```

use std::io::{Cursor, SeekFrom};

use time::{Duration, PrimitiveDateTime};

use crate::{Mp4, Mp4Error, Mp4Reader, Offset, TargetReader};

use super::attributes::TrackAttributes;

#[derive(Debug)]
pub struct Track<'a> {
    /// Attributes
    pub attributes: TrackAttributes,

    /// Borrowed reader over the MP4-file.
    pub(crate) reader: &'a mut Mp4Reader
}

impl <'a> Track<'a> {
    pub fn from_id(
        mp4: &'a mut Mp4,
        id: u32,
        reset: bool
    ) -> Result<Self, Mp4Error> {
        Self::new(mp4, TrackIdentifier::Id(id), reset)
    }

    pub fn from_name(
        mp4: &'a mut Mp4,
        name: &str,
        reset: bool
    ) -> Result<Self, Mp4Error> {
        Self::new(mp4, TrackIdentifier::Name(name), reset)
    }

    pub fn new(
        mp4: &'a mut Mp4,
        identifier: TrackIdentifier,
        reset: bool
    ) -> Result<Self, Mp4Error> {
        // atom order: tkhd -> mdhd -> hdlr -> stts -> stsz -> stco

        let attributes = TrackAttributes::new(mp4, identifier, reset)?;

        Ok(Self {
            attributes,
            reader: &mut mp4.reader,
        })
    }

    pub fn name(&self) -> &str {
        &self.attributes.name
    }

    pub fn id(&self) -> u32 {
        self.attributes.id
    }

    pub fn creation_time(&self) -> PrimitiveDateTime {
        self.attributes.creation_time
    }

    pub fn modification_time(&self) -> PrimitiveDateTime {
        self.attributes.modification_time
    }

    pub fn track_type(&self) -> &str {
        &self.attributes.track_type
    }

    pub fn offsets(&self) -> impl Iterator<Item = &Offset> {
        self.attributes.offsets.iter()
    }

    /// Duration for this longest track in seconds.
    pub fn duration(&self) -> Duration {
        self.attributes.duration()
    }

    /// Returns an iterator over raw track data,
    /// which yields a reader with the shape `Cursor<Vec<u8>>`
    /// for each byte offset in the track.
    pub fn data(&'a mut self) -> impl Iterator<Item = Result<Cursor<Vec<u8>>, Mp4Error>> + 'a {
        self.attributes.offsets
            .iter()
            .map(|o| self.reader.cursor(
                &TargetReader::File,
                o.size as usize,
                Some(SeekFrom::Start(o.position)), // relative search from previous offset instead?
                None
            ))
    }

    /// Returns an iterator over increasing, relative timestamps
    /// together with the sample's duration for the track,
    /// yielded as `(Duration, Duration)` (`(relative_timestamp, sample_duration)`)
    /// starting at 0.
    pub fn timestamps(&'a self) -> impl Iterator<Item = (Duration, Duration)> + 'a {
        let mut t = Duration::ZERO;
        self.attributes.offsets
            .iter()
            .map(move |o| {
                let t1 = t;
                t += o.duration;
                (t1, o.duration)
            })
    }
}

#[derive(Debug, PartialEq)]
pub enum TrackIdentifier<'a> {
    Name(&'a str),
    Id(u32)
}
