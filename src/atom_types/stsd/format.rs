use std::fmt::Display;

use binrw::BinRead;

use super::{Audio, Video};
use crate::support::chars_from_be_u32;

#[derive(Debug, BinRead)]
#[br(import {size: u32, data_format: DataFormat})]
pub enum DataLoad {
    #[br(pre_assert(data_format.is_video()))]
    Video(
        #[br(args {size})]
        Video
    ),
    #[br(pre_assert(data_format.is_audio()))]
    Audio(
        #[br(args {size})]
        Audio
    ),
    #[br(pre_assert(data_format.is_binary()))]
    Binary(
        #[br(count = size - 16)]
        Vec<u8>
    ),
}

impl DataLoad {
    pub fn video(&self) -> Option<&Video> {
        match &self {
            Self::Video(v) => Some(v),
            _ => None,
        }
    }
    
    pub fn audio(&self) -> Option<&Audio> {
        match &self {
            Self::Audio(v) => Some(v),
            _ => None,
        }
    }

    pub fn binary(&self) -> Option<&[u8]> {
        match &self {
            Self::Binary(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataFormat {
    Audio(AudioFormat),
    Video(VideoFormat),
    Binary([char; 4]) // String is not Copy. Needed for binrw arg.
}

impl Display for DataFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataFormat::Audio(fmt) => write!(f, "{fmt}"),
            DataFormat::Video(fmt) => write!(f, "{fmt}"),
            DataFormat::Binary(fmt) => write!(f, "{}", fmt.iter().collect::<String>()),
        }
    }
}

impl DataFormat {
    /// Create new `DataFormat` from Big Endian `u32` value.
    pub(crate) fn new(value: u32) -> DataFormat {
        match (AudioFormat::from_be_u32(value), VideoFormat::from_be_u32(value)) {
            (AudioFormat::Unknown, VideoFormat::Unknown) => DataFormat::Binary(chars_from_be_u32(value)),
            (AudioFormat::Unknown, vf) => DataFormat::Video(vf),
            (af, VideoFormat::Unknown) => DataFormat::Audio(af),
            // should probably assert that both audio and video do not
            // return valid formats here...
            _ => DataFormat::Binary(chars_from_be_u32(value))
        }
    }

    pub(crate) fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_))
    }

    pub(crate) fn is_audio(&self) -> bool {
        if self == &Self::Audio(AudioFormat::Unknown) {
            return false
        }
        matches!(self, Self::Audio(_))
    }

    pub(crate) fn is_video(&self) -> bool {
        if self == &Self::Video(VideoFormat::Unknown) {
            return false
        }
        matches!(self, Self::Video(_))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VideoFormat {
    /// `avc1` H.264 video
    Avc1,
    /// `cvid` Cinepak
    Cvid,
    /// `dvc ` NTSC DV-25 video
    Dvc,
    /// `dvcp` PAL DV-25 video
    Dvcp,
    /// `gif ` CompuServe Graphics Interchange Format
    Gif,
    /// `h263` H.263 video
    H263,
    /// `hvc1`
    Hvc1,
    /// `jpeg` JPEG
    Jpeg,
    /// `kpcd` Kodak Photo CD
    Kpcd,
    /// `mjpa` Motion-JPEG (format A)
    Mjpa,
    /// `mjpb` Motion-JPEG (format B)
    Mjpb,
    /// `mp4v` MPEG-4 video
    Mp4v,
    /// `png ` Portable Network Graphics
    Png,
    /// `raw ` Uncompressed RGB
    Raw,
    /// `rle ` Animation
    Rle,
    /// `rpza` Apple video
    Rpza,
    /// `smc ` Graphics
    Smc,
    /// `SVQ1` Sorenson video, version 1
    Svq1,
    /// `SVQ3` Sorenson video 3
    Svq3,
    /// `tiff` Tagged Image File Format
    Tiff,
    /// `2vu ` Uncompressed Y´CbCr,
    /// 8-bit-per-component 4:2:2
    TwoVu,
    /// `v210` Uncompressed Y´CbCr,
    /// 10-bit-per-component 4:2:2
    V210,
    /// `v216` Uncompressed Y´CbCr,
    /// 10, 12, 14, or 16-bit-per-component 4:2:2
    V216,
    /// `v308` Uncompressed Y´CbCr,
    /// 8-bit-per-component 4:4:4
    V308,
    /// `v408` Uncompressed Y´CbCr,
    /// 8-bit-per-component 4:4:4:4
    V408,
    /// `v410` Uncompressed Y´CbCr,
    /// 10-bit-per-component 4:4:4
    V410,
    /// `yuv2` Uncompressed Y´CbCr,
    /// 8-bit-per-component 4:2:2
    Yuv2,
    /// Unknown/undocumented video format
    Unknown
}

impl Display for VideoFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str().unwrap_or("Unknown"))
    }
}

