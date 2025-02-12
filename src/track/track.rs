//! An MP4 "track", containing compiled information such as time scale, byte offsets for data load in `mdat` etc.
//! Use `Track::samples()` to iterate over raw sample data for the track.
//!
//! ```rust
//! # Locate and iterate over GoPro telemetry
//! let path = PathBuf::from ("PATH/TO/GOPROVIDEO.MP4");
//! let gopro_gpmf_track = Mp4::new(&path).unwrap().track("GoPro MET");
//! ```

use std::io::{Cursor, SeekFrom};

use time::{Duration, PrimitiveDateTime};

use crate::{AudioFormat, Mp4, Mp4Error, Mp4Reader, SampleOffset, TargetReader, Tmcd, VideoFormat};

use super::{attributes::TrackAttributes, sample::Sample};

#[derive(Debug)]
pub struct Track<'a> {
    /// Attributes
    pub attributes: TrackAttributes,

    /// Borrowed reader over the MP4-file.
    pub(crate) reader: &'a mut Mp4Reader
}

impl <'a> Track<'a> {
    pub fn new(
        mp4: &'a mut Mp4,
        identifier: TrackIdentifier,
        reset: bool
    ) -> Result<Self, Mp4Error> {
        // common atom order: tkhd -> mdhd -> hdlr -> stts -> stsz -> stco

        let attributes = TrackAttributes::new(mp4, identifier, reset)?;

        // Always reset position
        // to enable relative seek.
        // A Track mutably borrows the mp4 reader
        // anyway meaning nothing else is affected
        // until the track is dropped.
        mp4.reset()?;

        Ok(Self {
            attributes,
            reader: &mut mp4.reader,
        })
    }

    /// Returns track with
    /// specified numerical ID.
    pub fn from_id(
        mp4: &'a mut Mp4,
        id: u32,
        reset: bool
    ) -> Result<Self, Mp4Error> {
        Self::new(mp4, TrackIdentifier::Id(id), reset)
    }

    /// Returns first track with
    /// specified free text name,
    /// e.g. "GoPro MET" for
    /// GoPro timed telemetry track.
    pub fn from_name(
        mp4: &'a mut Mp4,
        name: &str,
        reset: bool
    ) -> Result<Self, Mp4Error> {
        Self::new(mp4, TrackIdentifier::Name(name), reset)
    }

    /// Returns the first track with specified sub type,
    /// e.g. `vide` for video track
    /// (more than one may exist).
    pub fn from_subtype(
        mp4: &'a mut Mp4,
        subtype: &str,
        reset: bool
    ) -> Result<Self, Mp4Error> {
        Self::new(mp4, TrackIdentifier::SubType(subtype), reset)
    }

    pub fn from_attributes(
        mp4: &'a mut Mp4,
        attributes: TrackAttributes,
    ) -> Result<Self, Mp4Error> {
        mp4.reset()?;
        Ok(Self {
            attributes,
            reader: &mut mp4.reader
        })
    }

    /// Number of samples in track.
    pub fn len(&self) -> usize {
        self.attributes.offsets.len()
    }

    /// Summed sample size in bytes for the track.
    pub fn size(&self) -> u64 {
        self.offsets()
            .map(|o| o.size as u64)
            .sum()
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

    pub fn sub_type(&self) -> &str {
        &self.attributes.sub_type
    }

    pub fn offsets(&self) -> impl Iterator<Item = &SampleOffset> {
        self.attributes.offsets.iter()
    }

    /// Duration for this longest track in seconds.
    pub fn duration(&self) -> Duration {
        self.attributes.duration()
    }

    /// Deprecated, use `Track::samples()` instead.
    ///
    /// Returns an iterator over raw sample data,
    /// which yields each sample as a reader
    /// with the shape `Cursor<Vec<u8>>`.
    #[deprecated]
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

    /// Returns an iterator over the track's samples.
    pub fn samples(&'a mut self) -> impl Iterator<Item = Result<Sample, Mp4Error>> + 'a {
        // Keep track of relative timestamp
        let mut t = Duration::ZERO;

        self.attributes.offsets
            .iter()
            .map(move |offset| {
                let rel_t = t;
                t += offset.duration; // add delta to relative time for next iteration
                (offset, rel_t)
            })
            .map(|(offset, rel_t)| {
                Sample::new(
                    &mut self.reader,
                    offset.to_owned(),
                ).map(|s| s.with_time(rel_t))
            })
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

    pub fn tmcd(&self) -> Result<Tmcd, Mp4Error> {
        self.attributes.offsets.stsd.tmcd()
    }

    pub fn number_of_frames(&self) -> Result<u8, Mp4Error> {
        Ok(self.tmcd()?.number_of_frames)
    }

    pub fn is_video(&self) -> bool {
        self.attributes.offsets.stsd.is_video()
    }

    pub fn video_format(&self) -> Option<&VideoFormat> {
        self.attributes.offsets.stsd.video_format()
    }

    pub fn is_audio(&self) -> bool {
        self.attributes.offsets.stsd.is_audio()
    }

    pub fn audio_format(&self) -> Option<&AudioFormat> {
        self.attributes.offsets.stsd.audio_format()
    }

    pub fn is_binary(&self) -> bool {
        self.attributes.offsets.stsd.is_binary()
    }

    pub fn sample_rate(&self) -> Option<f64> {
        self.attributes.offsets.stsd.sample_rate()
    }
}

/// Represents ways to identify a track:
/// - Name: String extracted from `hdlr` atom (`handler_name`).
/// - ID: Numerical ID extracted from `tkhd` atom.
/// - SubType: String describing track type. Audio  = `soun`, video = `vide`. Extracted from `hdlr` atom.
#[derive(Debug, PartialEq)]
pub enum TrackIdentifier<'a> {
    /// Track name
    Name(&'a str),
    /// Track ID
    Id(u32),
    /// Example track subtypes:
    /// - `vide` = video
    /// - `soun` = audio
    /// - `tmcd` = time code
    /// - `gpmd` = GoPro metadata/telemetry
    /// - `djmd` = DJI metadata/telemetry
    /// - `meta`
    SubType(&'a str),
}

impl <'a> From<&'a str> for TrackIdentifier<'a> {
    /// Returns `TrackIdentifier::Id` if value parses
    /// to `u32`, or `TrackIdentifier::Name` otherwise.
    /// Meaning this onversion will never return
    /// `TrackIdentifier::SubType`
    fn from(value: &'a str) -> Self {
        match value.parse::<u32>() {
            Ok(n) => Self::Id(n),
            Err(_) => Self::Name(value),
        }
    }
}

impl std::fmt::Display for TrackIdentifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackIdentifier::Name(s) => write!(f, "{s}"),
            TrackIdentifier::Id(s) => write!(f, "{s}"),
            TrackIdentifier::SubType(s) => write!(f, "{s}"),
        }
    }
}
