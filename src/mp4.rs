//! Core MP4 struct and methods.
//! 
//! If an atom can not be located, try running `Mp4::reset()` first to set offset to 0.
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
//! use std::path::Path;
//! 
//! fn main() -> std::io::Result<()> {
//!     let mp4 = Mp4::new(Path::new("VIDEO.MP4"))?;
//!     
//!     for atom_header in mp4.into_iter() {
//!         println!("{atom_header:?}")
//!     }
//!
//!     // Derives duration for MP4 for longest track.
//!     println!("{:?}", mp4.duration());
//! 
//!     // Extracts offsets for GoPro GPMF telemetry (handlre name 'GoPro MET')
//!     println!("{:#?}", mp4.offsets("GoPro MET"));
//! 
//!     Ok(())
//! }
//! ```

use std::{
    io::{SeekFrom, Cursor, Read, Seek, BufReader, BufRead},
    fs::File,
    path::{Path, PathBuf},
};

use binrw::{
    BinReaderExt,
    BinRead,
    endian::Endian
};

use crate::{
    errors::Mp4Error,
    atom::{Atom, Co64, Tmcd, Mvhd},
    fourcc::FourCC,
    Offset,
    Stts,
    Stsz,
    Stco,
    Hdlr,
    Udta,
    AtomHeader,
};

/// Mp4 "file". Wrapper around a `BufReader`.
#[derive(Debug)]
pub struct Mp4{
    /// Path
    pub path: PathBuf,
    /// BufReader around open file.
    reader: BufReader<File>,
    /// File size in bytes.
    len: u64
}

impl Read for Mp4 {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl BufRead for Mp4 {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt)
    }
}

impl Seek for Mp4 {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

impl Iterator for Mp4 {
    type Item = AtomHeader;

    /// Non-fallible iterator over atom header.
    /// Returns `None` for errors.
    fn next(&mut self) -> Option<Self::Item> {
        // Read atom header, seek to position of next atom
        let header = self.next_header(true).ok()?;
        Some(header)
    }
}

impl Mp4 {
    /// New `Mp4` from path.
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let len = file.metadata()?.len(); // to avoid repeated sys calls
        Ok(Self{
            path: path.to_owned(),
            reader: BufReader::new(file),
            len
        })
    }

