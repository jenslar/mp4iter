use std::{
    borrow::BorrowMut,
    fs::File,
    io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom}, ops::Range,
};

use binrw::{BinRead, BinReaderExt, Endian};

use crate::{Atom, AtomHeader, FourCC, Mp4Error, CONTAINER};

/// `BufReader` over a `File`,
/// with read boundaries,
/// for e.g. atoms.
#[derive(Debug)]
pub(crate) struct Mp4Reader {
    /// File size.
    pub(crate) len: u64,
    /// Reader over the full MP4 file.
    pub(crate) file_reader: BufReader<File>,
    /// Atom header representing the
    /// `moov` atom (offsets correspond to the full file).
    pub(crate) moov_header: AtomHeader,
    /// In-memory buffer/reader over the `moov` atom
    /// data load (i.e. atoms contained by `moov`).
    pub(crate) moov_reader: Cursor<Vec<u8>>,
}

impl Mp4Reader {
    /// Creates a `BufReader` with default capacity (8KiB)
    /// for the full MP4 file,
    /// and in-memory buffer (`Cursor<Vec<u8>>`) over the `moov`
    /// atom in that file.
    ///
    /// Use `Mp4Reader::with_capacity()` instead to use
    /// custom buffer sizes. (e.g. GoPro often stores
    /// telemetry with chunk sizes just above the default
    /// 8KiB buffer size)
    pub(crate) fn new(file: File) -> Result<Self, Mp4Error> {
        Self::with_capacity(file, None)
    }

    /// Returns which reader to use for reading at `file_pos`
    /// (in-memory buffer:ed `moov` atom should always be faster when possible).
    pub(crate) fn select_reader(&mut self, file_pos: u64) -> TargetReader {
    // pub(crate) fn select_reader(&mut self, origin: AtomReadOrigin) -> Result<TargetReader, Mp4Error> {
        // let bounds = pos .. pos + len as u64;
        // match self.moov_header.in_bounds(&bounds) {
        match self.moov_header.bounds().contains(&file_pos) {
            true => TargetReader::Moov,
            false => TargetReader::File,
        }
        // let abs_pos: Option<u64> = match origin {
        //     AtomReadOrigin::Position(pos) => match pos {
        //         SeekFrom::Start(abs) => Some(abs),
        //         SeekFrom::End(rel) => Some(self.len - rel.abs() as u64), // rel must be negative to be valid
        //         SeekFrom::Current(rel) => match rel > 0 {
        //             true => Some(self.pos(&TargetReader::File)? + rel as u64),
        //             false => Some(self.pos(&TargetReader::File)? - rel.abs() as u64),
        //         },
        //     },
        //     AtomReadOrigin::Header(hdr) => Some(hdr.offset),
        //     AtomReadOrigin::None => None,
        // };
        // Ok(TargetReader::File)
    }

    pub(crate) fn with_capacity(
        file: File,
        capacity: Option<usize>
    ) -> Result<Self, Mp4Error> {
        let len = file.metadata()?.len();
        let reader = match capacity {
            Some(cap) => BufReader::with_capacity(cap, file),
            None => BufReader::new(file),
        };

        let mut rdr = Self {
            file_reader: reader,
            len,
            moov_header: AtomHeader::default(),
            moov_reader: Cursor::new(Vec::new()),
        };

        if let Some(moov_hdr) = rdr.find_header(&TargetReader::File, "moov", false)? {
            // no need to seek, already at moov data load position after having read header
            let moov_crs = rdr.cursor(
                &TargetReader::File,
                usize::try_from(moov_hdr.data_size())?,
                None,
                None
            )?;
            rdr.moov_header = moov_hdr;
            rdr.moov_reader = moov_crs;

            // reset bufreader to start of file after reading moov
            rdr.reset_file()?;

            return Ok(rdr);
        };

        Err(Mp4Error::MoovReadError)
    }

