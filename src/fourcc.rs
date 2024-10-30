//! MP4 atom FourCC.
//! See atom type in <https://developer.apple.com/documentation/quicktime-file-format/atoms>.
//! Some non-standard Four CC listed, stemming from e.g. GoPro MP4-files.

use std::fmt::Display;

/// MP4 atom Four CC.
/// See atom type in <https://developer.apple.com/documentation/quicktime-file-format/atoms>.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FourCC {
    /// Composition offset atom
    Ctts,
    /// Data Information Atoms
    Dinf,
    Dref,
    Edts,
    Elst,
    Ftyp,
    Free,
    Gmhd,
    Hdlr,
    Iods,
    Mdat,
    Mdhd,
    Mdia,
    Minf,
    /// Movie Atom
    Moov,
    /// Movie Header Atom
    Mvhd,
    Smhd,
    Stbl,
    /// Chunk offset, 32-bit values
    Stco,
    /// Chunk offset, 64-bit values
    Co64,
    Sdtp,
    Stsc,
    Stsd,
    Stss,
    Stsz,
    Stts,
    Tkhd,
    Tmcd,
    /// Track description
    Trak,
    Tref,
    /// User data
    Udta,
    Vmhd,

    Custom(String),
}

impl Display for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl FourCC {
    pub fn from_slice(fourcc: &[u8]) -> Self {
        assert_eq!(fourcc.len(), 4, "FourCC must have size 4.");
        match fourcc {
            // Atoms
            b"dinf" => Self::Dinf,
            b"dref" => Self::Dref,
            b"edts" => Self::Edts,
            b"elst" => Self::Elst,
            b"ftyp" => Self::Ftyp,
            b"gmhd" => Self::Gmhd,
            b"hdlr" => Self::Hdlr,
            b"iods" => Self::Iods,
            b"mdat" => Self::Mdat,
            b"mdhd" => Self::Mdhd,
            b"mdia" => Self::Mdia,
            b"minf" => Self::Minf,
            b"moov" => Self::Moov,
            b"mvhd" => Self::Mvhd,
            b"smhd" => Self::Smhd,
            b"sdtp" => Self::Sdtp,
            b"stbl" => Self::Stbl,
            b"stco" => Self::Stco,
            b"stsc" => Self::Stsc,
            b"stsd" => Self::Stsd,
            b"stss" => Self::Stss,
            b"stsz" => Self::Stsz,
            b"stts" => Self::Stts,
            b"tkhd" => Self::Tkhd,
            b"trak" => Self::Trak,
            b"tref" => Self::Tref,
            b"udta" => Self::Udta,
            b"vmhd" => Self::Vmhd,
            b"co64" => Self::Co64,

            // Atom-internal data structures
            b"tmcd" => Self::Tmcd,

            // UTF-8 does not work for single-byte char above 127
            // but ISO8859-1 mapping works for range 128-255
            _ => Self::Custom(
                fourcc
                    .iter()
                    .map(|n| *n as char)
                    .collect::<String>()
                ),
        }
    }

    pub fn from_u32(value: u32) -> Self {
        Self::from_slice(&value.to_be_bytes())
    }

    pub fn from_str(fourcc: &str) -> Self {
        match fourcc {
            "ctts" => Self::Ctts,
            "dinf" => Self::Dinf,
            "dref" => Self::Dref,
            "edts" => Self::Edts,
            "elst" => Self::Elst,
            "ftyp" => Self::Ftyp,
            "gmhd" => Self::Gmhd,
            "hdlr" => Self::Hdlr,
            "iods" => Self::Iods,
            "mdat" => Self::Mdat,
            "mdhd" => Self::Mdhd,
            "mdia" => Self::Mdia,
            "minf" => Self::Minf,
            "moov" => Self::Moov,
            "mvhd" => Self::Mvhd,
            "sdtp" => Self::Sdtp,
            "smhd" => Self::Smhd,
            "stbl" => Self::Stbl,
            "stco" => Self::Stco,
            "stsc" => Self::Stsc,
            "stsd" => Self::Stsd,
            "stss" => Self::Stss,
            "stsz" => Self::Stsz,
            "stts" => Self::Stts,
            "tkhd" => Self::Tkhd,
            "tmcd" => Self::Tmcd,
            "trak" => Self::Trak,
            "tref" => Self::Tref,
            "udta" => Self::Udta,
            "vmhd" => Self::Vmhd,
            "co64" => Self::Co64,
            _ => Self::Custom(fourcc.to_owned()),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Ctts => "ctts",
            Self::Dinf => "dinf",
            Self::Dref => "dref",
            Self::Edts => "edts",
            Self::Elst => "elst",
            Self::Ftyp => "ftyp",
            Self::Free => "free",
            Self::Gmhd => "gmhd",
            Self::Hdlr => "hdlr",
            Self::Iods => "iods",
            Self::Mdat => "mdat",
            Self::Mdhd => "mdhd",
            Self::Mdia => "mdia",
            Self::Minf => "minf",
            Self::Moov => "moov",
            Self::Mvhd => "mvhd",
            Self::Sdtp => "sdtp",
            Self::Smhd => "smhd",
            Self::Stbl => "stbl",
            Self::Stco => "stco",
            Self::Stsc => "stsc",
            Self::Stsd => "stsd",
            Self::Stss => "stss",
            Self::Stsz => "stsz",
            Self::Stts => "stts",
            Self::Tkhd => "tkhd",
            Self::Tmcd => "tmcd",
            Self::Trak => "trak",
            Self::Tref => "tref",
            Self::Udta => "udta",
            Self::Vmhd => "vmhd",
            Self::Co64 => "co64",
            // Self::Gpmf => "GPMF", // capitals in file
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl Default for FourCC {
    fn default() -> Self {
        Self::Custom("Unknown".to_owned())
    }
}