//! Sample description. Part of `stsd` atom.

use std::io::Cursor;

use binrw::{BinRead, BinReaderExt};

use crate::{Tmcd, Mp4Error};

use super::{Audio, DataFormat, DataLoad, Video};

#[derive(Debug, BinRead)]
pub struct SampleDescription {
    // General fields. Apply to all stsd atoms.
    // 20 bytes.

    /// Sample description size
    size: u32,
    /// Data format
    #[br(big)]
    #[br(map = |data: u32| DataFormat::new(data))]
    // data_format: u32, // technically 4 character ascii string
    data_format: DataFormat, // technically 4 character ascii string
    /// Reserved. Must be set to 0.
    _reserved: [u8; 6],
    /// Data reference index
    _data_reference_index: u16,

    // Custom fields. Depend on media type and data format.

    // size is total,
    // size of preceding fields (16 bytes) should be subtracted
    #[br(args {size, data_format})]
    data: DataLoad,
}

impl SampleDescription {
    pub fn data(&self) -> &DataLoad {
        &self.data
    }

    pub fn data_format(&self) -> &DataFormat {
        &self.data_format
    }

    pub fn data_format_string(&self) -> String {
        self.data_format.to_string()
    }

    pub(crate) fn tmcd(&self) -> Result<Tmcd, Mp4Error> {
        if self.data_format == DataFormat::Binary(['t','m','c','d']) {
            if let DataLoad::Binary(bytes) = &self.data {
                let mut cursor = Cursor::new(bytes);
                return Ok(cursor.read_ne::<Tmcd>()?)
            }
        }
        Err(Mp4Error::NoSuchAtom("tmcd".to_string()))
    }

    /// Returns true if the sample description is for video.
    pub fn is_video(&self) -> bool {
        self.data_format.is_video()
    }

    /// Returns true if the sample description is for video.
    pub fn video(&self) -> Option<&Video> {
        self.data.video()
    }

    /// Returns true if the sample description is for audio.
    pub fn is_audio(&self) -> bool {
        self.data_format.is_audio()
    }

    /// Returns the audio sample description.
    pub fn audio(&self) -> Option<&Audio> {
        self.data.audio()
    }

    /// Returns true if the sample description is for binary data.
    pub fn is_binary(&self) -> bool {
        self.data_format.is_binary()
    }

    /// Returns the binary sample description as raw bytes.
    pub fn binary(&self) -> Option<&[u8]> {
        self.data.binary()
    }

    /// Returns resolution in pixels if the sample description is for video.
    pub fn resolution(&self) -> Option<(u16, u16)>{
        let video = self.video()?;
        Some((video.width(), video.height()))
    }
}