    /// Seeks to position `pos` for target stream.
    ///
    /// Assumes `pos` is within target reader bounds.
    pub(crate) fn seek(
        &mut self,
        target: &TargetReader,
        pos: SeekFrom
    ) -> Result<u64, Mp4Error> {
        match target {
            TargetReader::File => Ok(self.file_reader.seek(pos)?),
            TargetReader::Moov => Ok(self.moov_reader.seek(pos)?),
        }
    }

    /// Seeks both streams to absolute file position `pos`,
    /// or the equivalent thereof (for `moov`).
    ///
    /// Raises error if file reader is not within `moov` boundaries.
    /// Panics if resulting offsets are not equal.
    pub(crate) fn sync_seek(&mut self, seek_to_abs: u64) -> Result<u64, Mp4Error> {
        let fpos = self.file_reader.seek(SeekFrom::Start(seek_to_abs))?;
        self.bounds_moov()?;
        let mpos = self
            .moov_reader
            .seek(SeekFrom::Start(seek_to_abs - self.moov_header.offset))?;
        assert_eq!(fpos, mpos + self.moov_header.offset, "MP4 readers's position could not be synchronised");
        Ok(fpos)
    }

    /// Returns the size of `n` number of sized type `T`.
    pub(crate) fn type_size<T: Sized>(&self, n: usize) -> usize {
        n * std::mem::size_of::<T>()
    }

    fn read_type<T>(
        &mut self,
        target: &TargetReader,
        endian: Endian
    ) -> Result<T, Mp4Error>
    where
        T: BinRead,
        <T as BinRead>::Args<'static>: Sized + Clone + Default,
    {
        match target {
            TargetReader::File => Ok(self.file_reader.read_type::<T>(endian)?),
            TargetReader::Moov => Ok(self.moov_reader.read_type::<T>(endian)?),
        }
    }

    /// Read native endian type `T`.
    pub(crate) fn read_ne<T>(
        &mut self,
        target: &TargetReader
    ) -> Result<T, Mp4Error>
    where
        T: BinRead,
        <T as BinRead>::Args<'static>: Sized + Clone + Default,
    {
        match target {
            TargetReader::File => Ok(self.file_reader.read_ne::<T>()?),
            TargetReader::Moov => Ok(self.moov_reader.read_ne::<T>()?),
        }
    }

    /// Read little endian type `T`.
    pub(crate) fn read_le<T>(
        &mut self,
        target: &TargetReader
    ) -> Result<T, Mp4Error>
    where
        T: BinRead,
        <T as BinRead>::Args<'static>: Sized + Clone + Default,
    {
        match target {
            TargetReader::File => Ok(self.file_reader.read_le::<T>()?),
            TargetReader::Moov => Ok(self.moov_reader.read_le::<T>()?),
        }
    }

    /// Read big endian type `T`.
    pub(crate) fn read_be<T>(
        &mut self,
        target: &TargetReader
    ) -> Result<T, Mp4Error>
    where
        T: BinRead,
        <T as BinRead>::Args<'static>: Sized + Clone + Default,
    {
        match target {
            TargetReader::File => Ok(self.file_reader.read_be::<T>()?),
            TargetReader::Moov => Ok(self.moov_reader.read_be::<T>()?),
        }
    }

    pub(crate) fn read_one<T>(
        &mut self,
        target: &TargetReader,
        endian: Endian,
        pos: Option<SeekFrom>,
        bounds: Option<(u64, u64)>,
    ) -> Result<T, Mp4Error>
    where
        T: BinRead,
        <T as BinRead>::Args<'static>: Sized + Clone + Default,
    {
        if let Some(p) = pos {
            self.seek(target, p)?;
        }
        let val = self.read_type::<T>(target, endian)?;
        self.bounds(target, bounds)?;
        Ok(val)
    }

