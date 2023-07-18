//! Media handler atom (`hdlr`).

use binrw::BinRead;

/// Media handler atom.
#[derive(Debug, Default, BinRead)]
pub struct Hdlr {
    _version: u8,
    _flags: [u8; 3],
    /// Byte 12-15
    /// Two possible values:
    /// - `mhlr`: media handler
    /// - `dhlr`: data handler
    pub component_type: [u8; 4],
    // pub component_type: ComponentType,
    /// Four CC for the type of media or data handler
    pub component_sub_type: [u8; 4],
    /// Reserved, should be set to 0.
    pub component_manufacturer: u32,
    /// Reserved, should be set to 0.
    pub component_flags: u32,
    /// Reserved, should be set to 0.
    pub component_flags_mask: u32,
    /// Counted string.
    /// 
    /// Set manually rather than via `BinRead`,
    /// since some (old?) MP4 are not counted strings,
    /// causing the parse to stall.
    #[br(ignore)]
    pub component_name: String,
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
        self.component_type.iter()
            .map(|n| *n as char)
            .collect()
    }

    // /// Returns component name as `String`, e.g. `GoPro TCD`
    // /// for GoPro time code.
    // pub fn name(&self) -> String {
    //     // Possibly not standard ASCII, value between 0-255
    //     self.component_name.iter()
    //         .map(|n| *n as char)
    //         .collect::<String>()
    //         .trim_end() // remove white space padding
    //         .to_owned()
    // }
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