impl VideoFormat {
    /// Convert from Big Endian `u32` to `AudioFormat`
    fn from_be_u32(value: u32) -> Self {
        match value {
            0x61766331 => Self::Avc1,
            0x63766964 => Self::Cvid,
            0x64766320 => Self::Dvc,
            0x64766370 => Self::Dvcp,
            0x67696620 => Self::Gif,
            0x68323633 => Self::H263,
            0x68766331 => Self::Hvc1,
            0x6a706567 => Self::Jpeg,
            0x6b706364 => Self::Kpcd,
            0x6d6a7061 => Self::Mjpa,
            0x6d6a7062 => Self::Mjpb,
            0x6d703476 => Self::Mp4v,
            0x706e6720 => Self::Png,
            0x72617720 => Self::Raw,
            0x726c6520 => Self::Rle,
            0x72707a61 => Self::Rpza,
            0x736d6320 => Self::Smc,
            0x53565131 => Self::Svq1,
            0x53565133 => Self::Svq3,
            0x74696666 => Self::Tiff,
            0x32767520 => Self::TwoVu,
            0x76323130 => Self::V210,
            0x76323136 => Self::V216,
            0x76333038 => Self::V308,
            0x76343038 => Self::V408,
            0x76343130 => Self::V410,
            0x79757632 => Self::Yuv2,
            _ => Self::Unknown,
        }
    }

    // Returns `&str` in the same form
    // format is stored, e.g. with added
    // space if only three bytes/characters.
    pub fn to_str(&self) -> Option<&str> {
        match self {
            VideoFormat::Avc1 => Some("avc1"),
            VideoFormat::Cvid => Some("cvid"),
            VideoFormat::Dvc => Some("dvc "),
            VideoFormat::Dvcp => Some("dvcp"),
            VideoFormat::Gif => Some("gif "),
            VideoFormat::H263 => Some("h263"),
            VideoFormat::Hvc1 => Some("hvc1"),
            VideoFormat::Jpeg => Some("jpeg"),
            VideoFormat::Kpcd => Some("kpcd"),
            VideoFormat::Mjpa => Some("mjpa"),
            VideoFormat::Mjpb => Some("mjpb"),
            VideoFormat::Mp4v => Some("mp4v"),
            VideoFormat::Png => Some("png "),
            VideoFormat::Raw => Some("raw "),
            VideoFormat::Rle => Some("rle "),
            VideoFormat::Rpza => Some("rpza"),
            VideoFormat::Smc => Some("smc "),
            VideoFormat::Svq1 => Some("SVQ1"),
            VideoFormat::Svq3 => Some("SVQ3"),
            VideoFormat::Tiff => Some("tiff"),
            VideoFormat::TwoVu => Some("2vu "),
            VideoFormat::V210 => Some("v210"),
            VideoFormat::V216 => Some("v216"),
            VideoFormat::V308 => Some("v308"),
            VideoFormat::V408 => Some("v408"),
            VideoFormat::V410 => Some("v410"),
            VideoFormat::Yuv2 => Some("yuv2"),
            VideoFormat::Unknown => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AudioFormat {
    /// `0x00000000` Not specified, This format descriptor should not be used,
    /// but may be found in some files.
    NotSpecified,
    /// `NONE` kSoundNotCompressed. This format descriptor should not be used,
    /// but may be found in some files.
    None,
    /// `raw` k8BitOffsetBinaryFormat
    Raw,
    /// `twos` k16BitBigEndianFormat
    Twos,
    /// `ac-3` kAC3AudioFormat, Digital Audio Compression Standard (AC-3, Enhanced AC-3)
    Ac3,
    /// `sowt` k16BitLittleEndianFormat, 16-bit little-endian, twos-complement
    Sowt,
    /// `MAC3` kMACE3Compression, Samples have been compressed using MACE 3:1. (Obsolete.)
    Mac3,
    /// `MAC6` kMACE6Compression, Samples have been compressed using MACE 6:1. (Obsolete.)
    Mac6,
    /// `ima4` kIMACompression, Samples have been compressed using IMA 4:1.
    Ima4,
    /// `fl32` kFloat32Format, 32-bit floating point
    Fl32,
    /// `fl64` kFloat64Format, 64-bit floating point
    Fl64,
    /// `in24` k24BitFormat, 24-bit integer
    In24,
    /// `in32` k32BitFormat, 32-bit integer
    In32,
    /// `ulaw` kULawCompression, uLaw 2:1
    Ulaw,
    /// `alaw` kALawCompression, uLaw 2:1
    Alaw,
    /// `0x6D730002` kMicrosoftADPCMFormat, Microsoft ADPCM-ACM code 2
    MsAdpcm,
    /// `0x6D730011` kDVIIntelIMAFormat, DVI/Intel IMAADPCM-ACM code 17
    IntelMa,
    /// `dvca` kDVAudioFormat, DV Audio
    Dvca,
    /// `QDMC` kQDesignCompression, QDesign music
    Qdmc,
    /// `QDM2` kQDesign2Compression, QDesign music version 2
    Qdm2,
    /// `Qclp` kQUALCOMMCompression, QUALCOMM PureVoice
    Qclp,
    /// `0x6D730055` kMPEGLayer3Format, MPEG-1 layer 3, CBR only (pre-QT4.1)
    Mpeg1Layer3,
    /// `.mp3` kFullMPEGLay3Format, MPEG-1 layer 3, CBR & VBR (QT4.1 and later)
    Mp3,
    /// `mp4a` kMPEG4AudioFormat, MPEG-4, Advanced Audio Coding (AAC)
    Mp4a,
    /// Unknown/undocumented audio format
    Unknown
}

impl Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str().unwrap_or("Unknown"))
    }
}