    /// Read `n` number of type `T` with specified `endian`.
    ///
    /// Optionally specify bounds for e.g. atoms.
    pub(crate) fn read_many<T>(
        &mut self,
        target: &TargetReader,
        endian: Endian,
        n: usize,
        pos: Option<SeekFrom>,
        bounds: Option<(u64, u64)>,
    ) -> Result<Vec<T>, Mp4Error>
    where
        T: BinRead,
        <T as BinRead>::Args<'static>: Sized + Clone + Default,
    {
        if let Some(p) = pos {
            self.seek(target, p)?;
        }
        let val = (0..n)
            .into_iter()
            .map(|_| self.read_type::<T>(target, endian))
            .collect::<Result<Vec<T>, Mp4Error>>()?;
        self.bounds(target, bounds)?;
        Ok(val)
    }

    /// Read a single byte.
    pub(crate) fn read_byte(
        &mut self,
        target: &TargetReader,
        pos: Option<SeekFrom>,
        bounds: Option<(u64, u64)>,
    ) -> Result<u8, Mp4Error> {
        let val = self.read_one::<u8>(target, Endian::Big, pos, bounds)?;
        self.bounds(target, bounds)?;
        Ok(val)
    }

    /// Read multiple bytes at current position in target stream
    /// using `ReadOption` to control reading behaviour:
    /// - `ReadOption::Sized(N)`: read `N` bytes
    /// - `ReadOption::Until(B)`: read until sentinel `B` encountered
    /// - `ReadOption::Counted`: read first byte in stream, use as byte count
    /// (i.e. `1 + n_u8` bytes will be read).
    pub(crate) fn read_bytes(
        &mut self,
        target: &TargetReader,
        option: ReadOption,
        pos: Option<SeekFrom>,
        bounds: Option<(u64, u64)>,
    ) -> Result<Vec<u8>, Mp4Error> {
        if let Some(p) = pos {
            self.seek(target, p)?;
        }
        let buf = match option {
            ReadOption::Sized(n) => {
                let mut b = vec![0_u8; n];
                match &target {
                    TargetReader::File => self.file_reader.read_exact(&mut b)?,
                    TargetReader::Moov => self.moov_reader.read_exact(&mut b)?,
                };
                b
            }
            ReadOption::Until(s) => {
                let mut b = vec![];
                match &target {
                    TargetReader::File => self.file_reader.read_until(s, &mut b)?,
                    TargetReader::Moov => self.moov_reader.read_until(s, &mut b)?,
                };
                b
            }
            ReadOption::Counted => {
                let mut b = vec![0_u8; self.read_byte(target, None, None)? as usize];
                match &target {
                    TargetReader::File => self.file_reader.read_exact(&mut b)?,
                    TargetReader::Moov => self.moov_reader.read_exact(&mut b)?,
                };
                b
            }
        };
        self.bounds(target, bounds)?;

        Ok(buf)
    }

    /// Returns size in bytes for target stream.
    pub(crate) fn len(&self, target: &TargetReader) -> u64 {
        match target {
            TargetReader::File => self.len,
            TargetReader::Moov => self.moov_reader.get_ref().len() as u64,
        }
    }

    /// Returns current position for specified reader.
    pub(crate) fn pos(&mut self, target: &TargetReader) -> Result<u64, Mp4Error> {
        match target {
            TargetReader::File => Ok(self.file_reader.stream_position()?),
            TargetReader::Moov => Ok(self.moov_reader.stream_position()?),
        }
    }

    /// Checks if file stream position is within
    /// `moov` atom bounds, and returns file stream position if true.
    fn in_moov_range(&mut self) -> Result<u64, Mp4Error> {
        let pos = self.file_reader.stream_position()?;
        match pos >= self.moov_header.offset {
            true => Ok(pos),
            false => Err(Mp4Error::BoundsError(
                pos,
                self.moov_header.start(),
                self.moov_header.end(),
            )),
        }
    }

    /// Syncs both readers to the same position.
    /// `target` indicates the stream whose current position,
    /// should be the resulting new position for both streams.
    ///
    /// Sync:ing positions is only possible if target position
    /// exists for both streams. I.e. it is only possible to sync
    /// within `moov` boundaries.
    pub(crate) fn sync_pos(&mut self, target: &TargetReader) -> Result<u64, Mp4Error> {
        match target {
            TargetReader::File => {
                let new_pos = self.in_moov_range()?; // can only sync pos within moov boundaries
                Ok(self
                    .moov_reader
                    .seek(SeekFrom::Start(new_pos - self.moov_header.offset))?)
            }
            TargetReader::Moov => {
                let new_pos = self.moov_reader.stream_position()?;
                Ok(self
                    .file_reader
                    .seek(SeekFrom::Start(new_pos + self.moov_header.offset))?)
            }
        }
    }

