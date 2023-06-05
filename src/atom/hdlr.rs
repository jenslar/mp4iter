//! Media handler atom (`hdlr`).

/// Media handler atom.
#[derive(Debug, Default)]
pub struct Hdlr {
    /// Byte 12-15
    // pub component_type: u32,
    pub component_type: ComponentType,
    /// Four CC for the type of media or data handler
    pub component_sub_type: u32,
    /// Reserved, should be set to 0.
    pub component_manufacturer: u32,
    /// Reserved, should be set to 0.
    pub component_flags: u32,
    /// Reserved, should be set to 0.
    pub component_flags_mask: u32,
    /// Counted string. Specifies the name of the component.
    /// May contain a zero-length (empty) string.
    /// First byte contains length of string.
    /// For e.g. GoPro MP4 it's padded with `0x20` (space)
    pub component_name: String,
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