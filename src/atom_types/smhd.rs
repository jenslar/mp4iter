//! Sound media information header atom ('smhd')
//!
//! Location: `moov/trak[multiple]/mdia/minf/smhd`
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/sound_media_information_header_atom>

use binrw::BinRead;

/// Sound media information header atom ('smhd')
///
/// Location:
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/sound_media_information_header_atom>
#[derive(Debug, Default, BinRead)]
pub struct Smhd {
    _version: u8,
    _flags: [u8; 3],
    pub(crate) balance: u16,
    pub(crate) reserved: u16
}

impl Smhd {
    pub fn balance(&self) -> u16 {
        self.balance
    }

    pub fn reserved(&self) -> u16 {
        self.reserved
    }
}