    /// Returns remaining number of bytes for target stream,
    /// or until upper bound.
    pub(crate) fn rem(
        &mut self,
        target: &TargetReader,
        bounds: Option<(u64, u64)>,
    ) -> Result<u64, Mp4Error> {
        match bounds {
            Some((_, end)) => Ok(end - self.pos(target)?),
            None => Ok(self.len(target) - self.pos(target)?),
        }
    }

    /// Resets postion to start of both file and moov readers.
    pub(crate) fn reset(&mut self) -> Result<(), Mp4Error> {
        self.reset_file()?;
        self.reset_moov()?;
        Ok(())
    }

    #[inline]
    pub(crate) fn reset_moov(&mut self) -> Result<u64, Mp4Error> {
        Ok(self.moov_reader.seek(SeekFrom::Start(0))?)
    }

    #[inline]
    pub(crate) fn reset_file(&mut self) -> Result<u64, Mp4Error> {
        Ok(self.file_reader.seek(SeekFrom::Start(0))?)
    }

    /// Reads bytes as if they were single-byte graphemes
    /// part of a ISO8859-1 string.
    /// (for e.g. reading FourCC that exceed ASCII range,
    /// since decoding these as UTF-8 will fail).
    ///
    /// Note that the returned `String` is a standard UTF-8
    /// encoded string.
    pub(crate) fn read_iso8859_1(
        &mut self,
        target: &TargetReader,
        option: ReadOption,
        pos: Option<SeekFrom>,
        bounds: Option<(u64, u64)>,
    ) -> Result<String, Mp4Error> {
        let buf = self.read_bytes(target, option, pos, bounds)?;
        Ok(buf.iter().map(|b| *b as char).collect())
    }

    /// Reads UTF-8 string.
    ///
    /// Note that this will fail on some FourCC names,
    /// with single-byte characters that exceed ASCII.
    /// To read FourCC, instead use `read_iso8859_1()`.
    pub(crate) fn read_string(
        &mut self,
        target: &TargetReader,
        option: ReadOption,
        pos: Option<SeekFrom>,
        bounds: Option<(u64, u64)>,
    ) -> Result<String, Mp4Error> {
        let buf = self.read_bytes(target, option, pos, bounds)?;
        Ok(String::from_utf8(buf)?)
    }

    /// Reads `len` bytes
    /// into `Cursor<Vec<u8>>`.
    ///
    /// Note that the `mdat` atom may be many GB in size.
    pub(crate) fn cursor(
        &mut self,
        target: &TargetReader,
        len: usize,
        pos: Option<SeekFrom>,
        bounds: Option<(u64, u64)>,
    ) -> Result<Cursor<Vec<u8>>, Mp4Error> {
        Ok(Cursor::new(self.read_bytes(
            target,
            ReadOption::Sized(len),
            pos,
            bounds,
        )?))
    }

    /// Reads FourCC at current position.
    pub(crate) fn fourcc(
        &mut self,
        target: &TargetReader
    ) -> Result<FourCC, Mp4Error> {
        Ok(FourCC::from_u32(
            self.read_type::<u32>(target, Endian::Big)?,
        ))
    }

