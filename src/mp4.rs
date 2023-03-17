//! Core MP4 struct and methods.
//! 
//! Note on `hdlr` atom and finding "component name"
//! (this crate was developed with the need for parsing GoPro MP4 files, hence the examples below):
//! - The component name is a counted string:
//!     - first byte specifies number of bytes, e.g. "0x0b" = 11, followed by the string.
//!     - For e.g. GoPro the component name for GPMF data "GoPro MET": starts after 8 32-bit fields.
//!     - All GoPro component names end in 0x20 so far: ' ':
//!     - ASCII/Unicode U+0020 (category Zs: Separator, space), so just read as utf-8 read_to_string after counter byte and strip whitespace?
//! 
//! ```rs
//! use mp4iter::Mp4;
//! //! use std::path::Path;
//! 
//! fn main() -> std::io::Result<()> {
//!     let mp4 = Mp4::new(Path::new("VIDEO.MP4"))?;
//!     
//!     // Iterate over atoms. Currently returns `None` on error.
//!     for atom in mp4.into_iter() {
//!         println!("{atom:?}")
//!     }
//!
//!     println!("{:?}", mp4.duration());
//! 
//!     Ok(())
//! }
//! ```

use std::{
    io::{SeekFrom, Cursor, Read, Seek},
    fs::{Metadata, File},
    path::Path,
    borrow::BorrowMut
};

use binread::{
    BinReaderExt,
    BinRead,
    endian::Endian
};
use time::ext::NumericalDuration;

use crate::{
    errors::Mp4Error,
    atom::{Atom, Co64},
    fourcc::FourCC,
    Offset,
    Stts,
    Stsz,
    Stco,
    Hdlr,
    Udta,
    AtomHeader,
};

/// Mp4 file.
pub struct Mp4{
    /// Open MP4 file.
    file: File,
    /// File size in bytes.
    pub len: u64
}

impl Iterator for Mp4 {
    type Item = AtomHeader;

    /// Non-fallible iterator over atom offsets.
    /// Returns `None` for errors.
    fn next(&mut self) -> Option<Self::Item> {
        let header = self.next().ok()?;

        Some(header)
    }
}

