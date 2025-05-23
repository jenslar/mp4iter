//! Core MP4 struct and methods.
//!
//! If an atom can not be located, try running `Mp4::reset()` first to set reader position to 0.
//!
//! ```rs
//! use mp4iter::Mp4;
//! use std::path::Path;
//!
//! fn main() -> std::io::Result<()> {
//!     let mut mp4 = Mp4::new(Path::new("GOPRO_VIDEO.MP4"))?;
//!
//!     // Iterate over atom headers
//!     for atom_header in mp4.into_iter() {
//!         println!("{atom_header:?}")
//!     }
//!
//!     // Duration for longest track.
//!     println!("{:?}", mp4.duration()?);
//!
//!     // Byte offsets (in 'mdat' atom) for GoPro GPMF telemetry (track name 'GoPro MET')
//!     println!("{:#?}", mp4.offsets("GoPro MET")?);
//!
//!     // List ID and name for all tracks
//!     let track_list = mp4.track_list()?;
//!     for (id, name) in track_list.iter() {
//!         println!("{id} {name}");
//!     }
//!
//!     // Extract information for a track
//!     let track = mp4.track("GoPro H.265")?; // Use track name or numerical ID
//!     println!("{}", track.height);
//!     println!("{}", track.width);
//!     // Iterate over raw track data
//!     for (i, result) in track.samples().enumerate() {
//!         let sample = result?;
//!         println!("{:04} {} bytes", i+1, sample.len())
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::{
    borrow::BorrowMut, fs::File, io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom}, path::{Path, PathBuf}
};

use crate::{
    atom_types::Stsc, reader::AtomReadOrigin, track::{ParsableTrackId, Track, TrackAttributes, TrackIdentifier}, Atom, AtomHeader, AudioFormat, Co64, Dref, Ftyp, Hdlr, Mdhd, Mp4Error, Mp4Reader, Mvhd, ReadOption, SampleOffsets, Sdtp, Smhd, Stco, Stsd, Stss, Stsz, Stts, TargetReader, Tkhd, Tmcd, VideoFormat, Vmhd
};
use binrw::{endian::Endian, BinRead, BinReaderExt};
use time::{ext::NumericalDuration};

/// MP4 reader.
#[derive(Debug)]
pub struct Mp4 {
    /// Path.
    path: PathBuf,
    /// Reader split between a `BufReader` over the full file,
    /// and an in-memory buffer over the `moov` atom.
    pub(crate) reader: Mp4Reader,
}

impl Seek for Mp4 {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.file_reader.seek(pos)
    }
}

impl Read for Mp4 {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.file_reader.read(buf)
    }
}

impl BufRead for Mp4 {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.reader.file_reader.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.reader.file_reader.consume(amt)
    }
}

impl Iterator for Mp4 {
    type Item = AtomHeader;

    /// 'Next' funtion for non-fallible iterator over atom headers.
    /// Returns `None` for errors, i.e. iteration will simply end.
    fn next(&mut self) -> Option<Self::Item> {
        // Read atom header, seek to position of next atom
        // Uses Mp4Reader's BufReader, not cursor over moov
        self.next_header(true).ok()
    }
}

// impl <R: Seek + Read + BinRead + BinReaderExt> Mp4<R> {
impl Mp4 {
    /// New `Mp4` from path.
    ///
    /// `Mp4::new()` uses default buffer size for `BufReader`,
    /// use `Mp4::with_capacity()` for custom buffer sizes.
    pub fn new(path: &Path) -> Result<Self, Mp4Error> {
        let file = File::open(path)?;
        Ok(Self {
            path: path.to_owned(),
            reader: Mp4Reader::new(file)?,
        })
    }

    /// New `Mp4` from path with custom buffer size
    /// for the underlying `BufReader` over the file.
    pub fn with_capacity(
        path: &Path,
        capacity: usize
    ) -> Result<Self, Mp4Error> {
        let file = File::open(path)?;
        Ok(Self {
            path: path.to_owned(),
            reader: Mp4Reader::with_capacity(file, Some(capacity))?,
        })
    }

    /// Mp4 file size in bytes.
    pub fn len(&self) -> u64 {
        self.reader.len(&TargetReader::File)
    }

    /// Mp4 file path.
    pub fn path(&self) -> PathBuf {
        self.path.to_owned()
    }

    /// Returns the underlying reader over the entire file.
    pub fn file_reader(&mut self) -> &mut BufReader<File> {
        &mut self.reader.file_reader
    }