    /// Returns MP4 header at current position.
    ///
    /// Does not verify that current position
    /// is at atom boundary.
    pub(crate) fn header(
        &mut self,
        target: &TargetReader,
        pos: Option<SeekFrom>,
    ) -> Result<AtomHeader, Mp4Error> {
        if let Some(p) = pos {
            self.seek(target, p)?;
        }

        let mut hdr = AtomHeader::default();

        // Get offset for header
        hdr.offset = self.pos(target)?;

        // Read 32bit total atom size
        hdr.atom_size = self.read_type::<u32>(target, Endian::Big)? as u64;

        // Can not read fourcc name as utf-8 since some
        // manufacturers use single-byte extended ascii/ISO8859-1
        hdr.name = self.fourcc(target)?;

        // Check if atom size is 64bit and read the 8 bytes
        // following directly after FourCC as new size if so
        if hdr.atom_size == 1 {
            hdr.atom_size = self.read_type::<u64>(target, Endian::Big)?;
            // some cameras exclusively use 64bit size, even for sub-64bit sized atoms
            // so need a flag to store this for correctly deriving offset to next atom
            hdr.size_64bit = true;
        }

        if hdr.atom_size == 0 {
            return Err(Mp4Error::ZeroSizeAtom {
                name: hdr.name.to_string(),
                offset: hdr.offset,
            });
        }

        // should this be .next() method instead?
        hdr.next = match CONTAINER.contains(&hdr.name.to_str()) {
            true => 0,
            false => hdr.atom_size - hdr.header_size() as u64,
        };

        Ok(hdr)
    }

    /// Return a reader over the atom at current offset.
    /// Assumes offset is at atom data payload position,
    /// i.e directly after the header, adjusted for 64-bit
    /// sizes.
    pub(crate) fn atom(
        &mut self,
        target: &TargetReader,
        origin: AtomReadOrigin,
        seek_to_data: bool,
    ) -> Result<Atom, Mp4Error> {
        let header = match origin {
            AtomReadOrigin::Position(pos) => self.header(target, Some(pos)),
            AtomReadOrigin::Header(hdr) => Ok(hdr),
            AtomReadOrigin::None => self.header(target, None),
        }?;
        Atom::new(&header, self.borrow_mut(), target, seek_to_data)
    }

    /// Returns atom positioned at start of first
    /// encountered atom with specified FourCC.
    ///
    /// Note that some atom types may occur more than once (e.g. `trak` and its child atoms).
    pub fn find_atom(
        &mut self,
        target: &TargetReader,
        fourcc: &str,
        reset: bool,
    ) -> Result<Atom, Mp4Error> {
        if let Some(header) = self.find_header(target, fourcc, reset)? {
            self.atom(target, AtomReadOrigin::Header(header), false) // cursor is already at data payload position
        } else {
            Err(Mp4Error::NoSuchAtom(fourcc.to_owned()))
        }
    }

    /// Returns atom positioned at start of first
    /// encountered atom with specified FourCC.
    ///
    /// Note that some atom types may occur more than once (e.g. `trak` and its child atoms).
    pub fn find_atom2(
        &mut self,
        target: &TargetReader,
        fourcc: &str,
        fourcc_sentinel: Option<&str>,
        reset: bool,
    ) -> Result<Atom, Mp4Error> {
        if let Some(header) = self.find_header2(target, fourcc, fourcc_sentinel, reset)? {
            // cursor is at data payload position
            self.atom(target, AtomReadOrigin::Header(header), false)
        } else {
            Err(Mp4Error::NoSuchAtom(fourcc.to_owned()))
        }
    }

    /// `next` method for iterating over atoms.
    pub(crate) fn next_header(
        &mut self,
        target: &TargetReader,
        seek_next: bool,
    ) -> Result<AtomHeader, Mp4Error> {
        let header = self.header(target, None)?;

        // Seek to next header offset. Do this for iter impl etc.
        if seek_next {
            match target {
                TargetReader::File => self.file_reader.seek(SeekFrom::Current(header.next as i64))?, // casting u64 as i64...
                TargetReader::Moov => self.moov_reader.seek(SeekFrom::Current(header.next as i64))?,
            };
        }

        Ok(header)
    }