impl Mp4 {
    /// New Mp4 from path. Sets offset to 0.
    /// Offset changes while parsing and must always
    /// be the start of an atom.
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let len = file.metadata()?.len(); // to avoid repeated sys calls
        Ok(Self{
            file,
            len
        })
    }

    /// Returns MP4 file size in bytes.
    /// To avoid a fallible call,
    /// use the public field `MP4::len` instead
    pub fn len(&self) -> std::io::Result<u64> {
        Ok(self.file.metadata()?.len())
    }

    /// Returns current position/byte offset in MP4 file.
    pub fn pos(&mut self) -> std::io::Result<u64> {
        self.file.seek(SeekFrom::Current(0))
    }

    /// Read bytes into specified type at current position.
    pub fn read_type<T: Sized + BinRead>(
        &mut self,
        len: u64,
        endian: Endian
    ) -> Result<T, Mp4Error> {
        match endian {
            Endian::Big => self.read(len)?.read_be::<T>().map_err(|err| err.into()),
            Endian::Little => self.read(len)?.read_le::<T>().map_err(|err| err.into()),
            Endian::Native => self.read(len)?.read_ne::<T>().map_err(|err| err.into())
        }
    }

    /// Read bytes into specified type at specified position.
    pub fn read_type_at<T: Sized + BinRead>(
        &mut self,
        len: u64,
        offset_from_start: u64,
        endian: Endian
    ) -> Result<T, Mp4Error> {
        self.seek_to(offset_from_start)?;
        self.read_type::<T>(len, endian)
    }

    /// Reads specified number of bytes at current position,
    /// and returns these as `Cursor<Vec<u8>>`.
    pub fn read(&mut self, len: u64) -> Result<Cursor<Vec<u8>>, Mp4Error> {
        let mut chunk = self.file.borrow_mut().take(len);
        let mut data = Vec::with_capacity(len as usize);
        let read_len = chunk.read_to_end(&mut data)? as u64;

        if read_len != len {
            return Err(Mp4Error::ReadMismatch{got: read_len, expected: len})
        } else {
            Ok(Cursor::new(data))
        }
    }

    /// Reads `len` number of bytes
    /// at specified position/byte offset `pos` from start of MP4,
    /// and returns these as `Cursor<vec<u8>>`.
    pub fn read_at(&mut self, pos: u64, len: u64) -> Result<Cursor<Vec<u8>>, Mp4Error> {
        self.seek_to(pos)?;
        self.read(len)
    }

    /// Seeks back or forth relative to current position.
    pub fn seek(&mut self, offset_from_current: i64) -> Result<(), Mp4Error> {
        let pos_seek = self.file.seek(SeekFrom::Current(offset_from_current))?;
        let pos = self.pos()?;
        if pos_seek != pos {
            return Err(Mp4Error::OffsetMismatch{got: pos_seek as u64, expected: pos})
        }
        Ok(())
    }

    /// Seeks from start.
    pub fn seek_to(&mut self, offset_from_start: u64) -> Result<(), Mp4Error> {
        let pos_seek = self.file.seek(SeekFrom::Start(offset_from_start))?;
        let pos = self.pos()?;
        if pos_seek != pos {
            return Err(Mp4Error::OffsetMismatch{got: pos_seek as u64, expected: pos})
        }
        Ok(())
    }

    // /// Check if len to read exceeds file size.
    // pub fn check_bounds(&mut self, len_to_try: u64) -> Result<(), Mp4Error> {
    //     // TODO 221016 bounds check not working as expected.
    //     // TODO        bounds error for max360/fusion if used (but not max-heromode),
    //     // TODO        but parses fine if commented out...
    //     // TODO        is self.len/size incorrectly set/used?
    //     // TODO        no error for hero-series if self.check_bounds is used
    //     // TODO        something in multi-device (Fusion/Max) gpmf structure that causes this?
    //     let aim = self.pos()? + len_to_try;
    //     let len = self.len;
    //     if aim > len {
    //         Err(Mp4Error::BoundsError((aim, len)))
    //     } else {
    //         Ok(())
    //     }
    // }

    pub fn next(&mut self) -> Result<AtomHeader, Mp4Error> {
        // 8 or 16 bytes header depending on whether 32 or 64bit sized atom
        let header = self.header()?;
        let iter_size = header.offset_next();
        self.seek(iter_size as i64)?; // casting u64 as i64...
        
        Ok(header)
    }

    /// Return atom header at current offset.
    /// No check is made to verify that current offset
    /// is at the start of an atom.
    pub fn header(&mut self) -> Result<AtomHeader, Mp4Error> {
        // Get offset before reading header
        let offset = self.pos()?;
        // Read 32bit atom size
        let mut size = self.read(4)?.read_be::<u32>()? as u64;
        // Read FourCC. u8 is read as char to prevent fourcc
        // that use ISO8859-1 (e.g. GoPro) to overflow ASCII range
        let string: String = self.read(4)?
            .into_inner()
            .iter()
            .map(|n| *n as char)
            .collect();
        let fourcc = FourCC::from_str(&string);
        // Set atom offset to position before header
        // Check if atom size is 64bit and read the 8 bytes
        // following directly after FourCC as new size if so
        if size == 1 {
            size = self.read(8)?.read_be::<u64>()?;
        }
        Ok(AtomHeader {
            size,
            name: fourcc,
            offset,
        })
    }

    /// Read data for atom at current offset as a single chunk.
    /// Note that e.g. the `mdat` atom may be many GB in size,
    /// and that raw data is read into memory as `Cursor<Vecu8>>`.
    pub fn atom(&mut self, header: &AtomHeader) -> Result<Atom, Mp4Error> {
        let cursor = self.read_at(header.data_offset(), header.data_size())?;

        Ok(Atom{
            header: header.to_owned(),
            cursor
        })
    }

    /// Finds first top-level atom with specified name (FourCC),
    /// then returns and sets `Mp4.offset` to start of that atom.
    /// 
    /// `Mp4::find()` will continue from current offset.
    /// Run `Mp4::reset()` to set start offset to 0.
    pub fn find(&mut self, name: &str) -> Result<Option<AtomHeader>, Mp4Error> {
        // TODO does not check self at current offset, only from next and on...
        while let Ok(header) = self.next() {
            if header.name.to_str() == name {
                return Ok(Some(header))
            }
        }
        Ok(None)
    }

    /// Exctract time to sample values (`stts` atom - one for each `trak`).
    /// 
    /// Path: `moov.trak<multiple>.mdia.minf.stbl.stts`
    pub fn stts(&mut self) -> Result<Stts, Mp4Error> {
        if let Some(header) = self.find("stts")? {
            self.atom(&header)?.stts()
        } else {
            Err(Mp4Error::NoSuchAtom("stts".to_owned()))
        }
    }

    /// Exctract sample to size values (`stsz` atom - one for each `trak`).
    /// 
    /// Path: `moov.trak<multiple>.mdia.minf.stbl.stsz`
    pub fn stsz(&mut self) -> Result<Stsz, Mp4Error> {
        if let Some(header) = self.find("stsz")? {
            self.atom(&header)?.stsz()
        } else {
            Err(Mp4Error::NoSuchAtom("stsz".to_owned()))
        }
    }

    /// Exctract chunk offset values for files below 32bit limit
    /// (`stco` atom - one for each `trak`).
    /// 
    /// Path: `moov.trak<multiple>.mdia.minf.stbl.stco`
    pub fn stco(&mut self) -> Result<Stco, Mp4Error> {
        if let Some(header) = self.find("stco")? {
            self.atom(&header)?.stco()
        } else {
            Err(Mp4Error::NoSuchAtom("stco".to_owned()))
        }
    }

    /// Exctract chunk offset values for files above 32bit limit
    /// (`co64` atom - one for each `trak`).
    /// 
    /// Path: `moov.trak (multiple).mdia.minf.stbl.stco`
    pub fn co64(&mut self) -> Result<Co64, Mp4Error> {
        if let Some(header) = self.find("co64")? {
            self.atom(&header)?.co64()
        } else {
            Err(Mp4Error::NoSuchAtom("co64".to_owned()))
        }
    }

    /// Exctract media handler values (`hdlr` atom).
    /// 
    /// Path: `moov.trak (multiple).mdia.hdlr`
    pub fn hdlr(&mut self) -> Result<Hdlr, Mp4Error> {
        if let Some(header) = self.find("hdlr")? {
            self.atom(&header)?.hdlr()
        } else {
            Err(Mp4Error::NoSuchAtom("hdlr".to_owned()))
        }
    }

    /// Extract user data atom (`udta`).
    /// Some vendors embed data such as device info,
    /// unique identifiers (Garmin VIRB UUID),
    /// or even data in vendor specific formats
    /// (GoPro undocumented GPMF data, separate from
    /// the main GPMF telemetry interleaved in the `mdat` atom).
    /// 
    /// Path: `moov.udta`
    pub fn udta(&mut self) -> Result<Udta, Mp4Error> {
        // Set position to start of file to avoid
        // previous reads to have moved the cursor
        // past the 'udta' atom.
        self.reset()?;
        
        if let Some(header) = self.find("udta")? {
            self.atom(&header)?.udta()
        } else {
            Err(Mp4Error::NoSuchAtom("udta".to_owned()))
        }
    }

    /// Returns duration of MP4.
    /// Derived from `mvhd` atom (inside `moov` atom),
    /// which lists duration for whichever track is the longest.
    pub fn duration(&mut self) -> Result<time::Duration, Mp4Error> {
        // ensure search is done from beginning of file
        self.reset()?;
        // Find 'mvhd' atom (inside 'moov' atom)
        if let Some(header) = self.find("mvhd")? {
            // seek to start of 'mvhd' + until 'time scale' field
            self.seek_to(header.data_offset() + 4)?; // old was offset + 20 using start of atom
            
            // Read time scale value and scaled duration, normalises to seconds
            let time_scale = self.read(4)?.read_be::<u32>()?;
            let scaled_duration = self.read(4)?.read_be::<u32>()?;

            // Generate 'time::Duration' from normalised duration
            let duration = (scaled_duration as f64 / time_scale as f64).seconds();
    
            Ok(duration)
        } else {
            Err(Mp4Error::NoSuchAtom("mvhd".to_owned()))
        }
    }

    /// Seek to start of file and set `self.offset = 0`.
    pub fn reset(&mut self) -> Result<(), Mp4Error> {
        self.seek_to(0)?;
        Ok(())
    }

    /// Returns file `std::fs::Metadata`.
    pub fn meta(&self) -> std::io::Result<Metadata> {
        self.file.metadata()
    }

    /// Extract byte offsets, byte sizes, and time/duration
    /// for track handler with specified `handler_name` ('hdlr' atom inside 'moov/trak' atom).
    /// Supports 64bit file sizes.
    /// 
    /// E.g. use `handler_name` "GoPro MET" for locating interleaved GoPro GPMF data in MP4.
    pub fn offsets(&mut self, handler_name: &str) -> Result<Vec<Offset>, Mp4Error> {
        // self.reset()?;
        while let Ok(hdlr) = self.hdlr() {
            
            if &hdlr.component_name == handler_name {
                // TODO 230112 order of 'st..' atoms consistent?
                // TODO        possible solution: find hdlr, read hdlr as atom, then inside hdlr cursor find stts etc
                let stts = self.stts()?;
                let stsz = self.stsz()?;
                // Check if file size > 32bit limit,
                // but always output 64bit offsets
                let stco = match self.len > u32::MAX as u64 {
                    true => self.co64()?,
                    false => Co64::from(self.stco()?),
                };

                // Assert equal size of all contained Vec:s to allow iter in parallel
                assert_eq!(stco.len(), stsz.len(), "'stco' and 'stsz' atoms differ in data size");
                assert_eq!(stts.len(), stsz.len(), "'stts' and 'stsz' atoms differ in data size");

                let offsets: Vec<Offset> = stts.iter()
                    .zip(stsz.iter())
                    .zip(stco.iter())
                    .map(|((duration, size), position)| Offset::new(*position, *size, *duration))
                    .collect();

                return Ok(offsets)
            }
        }

        Err(Mp4Error::MissingHandler(handler_name.to_owned()))
    }
}