impl AudioFormat {
    /// Convert from Big Endian `u32` to `AudioFormat`
    fn from_be_u32(value: u32) -> Self {
        match value {
            0x00000000 => Self::NotSpecified,
            0x0200736d => Self::MsAdpcm,
            0x1100736d => Self::IntelMa,
            0x5500736d => Self::Mpeg1Layer3,
            0x6d703461 => Self::Mp4a,
            0x4e4f4e45 => Self::None,
            0x72617720 => Self::Raw,
            0x74776f73 => Self::Twos,
            0x736f7774 => Self::Sowt,
            0x4d414333 => Self::Mac3,
            0x4d414336 => Self::Mac6,
            0x696d6134 => Self::Ima4,
            0x666c3332 => Self::Fl32,
            0x666c3634 => Self::Fl64,
            0x696e3234 => Self::In24,
            0x696e3332 => Self::In32,
            0x756c6177 => Self::Ulaw,
            0x616c6177 => Self::Alaw,
            0x64766361 => Self::Dvca,
            0x51444d43 => Self::Qdmc,
            0x51444d32 => Self::Qdm2,
            0x51636c70 => Self::Qclp,
            0x2e6d7033 => Self::Mp3,
            0x61632d33 => Self::Ac3,
            _ => Self::Unknown
        }
    }

    // Returns `&str` in the same form
    // format is stored, e.g. with added
    // space if only three bytes/characters.
    pub fn to_str(&self) -> Option<&str> {
        match self {
            AudioFormat::NotSpecified => Some("\0\0\0\0"),
            AudioFormat::None => Some("NONE"),
            AudioFormat::Raw => Some("raw "),
            AudioFormat::Twos => Some("twos"),
            AudioFormat::Sowt => Some("sowt"),
            AudioFormat::Mac3 => Some("MAC3"),
            AudioFormat::Mac6 => Some("MAC6"),
            AudioFormat::Ima4 => Some("ima4"),
            AudioFormat::Fl32 => Some("fl32"),
            AudioFormat::Fl64 => Some("fl64"),
            AudioFormat::In24 => Some("in24"),
            AudioFormat::In32 => Some("in32"),
            AudioFormat::Ulaw => Some("ulaw"),
            AudioFormat::Alaw => Some("alaw"),
            AudioFormat::MsAdpcm => Some("ms\0\x02"), // LE 0x6D730002
            AudioFormat::IntelMa => Some("ms\0\x11"), // LE 0x6D730011
            AudioFormat::Dvca => Some("dvca"),
            AudioFormat::Qdmc => Some("QDMC"),
            AudioFormat::Qdm2 => Some("QDM2"),
            AudioFormat::Qclp => Some("Qclp"),
            AudioFormat::Mpeg1Layer3 => Some("ms\0U"), // LE 0x6D730055
            AudioFormat::Mp3 => Some(".mp3"),
            AudioFormat::Mp4a => Some("mp4a"),
            AudioFormat::Ac3 => Some("ac-3"),
            AudioFormat::Unknown => None,
        }
    }
}