    /// Finds first main tree atom header with specified name (FourCC as string literal
    /// , e.g. "udta" for custom user data atom).
    ///
    /// If found, the header is returned with reader position at atom payload.
    ///
    /// If `reset` is set, the search will start from the beginning of the MP4.
    pub fn find_header(
        &mut self,
        target: &TargetReader,
        fourcc: &str,
        reset: bool,
    ) -> Result<Option<AtomHeader>, Mp4Error> {
        if reset {
            self.reset()?;
        }

        let fourcc = FourCC::from_str(fourcc);

        while self.pos(target)? < self.len(target) {
            // with seek=false, just reads at current pos,
            // and does not seek next atom header,
            // done manually to be able to return early
            let header = self.next_header(target, false)?;

            if header.name == fourcc {
                return Ok(Some(header));
            }

            // ... and if not the correct atom, seek to next one manually
            match target {
                TargetReader::File => self.file_reader.seek(SeekFrom::Current(header.next as i64))?,
                TargetReader::Moov => self.moov_reader.seek(SeekFrom::Current(header.next as i64))?,
            };
        }
        Ok(None)
    }

    /// Finds first main tree atom header with specified name (FourCC as string literal
    /// , e.g. "udta" for custom user data atom).
    ///
    /// If found, the header is returned with reader position at atom payload.
    ///
    /// If `reset` is set, the search will start from the beginning of the MP4.
    ///
    /// Returns `Ok(None)` if atom with FourCC `sentinel` is encountered. E.g.
    /// if searching for `stco`, and encounters next `trak` (avoiding
    /// the `stco` atom for the next track to be returned)
    pub fn find_header2(
        &mut self,
        target: &TargetReader,
        fourcc: &str,
        fourcc_sentinel: Option<&str>,
        reset: bool,
    ) -> Result<Option<AtomHeader>, Mp4Error> {
        if reset {
            self.reset()?;
        }

        let fourcc = FourCC::from_str(fourcc);

        while self.pos(target)? < self.len(target) {
            // with seek=false, just reads at current pos,
            // and does not seek next atom header,
            // done manually to be able to return early
            let header = self.next_header(target, false)?;

            if let Some(s) = fourcc_sentinel {
                let sentinel = FourCC::from_str(s);
                if header.name == sentinel {
                    return Ok(None)
                }
            }

            if header.name == fourcc {
                return Ok(Some(header));
            }

            // ... and if not the correct atom, seek to next one manually
            match target {
                TargetReader::File => self.file_reader.seek(SeekFrom::Current(header.next as i64))?,
                TargetReader::Moov => self.moov_reader.seek(SeekFrom::Current(header.next as i64))?,
            };
        }
        Ok(None)
    }

    /// Returns absolute position for next atom.
    /// Assumes current offset is at the start of an atom.
    ///
    /// Changes position to after atom header (8 or 16 bytes).
    pub(crate) fn next_abs(
        &mut self,
        target: &TargetReader,
        header: Option<&AtomHeader>,
    ) -> Result<u64, Mp4Error> {
        match header {
            Some(hdr) => Ok(hdr.offset_next_abs()),
            None => Ok(self.header(target, None)?.offset_next_abs()),
        }
    }

    /// Returns relative position for next atom.
    /// Assumes current offset is at the start of an atom.
    ///
    /// Changes position to after atom header (8 or 16 bytes).
    pub(crate) fn next_rel(
        &mut self,
        target: &TargetReader,
        header: Option<&AtomHeader>,
    ) -> Result<u64, Mp4Error> {
        match header {
            Some(hdr) => Ok(hdr.offset_next_rel()),
            None => Ok(self.header(target, None)?.offset_next_rel()),
        }
    }

    /// Seek to start of next atom.
    ///
    /// Assumes current offset is at the start of an atom.
    pub(crate) fn seek_next(
        &mut self,
        target: &TargetReader,
        header: Option<&AtomHeader>,
    ) -> Result<u64, Mp4Error> {
        let new_pos = self.next_abs(target, header)?;
        // self.seek_abs(new_pos)
        Ok(self.seek(target, SeekFrom::Start(new_pos))?)
    }