    /// Mp4 file size in bytes.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns current position/byte offset in MP4 file.
    pub fn pos(&mut self) -> std::io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }

    /// Seek to start of MP4 file.
    pub fn reset(&mut self) -> Result<u64, Mp4Error> {
        // self.seek_to(0)
        self.seek(SeekFrom::Start(0)).map_err(|e| e.into())
    }

    /// Reads `len` bytes starting at current position
    /// and returns these as `Cursor<Vec<u8>>`
    pub fn read_len(&mut self, len: u64) -> Result<Vec<u8>, Mp4Error> {
        let mut data = vec![0; len as usize];
        let read_len = self.read(&mut data)?;

        if read_len as u64 != len {
            return Err(Mp4Error::ReadMismatch{got: read_len as u64, expected: len})
        } else {
            Ok(data)
        }
    }

    /// Reads `len` bytes at absolute position `pos`,
    /// and returns these as `Vec<u8>`.
    pub fn read_len_at(&mut self, pos: u64, len: u64) -> Result<Vec<u8>, Mp4Error> {
        self.seek(SeekFrom::Start(pos))?;
        self.read_len(len)
    }

    /// Read bytes as type `T` at current position
    /// with specified endian.
    pub fn read_type<T>(
        &mut self,
        endian: Endian
    ) -> Result<T, Mp4Error> 
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        match endian {
            Endian::Big => self.read_be::<T>().map_err(|err| err.into()),
            Endian::Little => self.read_le::<T>().map_err(|err| err.into()),
            // Endian::Native => self.reader.read_ne::<T>().map_err(|err| err.into())
        }
    }

    /// Read bytes as type `T` at absolute position `pos`
    /// with specified endian.
    pub fn read_type_at<T>(
        &mut self,
        pos: u64,
        endian: Endian
    ) -> Result<T, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.seek(SeekFrom::Start(pos))?;
        self.read_type::<T>(endian)
    }

    /// Read BigEndian type `T` at current position.
    pub fn read_be<T>(&mut self) -> Result<T, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.reader.read_be::<T>().map_err(|e| e.into())
    }

    /// Read `n` number of BigEndian type `T` at current position.
    pub fn read_n_be<T>(&mut self, n: usize) -> Result<Vec<T>, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        (0..n).into_iter()
            .map(|_| self.read_be::<T>())
            .collect()
    }

    /// Read BigEndian type `T` at `pos` position.
    pub fn read_be_at<T>(&mut self, pos: u64) -> Result<T, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        // self.seek_to(pos)?;
        self.seek(SeekFrom::Start(pos))?;
        self.read_be::<T>()
    }

    /// Read `n` number of BigEndian type `T` at `pos` position.
    pub fn read_n_be_at<T>(&mut self, n: usize, pos: u64) -> Result<Vec<T>, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.seek(SeekFrom::Start(pos))?;
        self.read_n_be::<T>(n)
    }

    /// Read LittleEndian type `T` at current position.
    pub fn read_le<T>(&mut self) -> Result<T, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.reader.read_le::<T>().map_err(|e| e.into())
    }

    /// Read `n` number of LittleEndian type `T` at current position.
    pub fn read_n_le<T>(&mut self, n: usize) -> Result<Vec<T>, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        (0..n).into_iter()
            .map(|_| self.read_le::<T>())
            .collect()
    }

    /// Read LittleEndian type `T` at `pos` position.
    pub fn read_le_at<T>(&mut self, pos: u64) -> Result<T, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.seek(SeekFrom::Start(pos))?;
        self.read_le::<T>()
    }

    /// Read `n` number of LittleEndian type `T` at `pos` position.
    pub fn read_n_le_at<T>(&mut self, n: usize, pos: u64) -> Result<Vec<T>, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.seek(SeekFrom::Start(pos))?;
        self.read_n_le::<T>(n)
    }

    /// Read NativeEndian type `T` at current position.
    pub fn read_ne<T>(&mut self) -> Result<T, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.reader.read_ne::<T>().map_err(|e| e.into())
    }

    /// Read NativeEndian type `T` at `pos` position.
    pub fn read_ne_at<T>(&mut self, pos: u64) -> Result<T, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.seek(SeekFrom::Start(pos))?;
        self.read_ne::<T>()
    }

    /// Read `n` number of NativeEndian type `T` at current position.
    pub fn read_n_ne<T>(&mut self, n: usize) -> Result<Vec<T>, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        (0..n).into_iter()
            .map(|_| self.read_ne::<T>())
            .collect()
    }

    /// Read `n` number of NativeEndian type `T` at `pos` position.
    pub fn read_n_ne_at<T>(&mut self, n: usize, pos: u64) -> Result<Vec<T>, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.seek(SeekFrom::Start(pos))?;
        self.read_n_ne::<T>(n)
    }

    /// Read `len` number of bytes to string.
    pub fn read_string(&mut self, len: u64) -> Result<String, Mp4Error> {
        let mut buf = String::new();
        let _len = self.read_len(len)?.as_slice().read_to_string(&mut buf)?;
        Ok(buf)
    }

    /// Read `len` number of bytes to string at absolution position `pos`.
    pub fn read_string_at(&mut self, len: u64, pos: u64) -> Result<String, Mp4Error> {
        self.seek(SeekFrom::Start(pos))?;
        self.read_string(len)
    }

    /// Reads `len` bytes starting at current position
    /// and returns these as `Cursor<Vec<u8>>`.
    /// Note that the `mdat` atom may be many GB in size.
    pub fn cursor(&mut self, len: u64) -> Result<Cursor<Vec<u8>>, Mp4Error> {
        Ok(Cursor::new(self.read_len(len)?))
    }

    /// Reads `len` bytes starting at `pos` position
    /// and returns these as `Cursor<Vec<u8>>`
    pub fn cursor_at(&mut self, pos: u64, len: u64) -> Result<Cursor<Vec<u8>>, Mp4Error> {
        self.seek(SeekFrom::Start(pos))?;
        Ok(Cursor::new(self.read_len(len)?))
    }

    /// `next` method for iterating over atoms.
    pub fn next_header(&mut self, seek_next: bool) -> Result<AtomHeader, Mp4Error> {
        let header = self.header()?;

        // Seek to next header offset. Do this for iter impl etc.
        if seek_next {
            self.seek(SeekFrom::Current(header.next as i64))?; // casting u64 as i64...
        }
        
        Ok(header)
    }

    /// Return atom header at current offset.
    /// No check is made to verify that current offset
    /// is at the start of an atom.
    pub fn header(&mut self) -> Result<AtomHeader, Mp4Error> {
        AtomHeader::new(self)
    }

    /// Create atom at current offset as a single chunk.
    /// Assumes offset is at atom data load position.
    /// 
    /// If this method is used in isolation, calling `Atom::reset()`,
    /// sets offset to data load position.
    pub fn atom(&mut self, header: &AtomHeader) -> Atom {
        Atom::new(header, &mut self.reader)
    }

    /// Finds first top-level atom with specified name (FourCC).
    /// 
    /// `Mp4::find()` will continue from current offset.
    /// Run `Mp4::reset()` to set start offset to 0.
    pub fn find(&mut self, name: &str, reset: bool) -> Result<Option<AtomHeader>, Mp4Error> {
        if reset {
            self.reset()?;
        }
        // TODO does not check self at current offset, only from next and on...
        // Iterate, but set seek to next atom offset to false...
        let fourcc = FourCC::from_str(name);
        while let Ok(header) = self.next_header(false) {
            // if header.name.to_str() == name {
            if header.name == fourcc {
                return Ok(Some(header))
            }
            // ... and if not the correct atom, seek to next one manually
            // self.seek(header.next as i64)?;
            self.seek(SeekFrom::Current(header.next as i64))?;
        }
        Ok(None)
    }

    /// Exctract time to sample values for first encounterd
    /// `stts` atom at current position (one `stts` for each `trak`).
    /// 
    /// Path: `moov.trak<multiple>.mdia.minf.stbl.stts`
    pub fn stts(&mut self, reset: bool) -> Result<Stts, Mp4Error> {
        if let Some(header) = self.find("stts", reset)? {
            self.atom(&header).stts()
        } else {
            Err(Mp4Error::NoSuchAtom("stts".to_owned()))
        }
    }

    /// Exctract sample to size values (`stsz` atom - one for each `trak`).
    /// 
    /// Path: `moov.trak<multiple>.mdia.minf.stbl.stsz`
    pub fn stsz(&mut self, reset: bool) -> Result<Stsz, Mp4Error> {
        if let Some(header) = self.find("stsz", reset)? {
            self.atom(&header).stsz()
        } else {
            Err(Mp4Error::NoSuchAtom("stsz".to_owned()))
        }
    }

    /// Exctract chunk offset values for files below 32bit limit
    /// (`stco` atom - one for each `trak`).
    /// 
    /// Path: `moov.trak<multiple>.mdia.minf.stbl.stco`
    pub fn stco(&mut self, reset: bool) -> Result<Stco, Mp4Error> {
        if let Some(header) = self.find("stco", reset)? {
            self.atom(&header).stco()
        } else {
            Err(Mp4Error::NoSuchAtom("stco".to_owned()))
        }
    }

    /// Exctract chunk offset values for files above 32bit limit
    /// (`co64` atom - one for each `trak`).
    /// 
    /// Path: `moov.trak (multiple).mdia.minf.stbl.stco`
    pub fn co64(&mut self, reset: bool) -> Result<Co64, Mp4Error> {
        if let Some(header) = self.find("co64", reset)? {
            self.atom(&header).co64()
        } else {
            Err(Mp4Error::NoSuchAtom("co64".to_owned()))
        }
    }

    /// Exctract media handler values (`hdlr` atom).
    /// 
    /// Path: `moov.trak (multiple).mdia.hdlr`
    pub fn hdlr(&mut self, reset: bool) -> Result<Hdlr, Mp4Error> {
        if let Some(header) = self.find("hdlr", reset)? {
            self.atom(&header).hdlr()
        } else {
            Err(Mp4Error::NoSuchAtom("hdlr".to_owned()))
        }
    }

    /// Exctract Movie header atom (`mvhd` atom).
    /// 
    /// Path: `moov.mvhd`
    pub fn mvhd(&mut self, reset: bool) -> Result<Mvhd, Mp4Error> {
        if let Some(header) = self.find("mvhd", reset)? {
            self.atom(&header).mvhd()
        } else {
            Err(Mp4Error::NoSuchAtom("mvhd".to_owned()))
        }
    }

    /// Returns timecode data to derive start time of video.
    /// 
    /// For GoPro, use `GoPro TCD` as handler name.
    pub fn tmcd(&mut self, handler_name: &str) -> Result<Tmcd, Mp4Error> {
        // !!! this loop fails after first loop with /Users/jens/Dropbox/DEV/TESTDATA/Video/archivable_mp4_avc_aac/NarrTrad1.mp4
        while let Ok(hdlr) = self.hdlr(false) {
            // 1. find the `trak` with specified handler name
            // if &hdlr.name() == handler_name {
            if &hdlr.component_name == handler_name {
                // 2. find sample description atom
                if let Some(stsd_header) = self.find("stsd", false)? {
                    // seek to start of 'number of entries' field
                    // self.seek_to(stsd_header.data_offset() + 4)?;
                    self.seek(SeekFrom::Start(stsd_header.data_offset() + 4))?;

                    // 3. iterate sample descriptions (same general layout as any atom)
                    let no_of_entries = self.read_be::<u32>()?;
                    
                    for _i in (0..no_of_entries as usize).into_iter() {
                        let header = self.header()?;

                        if header.name == FourCC::Tmcd {
                            let mut tmcd = self.atom(&header).tmcd()?;
                            // !!! Duration is unscaled.
                            // !!! To get correct duration: offset.duration / tmcd.time_scale
                            tmcd.offsets = self.offsets_at_current_pos()?;
                            return Ok(tmcd)
                        }

                        // self.seek(header.offset_next_rel() as i64)?; // u64 cast as i64...
                        self.seek(SeekFrom::Current(header.offset_next_rel() as i64))?; // u64 cast as i64...
                    }
                }
            }
        }

        Err(Mp4Error::MissingHandler(handler_name.to_owned()))
    }

    /// Extract user data atom (`udta`).
    /// Some vendors embed data such as device info,
    /// unique identifiers (Garmin VIRB UUID),
    /// or even data in vendor specific formats
    /// (GoPro undocumented GPMF data, separate from
    /// the main GPMF telemetry interleaved in the `mdat` atom).
    /// 
    /// Path: `moov.udta`
    pub fn udta(&mut self, reset: bool) -> Result<Udta, Mp4Error> {
        // Set reset to true, position to start of file to avoid
        // previous reads to have moved the cursor
        // past the 'udta' atom.
        if let Some(header) = self.find("udta", reset)? {
            self.atom(&header).udta()
        } else {
            Err(Mp4Error::NoSuchAtom("udta".to_owned()))
        }
    }

    /// Returns duration of MP4.
    /// Derived from `mvhd` atom (inside `moov` atom),
    /// which lists duration for whichever track is the longest.
    /// 
    /// Reference `mvhd`: <https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-SW1>
    pub fn duration(&mut self, reset: bool) -> Result<time::Duration, Mp4Error> {
        let mvhd = self.mvhd(reset)?;
        Ok(mvhd.duration())
    }

    /// Returns creation time as UTC datetime and duration in seconds
    /// as the tuple `(START_TIME, DURATION)`.
    /// 
    /// Derived from `mvhd` atom,
    /// which lists duration for whichever track is the longest.
    /// 
    /// Note the some recording devices, such as GoPro may have the same
    /// `START_TIME` for all clips in the same session. This depends on the
    /// exact model. For these, find the `trak` with the title `GoPro TCD` instead
    /// and use the timecode data in there (`tmcd` entry in an `stsd` atom).
    pub fn time(&mut self, reset: bool) -> Result<(time::PrimitiveDateTime, time::Duration), Mp4Error> {
        let mvhd = self.mvhd(reset)?;
        Ok((mvhd.creation_time(), mvhd.duration()))
    }

    /// Extract byte offsets, byte sizes, and time/duration
    /// for track handler with specified `handler_name` ('hdlr' atom inside 'moov/trak' atom).
    /// Supports 64bit file sizes.
    /// 
    /// E.g. use `handler_name` "GoPro MET" for locating interleaved GoPro GPMF data in MP4.
    pub fn offsets(&mut self, handler_name: &str) -> Result<Vec<Offset>, Mp4Error> {
        // Something goes wrong with /Users/jens/Dropbox/DEV/TESTDATA/Video/archivable_mp4_avc_aac/NarrTrad1.mp4
        self.reset()?;
        while let Ok(hdlr) = self.hdlr(false) {
            if &hdlr.component_name == handler_name {
                return self.offsets_at_current_pos()
            }
        }
        Err(Mp4Error::MissingHandler(handler_name.to_owned()))
    }

    /// Internal. Returns offsets for current `trak`.
    /// I.e. assumes upcoming atoms are `stts` (sample to time),
    /// `stsz` (sample to size), `stco` (sample to offset) atoms, and process these.
    fn offsets_at_current_pos(&mut self) -> Result<Vec<Offset>, Mp4Error> {
        // TODO 230112 order of 'st..' atoms consistent?
        // TODO        possible solution: find hdlr, read hdlr as atom, then inside hdlr cursor find stts etc
        let stts = self.stts(false)?.expand();
        let stsz = self.stsz(false)?.expand();
        // Check if file size > 32bit limit,
        // but always output 64bit offsets
        let stco = match self.len > u32::MAX as u64 {
            true => self.co64(false)?.expand(),
            false => Co64::from(self.stco(false)?).expand(),
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