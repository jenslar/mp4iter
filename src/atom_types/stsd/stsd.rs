//! Video sample description atom (`stsd`).
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/video_sample_description>

use binrw::BinRead;

use crate::{Tmcd, Mp4Error};

use super::{Audio, AudioFormat, DataFormat, Video, VideoFormat, SampleDescription};

/// Video sample description atom (`stsd`).
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/video_sample_description>
#[derive(Debug, Default, BinRead)]
pub struct Stsd {
    _version: u8,
    _flags: [u8; 3],
    #[br(big)]
    _no_of_entries: u32,
    #[br(big, count = _no_of_entries)]
    descriptions: Vec<SampleDescription>
}

impl Stsd {
    pub fn descriptions(&self) -> &[SampleDescription] {
        &self.descriptions
    }

    pub fn tmcd(&self) -> Result<Tmcd, Mp4Error> {
        self.descriptions.iter()
            .find(|s| s.data_format() == &DataFormat::Binary(['t', 'm', 'c', 'd']))
            .map(|s| s.tmcd())
            .ok_or(Mp4Error::NoSuchAtom("tmcd".to_owned()))?
    }

    /// Returns `true` if the current `stsd`
    /// describes video.
    pub fn is_video(&self) -> bool {
        self.descriptions.iter()
            .any(|s| s.is_video())
    }

    /// Returns a sample description for video,
    /// if that is what the current `stsd` atom describes.
    pub fn video(&self) -> Option<&Video> {
        self.descriptions.iter()
            .find_map(|s| s.video())
    }

    /// Returns video format.
    pub fn video_format(&self) -> Option<&VideoFormat> {
        self.descriptions.iter()
            .find_map(|sd| {
                if let DataFormat::Video(fmt) = sd.data_format() {
                    Some(fmt)
                } else {
                    None
                }
            })
    }

    /// Returns `true` if the current `stsd`
    /// describes audio.
    pub fn is_audio(&self) -> bool {
        self.descriptions.iter()
            .any(|s| s.is_audio())
    }

    /// Returns a sample description for audio,
    /// if that is what the current `stsd` atom describes.
    pub fn audio(&self) -> Option<&Audio> {
        self.descriptions.iter()
            .find_map(|s| s.audio())
    }

    /// Returns audio format.
    pub fn audio_format(&self) -> Option<&AudioFormat> {
        self.descriptions.iter()
            .find_map(|sd| {
                if let DataFormat::Audio(fmt) = sd.data_format() {
                    Some(fmt)
                } else {
                    None
                }
            })
    }

    /// Returns `true` if the current `stsd`
    /// describes binary data.
    pub fn is_binary(&self) -> bool {
        self.descriptions.iter()
            .any(|s| s.is_binary())
    }

    /// Returns a sample description for binary data,
    /// if that is what the current `stsd` atom describes.
    pub fn binary(&self) -> Option<&[u8]> {
        self.descriptions.iter()
            .find_map(|s| s.binary())
    }

    /// Returns resolution in pixels
    /// as tuple `(WIDTH, HEIGHT)`,
    /// if current `stsd` describes video.
    pub fn resolution(&self) -> Option<(u16, u16)> {
        let video = self.video()?;
        Some((video.width(), video.height()))
    }

    /// Returns audio sample rate in Hz,
    /// if current `stsd` describes audio.
    pub fn sample_rate(&self) -> Option<f64> {
        self.audio()?.sample_rate()
    }
}