    /// Returns the header for the atom that `pos`
    /// is within bounds of. If `pos` is `None`,
    /// current reader position is used instead.
    ///
    /// Optionally ignore containers (otherwise
    /// sub-atoms will be ignored).
    ///
    /// Inclusive lower bound, exclusive upper bound
    /// when checking against atom start/end offset.
    /// I.e `start_of_atom <= pos < end_of_atom`,
    /// meaning that if the upper bound is at an atom boundary,
    /// the position for the atom that starts at that offset
    /// will be returned.
    pub(crate) fn header_closest(
        &mut self,
        target: &TargetReader,
        pos: Option<u64>,
        ignore_container: bool,
    ) -> Result<AtomHeader, Mp4Error> {
        self.reset()?;

        let p = pos.unwrap_or(self.pos(target)?);

        let mut header: Option<AtomHeader> = None;

        while self.pos(target)? < self.len {
            let hdr = self.header(target, None)?;
            let is_container = hdr.is_container();
            if ignore_container && is_container {
                self.seek(target, SeekFrom::Current(hdr.offset_next_rel() as i64))?;
                continue;
            }
            if hdr.contains(p) {
                header = Some(hdr);

                // If container that contains sub-containers
                if is_container {
                    continue;
                } else {
                    break;
                }
            }

            // Can not use .seek_next() without also seeking backwards <HEADER_SIZE>,
            // since this is not an atom boundary
            self.seek(target, SeekFrom::Current(hdr.offset_next_rel() as i64))?;
        }

        header.ok_or_else(|| Mp4Error::EndOfFile)
    }

    /// Returns the atom that current
    /// position is within bounds of.
    /// Optionally ignore containers (otherwise
    /// sub-atoms will be ignored).
    ///
    /// Inclusive lower bound, exclusive upper bound
    /// when checking against atom start/end offset.
    /// I.e `start_of_atom <= pos < end_of_atom`,
    /// meaning that if the upper bound is at an atom boundary,
    /// the position for the atom that starts at that offset
    /// will be returned.
    pub(crate) fn atom_closest(
        &mut self,
        target: &TargetReader,
        pos: Option<u64>,
        ignore_container: bool,
    ) -> Result<Atom, Mp4Error> {
        let header = self.header_closest(target, pos, ignore_container)?;
        Ok(self.atom(target, AtomReadOrigin::Header(header), true)?)
    }

    /// Bounds check against current position for e.g. atoms,
    /// where `bounds` is `Option<MIN, MAX>`.
    ///
    /// Does nothing if bounds is `None`.
    #[inline]
    pub(crate) fn bounds(
        &mut self,
        target: &TargetReader,
        bounds: Option<(u64, u64)>, // TODO change to std::ops::Range<u64>
    ) -> Result<(), Mp4Error> {
        if let Some((min, max)) = bounds {
            let pos = self.pos(target)?;
            if pos < min || pos > max {
                return Err(Mp4Error::BoundsError(pos, min, max));
            }
        }
        Ok(())
    }

    /// Checks if `BufReader<File>` is within `moov` boundaries.
    pub(crate) fn bounds_moov(&mut self) -> Result<(), Mp4Error> {
        self.bounds(
            &TargetReader::File,
            Some((self.moov_header.start(), self.moov_header.end()))
        )
    }
}

/// Sets read behaviour.
/// - `ReadOption::Sized(N)`: read `N` bytes
/// - `ReadOption::Until(B)`: read until sentinel `B` encountered
/// - `ReadOption::Counted`: read first byte in stream, use as byte count
pub enum ReadOption {
    /// Read `n` bytes.
    Sized(usize),
    /// Read until sentinel byte
    /// is encountered.
    Until(u8),
    /// First byte of buffer specifies
    /// remaining byte count.
    Counted,
}

/// Whether reader source is that of
/// in-memory `moov` buffer (`Cursor<Vec<u8>>`),
/// or reader over the full file (`BufReader<File>`).
#[derive(Debug, Clone, Copy)]
pub(crate) enum TargetReader {
    /// Represents the MP4-file
    File,
    /// Represents the `moov` atom
    Moov,
}

pub(crate) enum AtomReadOrigin {
    Position(SeekFrom),
    Header(AtomHeader),
    None,
}