    /// Returns the underlying in-memory reader over the `moov` atom.
    pub fn moov_reader(&mut self) -> &mut Cursor<Vec<u8>> {
        &mut self.reader.moov_reader
    }

    /// Returns current position/byte offset in MP4 file.
    pub(crate) fn pos_file(&mut self) -> Result<u64, Mp4Error> {
        self.reader.pos(&TargetReader::File)
    }

    /// Returns current position/byte offset in MP4 moov atom.
    pub(crate) fn pos_moov(&mut self) -> Result<u64, Mp4Error> {
        self.reader.pos(&TargetReader::Moov)
    }

    pub(crate) fn seek_file(
        &mut self,
        pos: SeekFrom
    ) -> Result<u64, Mp4Error> {
        self.reader.seek(&TargetReader::File, pos)
    }

    pub(crate) fn seek_moov(
        &mut self,
        pos: SeekFrom
    ) -> Result<u64, Mp4Error> {
        self.reader.seek(&TargetReader::Moov, pos)
    }

    /// Seek to start of MP4 file.
    pub fn reset(&mut self) -> Result<(), Mp4Error> {
        self.reader.reset()
    }

    /// Seek to end of MP4 file.
    pub fn end(&mut self) -> Result<(), Mp4Error> {
        self.reader.end()
    }

    /// Read a single type `T`.
    pub fn read_one<T>(
        &mut self,
        endian: Endian,
        pos: Option<SeekFrom>
    ) -> Result<T, Mp4Error>
    where
        T: BinRead,
        <T as BinRead>::Args<'static>: Sized + Clone + Default,
    {
        self.reader.read_one(&TargetReader::File, endian, pos, None)
    }

    /// Read `n` types `T`.
    pub(crate) fn read_many<T>(
        &mut self,
        n: usize,
        endian: Endian,
        pos: Option<SeekFrom>,
    ) -> Result<Vec<T>, Mp4Error>
    where
        T: BinRead,
        <T as BinRead>::Args<'static>: Sized + Clone + Default,
    {
        self.reader.read_many(&TargetReader::File, endian, n, pos, None)
    }

    /// Read a single byte.
    pub(crate) fn read_byte(
        &mut self,
        pos: Option<SeekFrom>
    ) -> Result<u8, Mp4Error> {
        self.reader.read_byte(&TargetReader::File, pos, None)
    }

    /// Read multiple bytes using `ReadOption`
    /// to control reading behaviour:
    /// - `ReadOption::Sized(N)`: read `N` bytes
    /// - `ReadOption::Until(B)`: read until `B` encountered
    /// - `ReadOption::Counted`: read first byte in stream, use as byte count
    /// (i.e. `1 + n_u8` bytes will be read).
    pub(crate) fn read_bytes(
        &mut self,
        option: ReadOption,
        pos: Option<SeekFrom>,
    ) -> Result<Vec<u8>, Mp4Error> {
        self.reader.read_bytes(&TargetReader::File, option, pos, None)
    }

    /// Reads `len` bytes starting at current position if
    /// `pos` is `None` and returns these as `Cursor<Vec<u8>>`.
    pub fn cursor(
        &mut self,
        len: u64,
        pos: Option<SeekFrom>
    ) -> Result<Cursor<Vec<u8>>, Mp4Error> {
        self.reader.cursor(&TargetReader::File, len as usize, pos, None)
    }

    /// `next` method for iterating over atoms.
    pub(crate) fn next_header(
        &mut self,
        seek_next: bool
    ) -> Result<AtomHeader, Mp4Error> {
        let header = self.header()?;

        // Seek to next header offset. Do this for iter impl etc.
        if seek_next {
            self.reader
                .seek(&TargetReader::File, SeekFrom::Current(header.next as i64))?;
        }

        Ok(header)
    }

    /// Return atom header at current offset.
    /// No check is made to verify that current offset
    /// is at the start of an atom (at u32 size specification).
    ///
    /// Results in MP4 position being set to the byte immediately
    /// following the header, adjusted for 64 bit atom size if necessary.
    pub(crate) fn header(&mut self) -> Result<AtomHeader, Mp4Error> {
        self.reader.header(&TargetReader::File, None)
    }

    /// Read atom at current position from target reader,
    /// with reader position at data payload.
    pub(crate) fn atom(
        &mut self,
        target: &TargetReader,
        origin: AtomReadOrigin
    ) -> Result<Atom, Mp4Error> {
        self.reader.atom(target, origin, true)
    }

