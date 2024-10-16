//! Handler reference atom (`hdlr`).
//!
//! > Note: Distinguish from 'Metadata handler atom' with the same FourCC.
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/handler_reference_atom>

use binrw::BinRead;

use crate::support::chars_from_bytes;

/// Handler reference atom (`hdlr`)
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/handler_reference_atom>
#[derive(Debug, Default, BinRead)]
pub struct Hdlr {
    _version: u8,
    _flags: [u8; 3],
    /// Byte 12-15
    /// Possible values:
    /// - `mhlr`: media handler
    /// - `dhlr`: data handler
    /// - `[0, 0, 0, 0]` (DJI Osmo)
    #[br(map(|data: [u8; 4]| chars_from_bytes(data)))]
    pub(crate) component_type: [char; 4],
    /// Four CC for the type of media or data handler
    #[br(map(|data: [u8; 4]| chars_from_bytes(data)))]
    pub(crate) component_sub_type: [char; 4],
    // pub component_sub_type: [u8; 4],
    /// Reserved, should be set to 0.
    pub(crate) component_manufacturer: u32,
    /// Reserved, should be set to 0.
    pub(crate) component_flags: u32,
    /// Reserved, should be set to 0.
    pub(crate) component_flags_mask: u32,
    /// May be a counted string (first byte specifies size),
    /// null terminated string, or neither.
    ///
    /// This field is parsed separately,
    /// since for some old MP4/Quicktimes
    /// `component_name` is not a counted string,
    /// causing the parse to stall.
    #[br(ignore)]
    pub(crate) component_name: String,
}

impl Hdlr {
    /// Returns component type as `String`.
    /// Should be either `mhlr` (media handler),
    /// or `dhlr` (data handler).
    ///
    /// See: <https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-BBCGFGJG>
    pub fn component_type(&self) -> String {
        self.component_type.iter()
            .map(|n| *n as char)
            .collect()
    }


    /// Returns component sub type as `String`.
    /// Should be either `mhlr` (media handler),
    /// or `dhlr` (data handler).
    ///
    /// See: <https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-BBCGFGJG>
    pub fn component_sub_type(&self) -> String {
        self.component_sub_type.iter()
            .map(|n| *n as char)
            .collect()
    }

    pub fn component_manufacturer(&self) -> u32 {
        self.component_manufacturer
    }

    pub fn component_flags(&self) -> u32 {
        self.component_flags
    }

    pub fn component_flags_mask(&self) -> u32 {
        self.component_flags_mask
    }

    pub fn component_name(&self) -> &str {
        self.component_name.as_str()
    }
}

#[derive(Debug)]
pub enum ComponentType {
    Video,
    Sound,
    TimeCode,
    Meta,
    Unknown
}

impl Default for ComponentType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<u32> for ComponentType {
    fn from(value: u32) -> Self {
        match &value.to_be_bytes() {
            b"vide" => Self::Video,
            b"soun" => Self::Sound,
            b"tmcd" => Self::TimeCode,
            b"meta" => Self::Meta,
            _ => Self::Unknown,
        }
    }
}