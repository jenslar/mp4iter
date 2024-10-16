//! Edit list atom (`elst`).
//!
//! Location: `moov/trak[multiple]/edts/elst`
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/edit_list_atom>

use binrw::BinRead;

/// Edit list atom (`elst`).
///
/// Location: `moov/trak[multiple]/edts/elst`
///
/// See: <https://developer.apple.com/documentation/quicktime-file-format/edit_list_atom>
#[derive(Debug, Default, BinRead)]
pub struct Elst {
    _version: u8,
    _flags: [u8; 3],
    _number_of_entries: u32,
    #[br(count = _number_of_entries)]
    pub (crate) edit_list_table: Vec<EditListTable>
}

impl Elst {
    pub fn edit_list_table(&self) -> &[EditListTable] {
        &self.edit_list_table
    }
}

#[derive(Debug, Default, BinRead)]
pub struct EditListTable {
    /// Unscaled duration of this edit.
    pub track_duration: u32,
    /// Containing the unscaled starting time within the media of this edit segment.
    /// If set to -1 the edit is empty.
    pub media_time: u32,
    /// Fixed-point number that specifies the relative rate at which to play the media.
    /// Can not be 0 or negative.
    pub media_rate: u32,
}

impl EditListTable {
    pub fn track_duration(&self) -> u32 {
        self.track_duration
    }

    pub fn media_time(&self) -> u32 {
        self.media_time
    }

    pub fn media_rate(&self) -> u32 {
        self.media_rate
    }
}