    /// Finds first top-level atom header with specified FourCC (as string literal).
    ///
    /// If `reset` = `true`, the search will start from the beginning of the MP4.
    pub fn find_header(&mut self, fourcc: &str, reset: bool) -> Result<Option<AtomHeader>, Mp4Error> {
        self.reader.find_header(&TargetReader::File, fourcc, reset)
    }

    /// Returns atom positioned at start of first
    /// encountered atom with specified FourCC.
    ///
    /// Note that some atom types may occur more than once (e.g. `trak` and its child atoms).
    pub fn find_atom(&mut self, fourcc_name: &str, reset: bool) -> Result<Atom, Mp4Error> {
        self.reader.find_atom(&TargetReader::File, fourcc_name, reset)
    }

    /// Returns atom with specified FourCC within `udta` (user data)
    /// container atom.
    pub fn find_user_data(&mut self, fourcc: &str) -> Result<Atom, Mp4Error> {
        // reset to start of mp4,
        // since moov (that contains udta) sometimes precedes `mdat`,
        // then find moov header to set position withing moov bounds
        if self
            .reader
            .find_header(&TargetReader::Moov, "udta", true)?
            .is_some()
        {
            self.reader.find_atom(&TargetReader::Moov, fourcc, false)
        } else {
            Err(Mp4Error::NoSuchAtom(fourcc.to_owned()))
        }
    }

    /// Returns all atom headers for child atoms in
    /// `udta` (user data) atom.
    pub fn user_data_headers(&mut self) -> Result<Vec<AtomHeader>, Mp4Error> {
        let mut headers: Vec<AtomHeader> = Vec::new();
        if let Some(hdr) = self.reader.find_header(&TargetReader::Moov, "udta", true)? {
            while self.reader.pos(&TargetReader::Moov)? < hdr.offset_next_abs() {
                headers.push(self.reader.next_header(&TargetReader::Moov, true)?)
            }
        }
        Ok(headers)
    }

    /// Returns data loads for all user data atoms.
    pub fn user_data_cursors(&mut self) -> Result<Vec<(String, Cursor<Vec<u8>>)>, Mp4Error> {
        self.user_data_headers()?
            .into_iter()
            .map(|h| {
                let pos = SeekFrom::Start(h.data_offset());
                let crs =
                    self.reader
                        .cursor(&TargetReader::Moov, h.data_size() as usize, Some(pos), None)?;
                        // .cursor_at(&TargetReader::Moov, h.data_size() as usize, pos, None)?;
                Ok((h.name.to_string(), crs))
            })
            .collect()
    }

    /// Returns time scale for longest track.
    /// Located in the `mvhd` atom.
    ///
    /// Used to derive e.g. frame rate together with
    /// the unscaled sample duration in `stsd`.
    ///
    /// For a specific track, use `time_scale_track()`
    /// since time scale may differ greatly between tracks.
    pub fn time_scale(&mut self) -> Result<u32, Mp4Error> {
        // reset position to start of file
        Ok(self.mvhd(true)?.time_scale)
    }

    /// Returns time scale for specified track.
    /// Located in the `mdhd` atom in each `trak`.
    pub fn time_scale_track(&mut self, track_name: &str, reset: bool) -> Result<u32, Mp4Error> {
        // reset position to start of file
        Ok(self.mdhd_track_name(track_name, reset)?.time_scale)
    }

    /// Returns video frame rate.
    pub fn frame_rate(&mut self) -> Result<f64, Mp4Error> {
        let mvhd = self.mvhd(true)?; // mvhd precedes stts
        let stts = self.stts_video(false)?;

        // video sample_count * MP4 time_scale / MP4 unscaled_duration
        Ok(stts.len() as f64 * mvhd.time_scale as f64 / mvhd.duration as f64)
    }

    /// Returns video resolution in pixels
    /// as tuple `(WIDTH, HEIGHT)`.
    pub fn resolution(&mut self, reset: bool) -> Result<(u16, u16), Mp4Error> {
        self.stsd_video(reset)? // stsd_video() resets to start of file
            .resolution()
            .ok_or_else(|| Mp4Error::ResolutionExtractionError)
    }

