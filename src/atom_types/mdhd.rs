//! Media header atom (`mdhd`).
//!
//! Similar to `mvhd`,
//! but only describes a single track (`trak`).
//! Specifies the characteristics of a media (`mdia`),
//! including time scale and duration.
//!
//! Location: `moov/trak/mdia/mdhd`
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/media_header_atom>

use binrw::BinRead;
use time::{ext::NumericalDuration, Duration};

/// Media header atom ('mdhd'). One per track (`trak`).
/// Specifies the characteristics of a media (`mdia`), including time scale and duration.
///
/// Path: `moov/trak/mdia/mdhd`
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/media_header_atom>
#[derive(Debug, Default, BinRead)]
#[br(big)]
pub struct Mdhd {
    _version: u8,
    _flags: [u8; 3],
    pub(crate) creation_time: u32, // should be UTC
    pub(crate) modification_time: u32, // should be UTC
    pub(crate) time_scale: u32,
    /// Unscaled duration. I.e. "ticks"
    /// that require dividing by time scale
    /// to derive a value in seconds.
    pub(crate) duration: u32,
    /// Specifies the language code for this media.
    // pub(crate) language: LangaugeCode, // i16 is a guess taken from max value in language code enumerations
    // pub(crate) language: LangaugeCode, // i16 is a guess taken from max value in language code enumerations
    // pub(crate) language: [u8; 3], // iso spec states [u8; 3], but raises off by one error for atom upper bound
    // i16 errors out on ~/Desktop/DEV/humlab/elan/henrik_jens/ELAN/compose0001_h_3D_720.mp4 (home mbp)
    // pub(crate) language: LangaugeCode, // i16 was a guess taken from max value in language code enumerations
    // 1 bit pad 0 then [u5; 3]
    #[br(map = |data: u16| derive_language_code(data))]
    pub(crate) language: String, // i16 was a guess taken from max value in language code enumerations
    pub(crate) quality: u16,
}

impl Mdhd {
    pub fn creation_time(&self) -> u32 {
        self.creation_time
    }

    pub fn modification_time(&self) -> u32 {
        self.modification_time
    }

    pub fn time_scale(&self) -> u32 {
        self.time_scale
    }

    pub fn duration_unscaled(&self) -> u32 {
        self.duration
    }

    // See ISO 639‐2/T for the set of three
    // character codes.
    // mp4 spec: Each character is packed as the difference between its ASCII value and 0x60.
    /// ISO-639-2/T language code
    pub fn language(&self) -> &str {
    // pub fn language(&self) -> &[u8; 3] {
        &self.language
    }

    pub fn quality(&self) -> u16 {
        self.quality
    }

    /// Duration of the longest track in seconds.
    pub fn duration(&self) -> Duration {
        (self.duration as f64 / self.time_scale as f64).seconds()
    }
}

/// Derive three letter ISO639-2/T language code.
/// 
/// Packed in 16 bits `X u5 u5 u5`:
/// - most significant bit is padding (BE so left most)
/// - 1 `u5` + `0x60`
/// - 1 `u5` + `0x60`
/// - 1 `u5` + `0x60`
fn derive_language_code(data: u16) -> String {
    [
        // value between 0-31 + 96 = ascii range so casting to u8 is ok
        (((0b0111_1100_0000_0000 & data) >> 10) as u8 + 0x60) as char,
        (((0b0000_0011_1110_0000 & data) >> 5) as u8 + 0x60) as char,
        ((0b0000_0000_0001_1111 & data) as u8 + 0x60) as char
    ]
    .iter()
    .collect()
}

/// QuickTime language code values.
#[derive(Debug, Default, BinRead)]
#[br(repr(i16))]
pub enum LangaugeCode {
    English = 0,
    French = 1,
    German = 2,
    Italian = 3,
    Dutch = 4,
    Swedish = 5,
    Spanish = 6,
    Danish = 7,
    Portuguese = 8,
    Norwegian = 9,
    Hebrew = 10,
    Japanese = 11,
    Arabic = 12,
    Finnish = 13,
    Greek = 14,
    Icelandic = 15,
    Maltese = 16,
    Turkish = 17,
    Croatian = 18,
    TraditionalChinese = 19,
    Urdu = 20,
    Hindi = 21,
    Thai = 22,
    Korean = 23,
    Lithuanian = 24,
    Polish = 25,
    Hungarian = 26,
    Estonian = 27,
    // Lettish = 28, // which to use?
    Latvian = 28,
    // Saami = 29, // which to use?
    Sami = 29,
    Faroese = 30,
    Farsi = 31,
    Russian = 32,
    SimplifiedChinese = 33,
    Flemish = 34,
    Irish = 35,
    Albanian = 36,
    Romanian = 37,
    Czech = 38,
    Slovak = 39,
    Slovenian = 40,
    Yiddish = 41,
    Serbian = 42,
    Macedonian = 43,
    Bulgarian = 44,
    Ukrainian = 45,
    Belarusian = 46,
    Uzbek = 47,
    Kazakh = 48,
    Azerbaijani = 49,
    AzerbaijanAr = 50,
    Armenian = 51,
    Georgian = 52,
    Moldavian = 53,
    Kirghiz = 54,
    Tajiki = 55,
    Turkmen = 56,
    Mongolian = 57,
    MongolianCyr = 58,
    Pashto = 59,
    Kurdish = 60,
    Kashmiri = 61,
    Sindhi = 62,
    Tibetan = 63,
    Nepali = 64,
    Sanskrit = 65,
    Marathi = 66,
    Bengali = 67,
    Assamese = 68,
    Gujarati = 69,
    Punjabi = 70,
    Oriya = 71,
    Malayalam = 72,
    Kannada = 73,
    Tamil = 74,
    Telugu = 75,
    Sinhala = 76,
    Burmese = 77,
    Khmer = 78,
    Lao = 79,
    Vietnamese = 80,
    Indonesian = 81,
    Tagalog = 82,
    MalayRoman = 83,
    MalayArabic = 84,
    Amharic = 85,
    // Galla = 87, // which to use?
    Oromo = 87,
    Somali = 88,
    Swahili = 89,
    Kinyarwanda = 90,
    Rundi = 91,
    Nyanja = 92,
    Malagasy = 93,
    Esperanto = 94,
    Welsh = 128,
    Basque = 129,
    Catalan = 130,
    Latin = 131,
    Quechua = 132,
    Guarani = 133,
    Aymara = 134,
    Tatar = 135,
    Uighur = 136,
    Dzongkha = 137,
    JavaneseRom = 138,
    Unspecified21956 = 21956, // found in DJI Osmo Action 4
    #[default]
    Unspecified32767 = 32767,
}

// impl LangaugeCode {
//     pub fn from_i16(value: i16) -> LangaugeCode {
//         Self::from(value)
//     }
// }

// impl TryFrom<i16> for LangaugeCode {
//     type Error;

//     fn try_from(value: i16) -> Result<Self, Self::Error> {
//         i16::try_from(self)
//     }
// }
