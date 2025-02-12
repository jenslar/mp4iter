use time::{Duration, PrimitiveDateTime, ext::NumericalDuration};

use crate::{AudioFormat, Mp4, Mp4Error, SampleOffset, SampleOffsets, Tmcd, VideoFormat};

use super::TrackIdentifier;

#[derive(Debug)]
pub struct TrackAttributes {
    // atom order (intermediary atoms ignored):
    // tkhd -> mdhd -> hdlr -> stsd -> stts -> stsc -> stsz -> stco

    /// Track name.
    /// `hdlr.component_name`
    pub(crate) name: String,
    /// Track ID.
    /// `tkhd.track_id`
    pub(crate) id: u32,
    /// Creation time.
    /// `tkhd.creation_time`
    pub(crate) creation_time: PrimitiveDateTime,
    /// Modification time.
    /// `tkhd.modification_time`
    pub(crate) modification_time: PrimitiveDateTime,

    /// Track type, e.g. `soun` for an audio track.
    /// hdlr.component_sub_type ([char; 4])
    pub(crate) sub_type: String,

    /// Track time scale.
    /// `mdhd.time_scale`
    pub(crate) time_scale: u32,
    /// Unscaled duration of track.
    /// `mdhd.duration`
    pub(crate) duration: u32,

    /// Width in pixels.
    /// Will be set to 0 if
    /// not a video track.
    pub(crate) width: f64,
    /// Height in pixels.
    /// Will be set to 0 if
    /// not a video track.
    pub(crate) height: f64,

    /// Absolute sample offsets, sizes in bytes,
    /// and sample durations
    /// for all samples in this track.
    // pub(crate) offsets: Vec<Offset>, // derived from stts, stsc, stsz, stco
    pub(crate) offsets: SampleOffsets, // derived from stts, stsc, stsz, stco
}

impl TrackAttributes {
    pub fn new(
        mp4: &mut Mp4,
        identifier: TrackIdentifier,
        reset: bool
    ) -> Result<Self, Mp4Error> {
        if reset {
            mp4.reset()?;
        }

        // Loop until EOF
        loop {
            // 1. find correct tkhd via hdlr.component_name

            // Parse tkhd + mdhd first, since these precede
            // the hdlr atom containing the handler/track name.
            let tkhd = match mp4.tkhd(false) {
                Ok(atom) => atom,
                Err(_err) => return Err(Mp4Error::NoSuchTrack(identifier.to_string())),
            };
            let track_id = TrackIdentifier::Id(tkhd.track_id());
            let mdhd = mp4.mdhd(false)?;
            let hdlr = mp4.hdlr(false)?;
            let track_subtype = TrackIdentifier::SubType(hdlr.component_sub_type());
            let track_name = TrackIdentifier::Name(hdlr.component_name());

            // Find and read past header for container atom stbl (sample table box)
            // which is the correct position for finding/reading offsets.
            // Note that atom order in the sample table box is only recommended.
            let _ = mp4.find_header("stbl", false)?;

            // 2. find mdhd, hdlr that follow after
            if identifier == track_id || identifier == track_name || identifier == track_subtype {
                let attributes = Self {
                    name: hdlr.component_name().to_owned(),
                    id: tkhd.track_id,
                    creation_time: tkhd.creation_time(),
                    modification_time: tkhd.modification_time(),
                    sub_type: hdlr.component_sub_type().to_owned(),
                    time_scale: mdhd.time_scale,
                    duration: mdhd.duration,
                    width: tkhd.width(),
                    height: tkhd.height(),
                    // offsets: mp4.sample_offsets_current_pos(mdhd.time_scale, true)?
                    offsets: SampleOffsets::new(mp4, mdhd.time_scale, true, None)?
                };

                return Ok(attributes)
            }
        }
    }

    /// Returns attributes for all tracks.
    pub fn all(
        mp4: &mut Mp4,
        reset: bool
    ) -> Result<Vec<Self>, Mp4Error> {
        if reset {
            mp4.reset()?;
        }

        let mut attributes: Vec<Self> = Vec::new();

        // Loop until no more tracks
        while let Ok(tkhd) = mp4.tkhd(false) {
            // Parse tkhd + mdhd first, since these precede
            // the hdlr atom containing the handler/track name.
            let mdhd = mp4.mdhd(false)?;
            let hdlr = mp4.hdlr(false)?;

            attributes.push(Self {
                name: hdlr.component_name().to_owned(),
                id: tkhd.track_id,
                creation_time: tkhd.creation_time(),
                modification_time: tkhd.modification_time(),
                sub_type: hdlr.component_sub_type().to_owned(),
                time_scale: mdhd.time_scale,
                duration: mdhd.duration,
                width: tkhd.width(),
                height: tkhd.height(),
                // Offsets also contain stsd, since this atom
                // is only recomended to precede
                // the sample info atoms...
                offsets: SampleOffsets::new(mp4, mdhd.time_scale, true, None)?
            })
        }

        Ok(attributes)
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn creation_time(&self) -> PrimitiveDateTime {
        self.creation_time
    }

    pub fn modification_time(&self) -> PrimitiveDateTime {
        self.modification_time
    }

    /// Track sub type, e.g. `vide` for a video track.
    pub fn sub_type(&self) -> &str {
        &self.sub_type
    }

    pub fn time_scale(&self) -> u32 {
        self.time_scale
    }

    pub fn duration_unscaled(&self) -> u32 {
        self.duration
    }

    /// Duration for this longest track in seconds.
    pub fn duration(&self) -> Duration {
        (self.duration as f64 / self.time_scale as f64).seconds()
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    // pub fn offsets(&self) -> impl Iterator<Item = &Offset> {
    pub fn offsets(&self) -> &[SampleOffset] {
        &self.offsets.offsets
    }

    // --- BELOW VIA STSD ATOM

    pub fn tmcd(&self) -> Result<Tmcd, Mp4Error> {
        self.offsets.stsd.tmcd()
    }

    pub fn is_video(&self) -> bool {
        self.offsets.stsd.is_video()
    }

    pub fn video_format(&self) -> Option<&VideoFormat> {
        self.offsets.stsd.video_format()
    }

    pub fn is_audio(&self) -> bool {
        self.offsets.stsd.is_audio()
    }

    pub fn audio_format(&self) -> Option<&AudioFormat> {
        self.offsets.stsd.audio_format()
    }

    pub fn is_binary(&self) -> bool {
        self.offsets.stsd.is_binary()
    }

    /// E.g. sample rate for audio track.
    pub fn sample_rate(&self) -> Option<f64> {
        self.offsets.stsd.sample_rate()
    }

    /// E.g. frame rate for video track.
    pub fn frame_rate(&self) -> f64 {
        // video sample_count * MP4 time_scale / MP4 unscaled_duration
        self.offsets.len() as f64 * self.time_scale as f64 / self.duration as f64
    }

    /// E.g. frame rate for video track.
    pub fn frame_rate_file(
        &self,
        mvhd_time_scale: u32,
        mvhd_duration: u32,
    ) -> f64 {
        // video sample_count * MP4 time_scale / MP4 unscaled_duration
        self.offsets.len() as f64 * mvhd_time_scale as f64 / mvhd_duration as f64
    }
}