    /// Returns video format.
    pub fn video_format(&mut self, reset: bool) -> Result<VideoFormat, Mp4Error> {
        self.stsd_video(reset)? // stsd_video() resets to start of file
            .video_format()
            .cloned()
            .ok_or_else(|| Mp4Error::VideoFormatExtractionError)
    }

    /// Returns audio format.
    pub fn audio_format(&mut self, reset: bool) -> Result<AudioFormat, Mp4Error> {
        self.stsd_audio(reset)? // stsd_video() resets to start of file
            .audio_format()
            .cloned()
            .ok_or_else(|| Mp4Error::AudioFormatExtractionError)
    }

    /// Returns audio sample rate in Hz.
    pub fn sample_rate(&mut self, reset: bool) -> Result<f64, Mp4Error> {
        self.stsd_audio(reset)?
            .sample_rate()
            .ok_or(Mp4Error::SampleRateExtractionError)
    }

    pub fn major_brand(&mut self, reset: bool) -> Result<String, Mp4Error> {
        Ok(self.ftyp(reset)?.major_brand())
    }

    pub fn compatible_brands(&mut self, reset: bool) -> Result<Vec<String>, Mp4Error> {
        Ok(self.ftyp(reset)?.compatible_brands())
    }

    // -----
    // Atoms
    // -----

    /// Extract movie header atom (`mvhd` atom).
    ///
    /// Path: `moov.mvhd`
    pub fn ftyp(&mut self, reset: bool) -> Result<Ftyp, Mp4Error> {
        self.reader.find_atom(&TargetReader::File, "ftyp", reset)?.ftyp()
    }

