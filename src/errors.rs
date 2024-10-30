//! Various MP4-related errors.

use std::{fmt, num::TryFromIntError};

#[derive(Debug)]
pub enum Mp4Error {
    /// Converted `BinResult` error.
    BinReadError(binrw::Error),
    /// Converted `Utf8Error`.
    Utf8Error(std::string::FromUtf8Error),
    /// IO error
    IOError(std::io::Error),
    /// Filesizes of e.g. 0 sized place holders.
    ReadMismatch{got: u64, expected: u64},
    /// Failed to locate offsets with specified handler name
    /// for interleaved data
    /// from `stts`, `stsz`, and `stco` atoms in MP4.
    /// For e.g. GoPro MP4-files.
    NoSuchHandlerName(String),
    UnscaledDurationError,
    ResolutionExtractionError,
    SampleRateExtractionError,
    VideoFormatExtractionError,
    AudioFormatExtractionError,
    SampleOffsetError,
    MissingHandlerName,
    /// Seek mismatch.
    OffsetMismatch{got: u64, expected: u64},
    /// Missing offsets for `hdlr`.
    NoOffsets(String),
    /// Atom mismatch.
    AtomMismatch{got: String, expected: String},
    /// MP4 0 sized atoms,
    /// e.g. 1k Dropbox place holders.
    UnexpectedAtomSize{len: u64, offset: u64},
    /// No such atom.
    NoSuchAtom(String),
    /// Zero size atom.
    ZeroSizeAtom{name: String, offset: u64},
    /// Atom ouf of bounds.
    /// `(GOT_POS, MIN_POS, MAX_POS)`
    BoundsError(u64, u64, u64),
    /// Filesizes of e.g. 0 sized place holders.
    UnexpectedFileSize(u64),
    /// Unknown base type when parsing `Values`.
    UnknownBaseType(u8),
    /// Missing type definition for Complex type (`63`/`?`)
    MissingComplexType,
    /// Failed to read moov atom.
    MoovReadError,
    /// Exceeded recurse depth when parsing GPMF into `Stream`s
    RecurseDepthExceeded((usize, usize)),
    /// Invalid FourCC. For detecting `&[0, 0, 0, 0]`.
    /// E.g. GoPro `udta` atom contains
    /// mainly undocumented GPMF data and is padded with
    /// zeros.
    InvalidFourCC,
    ZeroLengthVideo,
    ZeroSizeData,
    TryFromInt(TryFromIntError),
    EndOfFile,
}

impl std::error::Error for Mp4Error {}

impl fmt::Display for Mp4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BinReadError(err) => write!(f, "{err}"),
            Self::Utf8Error(err) => write!(f, "{err}"),
            Self::IOError(err) => write!(f, "IO error: {}", err),
            Self::ReadMismatch{got, expected} => write!(f, "Read {got} bytes, expected {expected} bytes."),
            Self::OffsetMismatch{got, expected} => write!(f, "Moved {got} bytes, expected to move {expected} bytes"),
            Self::NoOffsets(fcc) => write!(f, "No offsets found for track with 'hdlr' component name '{fcc}'"),
            Self::UnscaledDurationError => write!(f, "Failed to determine unscaled duration."),
            Self::ResolutionExtractionError => write!(f, "Failed to determine pixel resolution."),
            Self::SampleRateExtractionError => write!(f, "Failed to determine audio sample rate."),
            Self::VideoFormatExtractionError => write!(f, "Failed to determine video format."),
            Self::AudioFormatExtractionError => write!(f, "Failed to determine audio format."),
            Self::SampleOffsetError => write!(f, "Failed to extract sample offsets for track."),
            Self::AtomMismatch{got, expected} => write!(f, "Atom mismatch. Expected '{expected}', got '{got}'"),
            Self::UnexpectedAtomSize{len, offset} => write!(f, "Unexpected MP4 atom size of {len} bytes @ offset {offset}."),
            Self::NoSuchAtom(name) => write!(f, "No such atom {name}."),
            Self::ZeroSizeAtom{name, offset} => write!(f, "Zero size atom '{name}' at offset {offset}."),
            Self::BoundsError(got, start, end) => write!(f, "Bounds error: position {got} is outside boundaries {start} - {end}."),
            Self::UnexpectedFileSize(size) => write!(f, "Unexpected file size of {size} bytes."),
            Self::UnknownBaseType(bt) => write!(f, "Unknown base type {}/'{}'", bt, *bt as char),
            Self::MissingComplexType => write!(f, "Missing type definitions for complex type '?'"),
            Self::RecurseDepthExceeded((depth, max)) => write!(f, "Recurse depth {depth} exceeds max depth {max}"),
            Self::InvalidFourCC => write!(f, "Invalid FourCC"),
            Self::NoSuchHandlerName(hdlr) => write!(f, "No such handler name '{hdlr}'."),
            Self::MissingHandlerName => write!(f, "Failed to determine handler ('hdlr') component name."),
            Self::MoovReadError => write!(f, "Failed to read moov atom."),
            Self::ZeroLengthVideo => write!(f, "Video duration is zero"),
            Self::ZeroSizeData => write!(f, "No data to read"),
            Self::TryFromInt(err) => write!(f, "{err}"),
            Self::EndOfFile => write!(f, "Reached end of file"),
        }
    }
}

/// Converts std::io::Error to Mp4Error
impl From<std::io::Error> for Mp4Error {
    fn from(err: std::io::Error) -> Self {
        Mp4Error::IOError(err)
    }
}

/// Converts std::string::FromUtf8Error to Mp4Error
/// (`&str` reqiures `std::str::Utf8Error`)
impl From<std::string::FromUtf8Error> for Mp4Error {
    fn from(err: std::string::FromUtf8Error) -> Mp4Error {
        Mp4Error::Utf8Error(err)
    }
}

/// Converts Mp4Error to std::io::Error
impl From<Mp4Error> for std::io::Error {
    fn from(err: Mp4Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, err)
    }
}

/// Converts binread::Error to Mp4Error
impl From<binrw::Error> for Mp4Error {
    fn from(err: binrw::Error) -> Mp4Error {
        Mp4Error::BinReadError(err)
    }
}

/// Converts TryFromIntError to Mp4Error
impl From<TryFromIntError> for Mp4Error {
    fn from(err: TryFromIntError) -> Mp4Error {
        Mp4Error::TryFromInt(err)
    }
}