    /// Extract movie header atom (`mvhd` atom).
    ///
    /// Path: `moov.mvhd`
    pub fn mvhd(&mut self, reset: bool) -> Result<Mvhd, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "mvhd", reset)?.mvhd()
    }

    /// Extract track header (`tkhd` atom).
    /// One for each track.
    ///
    /// Path: `moov.trak[multiple].tkhd`
    pub fn tkhd(&mut self, reset: bool) -> Result<Tkhd, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "tkhd", reset)?.tkhd()
    }

    /// Extract track header (`tkhd` atom)
    /// for specific handler name (i.e. specific track).
    ///
    /// Path: `moov.trak[multiple].tkhd`
    pub fn tkhd_handler(&mut self, handler_name: &str, reset: bool) -> Result<Tkhd, Mp4Error> {
        if reset {
            self.reset()?;
        }
        loop {
            // Parse tkhd first, since it precedes
            // the hdlr atom containing the handler name.
            let tkhd = self.tkhd(false)?;
            let hdlr = self.hdlr(false)?;
            // Only return tkhd if handler name is correct
            if hdlr.component_name() == handler_name {
                return Ok(tkhd);
            }
        }
    }

    /// Extract media header (`mdhd` atom).
    /// One for each track.
    ///
    /// Path: `moov.trak[multiple].mdhd`
    pub fn mdhd(&mut self, reset: bool) -> Result<Mdhd, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "mdhd", reset)?.mdhd()
    }

    /// Extract media header (`mdhd` atom)
    /// for specific track name (i.e. specific track).
    ///
    /// Path: `moov.trak[multiple].mdhd`
    pub fn mdhd_track_name(&mut self, track_name: &str, reset: bool) -> Result<Mdhd, Mp4Error> {
        if reset {
            self.reset()?;
        }
        loop {
            // Parse tkhd first, since it precedes
            // the hdlr atom containing the handler name.
            let mdhd = self.mdhd(false)?;
            let hdlr = self.hdlr(false)?;
            // Only return tkhd if handler name is correct
            if hdlr.component_name() == track_name {
                return Ok(mdhd);
            }
        }
    }

    /// Extract media header (`mdhd` atom)
    /// for specific track sub type,
    /// (e.g. video = `vide``, audio = `soun`, time code = `tmcd`).
    ///
    /// Path: `moov.trak[multiple].mdhd`
    pub fn mdhd_track_subtype(&mut self, sub_type: &str, reset: bool) -> Result<Mdhd, Mp4Error> {
        if reset {
            self.reset()?;
        }
        loop {
            // Parse tkhd first, since it precedes
            // the hdlr atom containing the handler name.
            let mdhd = self.mdhd(false)?;
            let hdlr = self.hdlr(false)?;
            // Only return tkhd if handler name is correct
            if hdlr.component_sub_type() == sub_type {
                return Ok(mdhd);
            }
        }
    }

    /// Extract media handler values (`hdlr` atom).
    ///
    /// Path: `moov.trak[multiple].mdia.hdlr`
    pub fn hdlr(&mut self, reset: bool) -> Result<Hdlr, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "hdlr", reset)?.hdlr()
    }

    /// Extract sync sample atom for first encountered
    /// `stss` atom at current position (optionally one `stss` for each `trak`).
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stss`
    pub fn stss(&mut self, reset: bool) -> Result<Stss, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "stss", reset)?.stss()
    }

    /// Extract sample dependency flags atom for first encountered
    /// `sdtp` atom at current position (optionally one `sdtp` for each `trak`).
    ///
    /// Note that number of entries should be derived from `stsz` atoms entry number.
    /// However, since `sdtp` precedes `stsz`, atom size is used to derive this value instead.
    /// If necessary, verify with the associated `stsz` (the one in the same track/`trak`)
    /// that follows the `sdtp` atom.
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.sdtp`
    pub fn sdtp(&mut self, reset: bool) -> Result<Sdtp, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "sdtp", reset)?.sdtp()
    }

    /// Extract time to sample values for first encountered
    /// `stts` atom at current position (one `stts` for each `trak`).
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stts`
    pub fn stts(&mut self, reset: bool) -> Result<Stts, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "stts", reset)?.stts()
    }

    /// Returns the first encountered `stts` atom that describes video data.
    pub fn stts_video(&mut self, reset: bool) -> Result<Stts, Mp4Error> {
        if reset {
            self.reset()?;
        }
        loop {
            let stsd = self.stsd(false)?;
            if stsd.is_video() {
                return self.stts(false);
            }
        }
    }

    /// Extract sample to size values (`stsz` atom - one for each `trak`).
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stsz`
    pub fn stsz(&mut self, reset: bool) -> Result<Stsz, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "stsz", reset)?.stsz()
    }

    /// Returns chunk offset values for files below 32bit limit
    /// (`stco` atom - one for each `trak`).
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stco`
    pub fn stco(&mut self, reset: bool) -> Result<Stco, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "stco", reset)?.stco()
    }

    /// Extract sample to chunk values
    /// (`stsc` atom - one for each `trak`).
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stco`
    pub fn stsc(&mut self, reset: bool) -> Result<Stsc, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "stsc", reset)?.stsc()
    }

    /// Extract chunk offset values for files above 32bit limit
    /// (`co64` atom - one for each `trak`).
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stco`
    pub fn co64(&mut self, reset: bool) -> Result<Co64, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "co64", reset)?.co64()
    }

    /// Extract `dref` atom.
    ///
    /// Path: `moov.trak[multiple].mdia.minf.dinf.dref`
    pub fn dref(&mut self, reset: bool) -> Result<Dref, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "dref", reset)?.dref()
    }

    /// Extract sound media information header atom (`smhd`).
    ///
    /// Path: `moov.trak[multiple].mdia.minf.smhd`
    pub fn smhd(&mut self, reset: bool) -> Result<Smhd, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "smhd", reset)?.smhd()
    }

    /// Video media information header atom (`vmhd`)
    ///
    /// Path: `moov.trak[multiple].mdia.minf.vmhd`
    pub fn vmhd(&mut self, reset: bool) -> Result<Vmhd, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "vmhd", reset)?.vmhd()
    }

    /// Extract sample description atom (`stsd` atom).
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stsd`
    pub fn stsd(&mut self, reset: bool) -> Result<Stsd, Mp4Error> {
        self.reader.find_atom(&TargetReader::Moov, "stsd", reset)?.stsd()
    }

    /// Returns the first encountered sample description atom
    /// (`stsd` atom) for specified track.
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stsd`
    pub fn stsd_track(&mut self, track_name: &str) -> Result<Stsd, Mp4Error> {
        loop {
            let hdlr = self.hdlr(false)?;
            if hdlr.component_name() == track_name {
                return self.stsd(false);
            }
        }
    }

    /// Returns the first encountered `stsd` atom describing video,
    /// either searching from current position or the start of the file
    /// (`reset = true`).
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stsd`
    pub fn stsd_video(&mut self, reset: bool) -> Result<Stsd, Mp4Error> {
        if reset {
            self.reset()?;
        }
        loop {
            let stsd = self.stsd(false)?;
            if stsd.is_video() {
                return Ok(stsd);
            }
        }
    }

    /// Returns the first encountered `stsd` atom describing audio,
    /// either searching from current position or the start of the file
    /// (`reset = true`).
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stsd`
    pub fn stsd_audio(&mut self, reset: bool) -> Result<Stsd, Mp4Error> {
        if reset {
            self.reset()?;
        }
        loop {
            let stsd = self.stsd(false)?;
            if stsd.is_audio() {
                return Ok(stsd);
            }
        }
    }

    /// Returns the first encountered `stsd` atom describing binary data,
    /// searching from current position.
    ///
    /// Path: `moov.trak[multiple].mdia.minf.stbl.stsd`
    pub fn stsd_binary(&mut self) -> Result<Stsd, Mp4Error> {
        self.reset()?;
        loop {
            let stsd = self.stsd(false)?;
            if stsd.is_binary() {
                return Ok(stsd);
            }
        }
    }

    /// Returns timecode data to derive start time of video.
    ///
    /// Note that `tmcd` is not in the main MP4 atom tree,
    /// but contained within `stsd` and therefore can not
    /// be located via `Mp4` find methods.
    ///
    /// This atom is optional and its presence depends on
    /// what device or software produced the MP4. `track_name`
    /// is the `component_name` field in the `hdlr` atom and is the same string
    /// FFmpeg reports as `handler_name`.
    ///
    /// For GoPro, use `GoPro TCD` as handler name.
    pub fn tmcd_track_name(&mut self, track_name: &str, reset: bool) -> Result<Tmcd, Mp4Error> {
        let mdhd = self.mdhd_track_name(track_name, reset)?;
        // Loop until 'hdlr' atom with correct track name encountered.
        let mut tmcd = self.stsd(false)?.tmcd()?;
        // Find the following stts, stsc, stco atoms to generate offsets.
        // Note that time scale will be set to 1 if its actual value is 0 (invalid)
        tmcd.offsets = SampleOffsets::new(self, mdhd.time_scale, true, None)?;
        Ok(tmcd)
    }

    /// Returns timecode data to derive start time of video.
    ///
    /// Note that `tmcd` is not in the main MP4 atom tree,
    /// but contained within `stsd` and therefore can not
    /// be located via `Mp4` find methods.
    ///
    /// This atom is optional and its presence depends on
    /// what device or software produced the MP4.
    pub fn tmcd2(&mut self, reset: bool) -> Result<Tmcd, Mp4Error> {
        let mdhd = self.mdhd_track_subtype("tmcd", reset)?;
        // Loop until 'hdlr' atom with correct track name encountered.
        let mut tmcd = self.stsd(false)?.tmcd()?;
        // Find the following stts, stsc, stco atoms to generate offsets.
        // Note that time scale will be set to 1 if its actual value is 0 (invalid)
        tmcd.offsets = SampleOffsets::new(self, mdhd.time_scale, true, None)?;
        Ok(tmcd)
    }

    /// Extract user data atom (`udta`).
    /// Some vendors embed data such as device info,
    /// unique identifiers (Garmin VIRB UUID),
    /// or even data in vendor specific formats
    /// (GoPro undocumented GPMF data, separate from
    /// the main GPMF telemetry interleaved in the `mdat` atom).
    ///
    /// Path: `moov.udta`
    pub fn udta(&mut self, reset: bool) -> Result<Atom, Mp4Error> {
        // Set reset to true, position to start of file to avoid
        // previous reads to have moved the cursor
        // past the 'udta' atom.
        self.reader.find_atom(&TargetReader::Moov, "udta", reset)
    }

    // -----
    // Track
    // -----

    // /// Returns the `Track` with specified track name (`hdlr.component_name`).
    // ///
    // /// Contains timestamps and offsets for byte load in `mdat` atom.
    // pub fn track(&mut self, track_name: &str, reset: bool) -> Result<Track, Mp4Error> {
    //     Track::from_name(self.borrow_mut(), track_name, reset)
    // }

    /// Returns the track with specified identifier, e.g.:
    /// - `TrackIdentifier::Name("GoPro MET")` for GoPro timed metadata track.
    /// - `TrackIdentifier::Id(2)` for the track with internal numerical ID 2 (not always continuous, e.g. 1, 2, 3, 5 is possible).
    /// - `TrackIdentifier::SubType("vide")` for the video track.
    ///
    /// Some kinds of tracks may be present more than once, such audio if multi-languge,
    /// or two video tracks for 360 cameras.
    // pub fn track(&mut self, identifier: TrackIdentifier, reset: bool) -> Result<Track, Mp4Error> {
    pub fn track(&mut self, identifier: impl ParsableTrackId, reset: bool) -> Result<Track, Mp4Error> {
        Track::new(self.borrow_mut(), identifier, reset)
    }

    /// Returns general info for all track in the MP4 file.
    pub fn track_list(&mut self, reset: bool) -> Result<Vec<TrackAttributes>, Mp4Error> {
        TrackAttributes::all(self.borrow_mut(), reset)
    }

    /// Returns creation time of MP4.
    ///
    /// Derived from `mvhd` atom (inside `moov` atom).
    ///
    /// Note that some recording devices that split into clips,
    /// such as GoPro, may have the same start time for all clips
    /// from the same session. This depends on the exact model.
    /// For these, find the `trak` with the title `GoPro TCD` instead,
    /// and use the timecode data in there (`tmcd` entry in an `stsd` atom).
    /// The convenience method `Mp4::time_first_frame()` returns this value.
    ///
    /// Reference `mvhd`: <https://developer.apple.com/documentation/quicktime-file-format/movie_header_atom>
    pub fn creation_time(&mut self, reset: bool) -> Result<time::PrimitiveDateTime, Mp4Error> {
        let mvhd = self.mvhd(reset)?;
        Ok(mvhd.creation_time())
    }

    /// Returns duration for longest track.
    ///
    /// Derived from `mvhd` atom (inside `moov` atom),
    /// which lists duration for whichever track is the longest.
    ///
    /// For individual tracks, the `mdhd` atom must be used.
    ///
    /// Reference `mvhd`: <https://developer.apple.com/documentation/quicktime-file-format/movie_header_atom>
    pub fn duration(&mut self, reset: bool) -> Result<time::Duration, Mp4Error> {
        let mvhd = self.mvhd(reset)?;
        Ok(mvhd.duration())
    }

    pub fn duration_track(&mut self, track_name: &str, reset: bool) -> Result<time::Duration, Mp4Error> {
        let mdhd = self.mdhd_track_name(track_name, reset)?;
        Ok(mdhd.duration())
    }

    /// Returns creation time as datetime and duration in seconds
    /// as the tuple `(CREATION_TIME, DURATION)`.
    ///
    /// Derived from `mvhd` atom,
    /// which lists duration for whichever track is the longest.
    ///
    /// Start time may default to MP4 default time
    /// `1904-01-01 00:00:00` depending on device and clock settings.
    ///
    /// Note that some recording devices that split into clips,
    /// such as GoPro, may have the same start time for all clips
    /// from the same session. This depends on the exact model.
    /// For these, find the `trak` with the title `GoPro TCD` instead,
    /// and use the timecode for the first frame in there
    /// (`tmcd` entry in an `stsd` atom).
    pub fn time(
        &mut self,
        reset: bool,
    ) -> Result<(time::PrimitiveDateTime, time::Duration), Mp4Error> {
        let mvhd = self.mvhd(reset)?;
        Ok((mvhd.creation_time(), mvhd.duration()))
    }

    pub fn time_first_frame(&mut self, reset: bool) -> Result<time::Duration, Mp4Error> {
        let mut track = self.track(TrackIdentifier::SubType("tmcd"), reset)?;

        let number_of_frames = track.tmcd()?.number_of_frames;

        // Get first sample, but return error if not found.
        let result = track.samples()
            .nth(0)
            .ok_or_else(|| Mp4Error::NoOffsets("track sub-type 'tmcd'".to_string()))?;

        let unscaled_time: u32 = result?.read_be()?;
        let duration = (unscaled_time as f64 / number_of_frames as f64).seconds();

        Ok(duration)
    }

    /// Returns all headers for "main" tree atoms,
    /// i.e. atoms that follow
    /// `<32-bit SIZE><FourCC header><Optional 64-bit size><Data load>`
    /// in the MP4 atom tree.
    pub fn headers(&mut self) -> Result<Vec<AtomHeader>, Mp4Error> {
        let mut hdrs: Vec<AtomHeader> = Vec::new();
        loop {
            match self.next_header(true) {
                Ok(hdr) => {
                    if self.pos_file()? < self.len() {
                        hdrs.push(hdr)
                    } else {
                        break;
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(hdrs)
    }

    pub fn eof(&mut self) -> Result<bool, Mp4Error>{
        Ok(self.pos_file()? >= self.len())
    }
}
