//! MP4 atom. Bounds checked at read, not seek.
//!
//! See: <https://developer.apple.com/documentation/quicktime-file-format/atoms>.

use std::io::{Cursor, SeekFrom, Read};

use binrw::{BinRead, Endian};

use crate::{atom_types::Stsc, errors::Mp4Error, fourcc::FourCC, reader::{Mp4Reader, ReadOption, TargetReader}, Mdhd, Vmhd};

use crate::{Tkhd, AtomHeader, Co64, Dref, Elst, Ftyp, Hdlr, Mvhd, Sdtp, Smhd, Stco, Stsd, Stss, Stsz, Stts, Tmcd};

/// MP4 atom.
#[derive(Debug)]
pub struct Atom<'a> {
    /// Header
    pub header: AtomHeader,
    /// Reader over MP4, starting
    /// after atom header.
    reader: &'a mut Mp4Reader,
    /// Specifies wether the reader
    /// is a `BufReader<File>`
    /// or `Cursor<Vec<u8>>` over the `moov` atom.
    target: TargetReader
}

impl <'a> Atom<'a> {
    /// Creates a new `Atom`.
    ///
    /// Set `seek` to `true` to ensure the
    /// reader position is at dataload,
    /// otherwise there are no guarantees
    /// this is the case.
    pub(crate) fn new(
        header: &AtomHeader,
        reader: &'a mut Mp4Reader,
        target: &TargetReader,
        seek: bool
    ) -> Result<Self, Mp4Error> {
        if seek {
            reader.seek(target, SeekFrom::Start(header.data_offset()))?;
        }
        Ok(Self {
            header: header.to_owned(),
            reader,
            target: target.to_owned()
        })
    }

    /// Total size of the atom in bytes.
    pub fn len(&self) -> u64 {
        self.header.atom_size
    }

    /// Returns atom FourCC name.
    pub fn name(&self) -> FourCC {
        self.header.name.to_owned()
    }

    /// Returns size of data load in bytes.
    pub fn data_size(&self) -> u64 {
        self.header.data_size()
    }

    /// Returns an in-memory buffer over data payload
    /// as a cursor.
    ///
    /// Note that e.g. the `mdat` atom may be many GB in size.
    pub fn cursor(&mut self) -> Result<Cursor<Vec<u8>>, Mp4Error> {
        self.reader.cursor(
            &self.target,
            self.data_size() as usize,
            None,
            Some((self.min(), self.max()))
        )
    }

    /// Read the full data load (following the header) into `Vec<u8>`.
    pub fn read_data(&mut self) -> Result<Vec<u8>, Mp4Error> {
        // ensure position is after header
        self.verify_pos(self.data_offset())?;
        self.reader.read_bytes(
            &self.target,
            ReadOption::Sized(self.data_size() as usize),
            None,
            Some((self.min(), self.max()))
        )
    }

    /// Seek from current position.
    ///
    /// Note that seeking is not bounds checked,
    /// only reading is.
    pub fn seek(&mut self, offset_from_current: i64) -> Result<u64, Mp4Error> {
        let new_pos = self.reader.seek(&self.target, SeekFrom::Current(offset_from_current))?;
        Ok(new_pos)
    }

    pub fn read_one<T>(
        &mut self,
        endian: Endian,
        pos: Option<SeekFrom>,
    ) -> Result<T, Mp4Error>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.reader.read_one(
            &self.target,
            endian,
            pos,
            Some((self.min(), self.max()))
        )
        // match pos {
        //     Some(p) => self.reader.read_one(
        //         &self.target,
        //         endian,
        //         p,
        //         Some((self.min(), self.max()))
        //     ),
        //     None => self.reader.read_one(
        //         &self.target,
        //         endian,
        //         Some((self.min(), self.max()))
        //     )
        // }
    }

    pub fn read_many<T>(
        &mut self,
        n: usize,
        endian: Endian,
        pos: Option<SeekFrom>,
    ) -> Result<Vec<T>, Mp4Error>
        where
        T: BinRead,
        <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.reader.read_many::<T>(
            &self.target,
            endian,
            n,
            pos,
            Some((self.min(), self.max()))
        )
        // match pos {
        //     Some(p) => self.reader.read_many_at::<T>(
        //         &self.target,
        //         endian,
        //         n,
        //         p,
        //         Some((self.min(), self.max()))
        //     ),
        //     None => self.reader.read_many::<T>(
        //         &self.target,
        //         endian,
        //         n,
        //         Some((self.min(), self.max()))
        //     ),
        // }
    }

    /// Attempt to read `FourCC` at current position.
    pub fn read_fourcc(&mut self) -> Result<FourCC, Mp4Error> {
        Ok(FourCC::from_u32(
            self.read_one::<u32>(Endian::Big, None)?
        ))
    }

    /// Read entire atom data load to string.
    pub fn read_to_string(&mut self) -> Result<String, Mp4Error> {
        let mut buf = String::new();
        let _n = self.cursor()?
            .read_to_string(&mut buf);
        Ok(buf)
    }

    /// Current position of the reader.
    pub fn pos(&mut self) -> Result<u64, Mp4Error> {
        Ok(self.reader.seek(&self.target, SeekFrom::Current(0))?)
    }

    /// The absolute byte offset to the atom's start in MP4 file.
    pub fn min(&self) -> u64 {
        self.header.offset
    }

    /// The absolute byte offset to the atom's end in MP4 file.
    pub fn max(&self) -> u64 {
        self.header.offset + self.len()
    }

    /// Returns number of bytes left to read for this atom.
    pub fn remaining(&mut self) -> Result<u64, Mp4Error> {
        Ok(self.max() - self.pos()?)
    }

    /// The absolute byte offset in the MP4 file
    /// for this atom's data load (i.e. after header).
    pub fn data_offset(&self) -> u64 {
        self.header.data_offset()
    }

    /// Set reader position to start of atom data load
    /// (after header, adjusted for 64 bit atom size).
    pub fn reset(&mut self) -> Result<u64, Mp4Error> {
        Ok(self.reader.seek(&self.target, SeekFrom::Start(self.data_offset()))?)
    }

    /// Verifies that `name` (FourCC),
    /// matches that of current atom.
    fn verify_fcc(
        &self,
        name: &FourCC
    ) -> Result<(), Mp4Error> {
        if &self.header.name != name {
            return Err(Mp4Error::AtomMismatch {
                got: self.header.name.to_str().to_owned(),
                expected: name.to_str().to_owned()
            })
        }
        Ok(())
    }

    /// Verifies that `pos`
    /// matches current position.
    fn verify_pos(
        &mut self,
        pos: u64
    ) -> Result<(), Mp4Error> {
        let current = self.pos()?;
        if pos != current {
            return Err(Mp4Error::OffsetMismatch {
                got: current,
                expected: pos
            })
        }
        Ok(())
    }

    /// Parse the atom into `Stts` (sample to time)
    /// if `Atom.name` is `stts`,
    pub fn stts(&mut self) -> Result<Stts, Mp4Error> {
        self.verify_fcc(&FourCC::Stts)?;
        let atom = self.reader.read_ne::<Stts>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Stsz` (sample to size in bytes)
    /// if `Atom.name` is `stsz`,
    pub fn stsz(&mut self) -> Result<Stsz, Mp4Error> {
        self.verify_fcc(&FourCC::Stsz)?;
        let atom = self.reader.read_ne::<Stsz>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Stco` (sample to offset in bytes)
    /// if `Atom.name` is `stco`.
    pub fn stco(&mut self) -> Result<Stco, Mp4Error> {
        self.verify_fcc(&FourCC::Stco)?;
        let atom = self.reader.read_ne::<Stco>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Stsc` (sample to chunk)
    /// if `Atom.name` is `stsc`.
    pub fn stsc(&mut self) -> Result<Stsc, Mp4Error> {
        self.verify_fcc(&FourCC::Stsc)?;
        let atom = self.reader.read_ne::<Stsc>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Sts` (sync sample atom)
    /// if `Atom.name` is `stss`.
    pub fn stss(&mut self) -> Result<Stss, Mp4Error> {
        self.verify_fcc(&FourCC::Stss)?;
        let atom = self.reader.read_ne::<Stss>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Co64` (sample to offset in bytes)
    /// if `Atom.name` is `co64`.
    ///
    /// 64-bit equivalent to `stco` for
    /// file sizes above the 32-bit limit.
    pub fn co64(&mut self) -> Result<Co64, Mp4Error> {
        self.verify_fcc(&FourCC::Co64)?;
        let atom = self.reader.read_ne::<Co64>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Hdlr` if `Atom.name` is `hdlr`,
    pub fn hdlr(&mut self) -> Result<Hdlr, Mp4Error> {
        self.verify_fcc(&FourCC::Hdlr)?;
        // hdlr atom without component name set,
        // since parsing depends on what generated the mp4.
        // E.g. gopro has a counted string
        // (first byte in component name contains its length in bytes)
        // that ends with space \x20, whereas DJI cameras and sound handlers in old (?)
        // Apple mp4/quicktime is not counted and seems to be a null terminated string.
        // let mut hdlr = match self.moov {
        //     true => self.reader.moov_rdr.read_ne::<Hdlr>()?,
        //     false => self.reader.read_ne::<Hdlr>()?
        // };
        let mut hdlr = self.reader.read_ne::<Hdlr>(&self.target)?;

        // determine how many bytes left to read in atom
        let rem = self.remaining()?;

        // Below will not work for some MOV disguised as MP4.
        // hdlr.component_name = match self.reader.read_iso8859_1(ReadOption::Counted) {
        //     Ok(s) => s,
        //     // what is position if error is raised?
        //     Err(_) => match self.reader.read_iso8859_1(ReadOption::Sized(rem as usize)) {
        //         Ok(s) => s,
        //         Err(_) => self.reader.read_iso8859_1(ReadOption::Sentinel(0))?,
        //     },
        // };

        // create a Vec<u8> with remaining bytes as basis for string
        // let name_vec = self.read_n_be::<u8>(rem as usize)?;
        let name_vec = self.reader
            .read_bytes(
                &self.target,
                ReadOption::Sized(rem as usize),
                None,
                Some((self.min(), self.max()))
            )?;

        // Workaround for older MP4 (or perhaps Quicktimes only)
        // with handler names that are not counted strings.
        // Check if value of first byte is less than number of remaining bytes
        // and only then assume a counted string. May still return
        // wrong/gibberish value for old quicktimes, but shouldn't fail since
        // string is assumed to be "extended ascii" (following ISO8859-1 mapping)
        hdlr.component_name = if let Some(maybe_count) = name_vec.first() {
            if (*maybe_count as u64) <= rem {
                bytes2iso8859_1(&name_vec[1 .. *maybe_count as usize + 1])
            } else {
                bytes2iso8859_1(&name_vec)
            }
        } else {
            return Err(Mp4Error::MissingHandlerName)
        };

        self.bounds()?;

        Ok(hdlr)
    }

    /// Parse the atom into `Ftyp` if `Atom.name` is `ftyp`,
    pub fn ftyp(&mut self) -> Result<Ftyp, Mp4Error> {
        self.verify_fcc(&FourCC::Ftyp)?;
        let size = self.data_size() as u32;
        let atom = Ftyp::read_ne_args(
            &mut self.reader.file_reader,
            binrw::args! {data_size: size}
        )?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Sdtp` if `Atom.name` is `sdtp`,
    pub fn sdtp(&mut self) -> Result<Sdtp, Mp4Error> {
        self.verify_fcc(&FourCC::Sdtp)?;
        let size = self.data_size() as u32;
        let atom = Sdtp::read_ne_args(
            &mut self.reader.moov_reader,
            binrw::args! {data_size: size}
        )?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Dref` if `Atom.name` is `dref`,
    ///
    /// One per track (`trak`).
    pub fn dref(&mut self) -> Result<Dref, Mp4Error> {
        self.verify_fcc(&FourCC::Dref)?;
        let atom = self.reader.read_ne::<Dref>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Elst` if `Atom.name` is `elst`,
    ///
    /// One per track (`trak`).
    pub fn elst(&mut self) -> Result<Elst, Mp4Error> {
        self.verify_fcc(&FourCC::Elst)?;
        let atom = self.reader.read_ne::<Elst>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Smhd` if `Atom.name` is `smhd`,
    pub fn smhd(&mut self) -> Result<Smhd, Mp4Error> {
        self.verify_fcc(&FourCC::Smhd)?;
        let atom = self.reader.read_ne::<Smhd>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Vmhd` if `Atom.name` is `smhd`,
    pub fn vmhd(&mut self) -> Result<Vmhd, Mp4Error> {
        self.verify_fcc(&FourCC::Vmhd)?;
        let atom = self.reader.read_ne::<Vmhd>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Mvhd` if `Atom.name` is `mvhd`,
    pub fn mvhd(&mut self) -> Result<Mvhd, Mp4Error> {
        self.verify_fcc(&FourCC::Mvhd)?;
        let atom = self.reader.read_ne::<Mvhd>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Tkhd` if `Atom.name` is `tkhd`,
    ///
    /// One per track (`trak`).
    pub fn tkhd(&mut self) -> Result<Tkhd, Mp4Error> {
        self.verify_fcc(&FourCC::Tkhd)?;
        let atom = self.reader.read_ne::<Tkhd>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom into `Mdhd` if `Atom.name` is `mdhd`,
    ///
    /// One per track (`trak`).
    pub fn mdhd(&mut self) -> Result<Mdhd, Mp4Error> {
        self.verify_fcc(&FourCC::Mdhd)?;
        let atom = self.reader.read_ne::<Mdhd>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Timecode entry in a sample description atom (`stsd`).
    /// Layout mimics that of atoms: `size | fourcc | data load`.
    /// Found in `trak` atoms with the media handler type `tmcd`.
    ///
    /// Note: Does not set offsets.
    ///
    /// Can be used to e.g. sort clips chronologically if part of the same
    /// recording session.
    ///
    /// For a GoPro MP4, find a `trak` where the `hdlr` atom has
    /// component type `tmcd`, and component name `GoPro TCD`.
    pub fn tmcd(&mut self) -> Result<Tmcd, Mp4Error>  {
        self.verify_fcc(&FourCC::Tmcd)?;
        let atom = self.reader.read_ne::<Tmcd>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Parse the atom as video sample description `Stsd` if `Atom.name` is `stsd`.
    ///
    /// One per track (`trak`).
    pub fn stsd(&mut self) -> Result<Stsd, Mp4Error> {
        self.verify_fcc(&FourCC::Stsd)?;
        let atom = self.reader.read_ne::<Stsd>(&self.target)?;
        self.bounds()?;
        Ok(atom)
    }

    /// Bounds check against current position,
    /// to prevent reading outside atom start/end
    /// byte offsets.
    #[inline]
    pub fn bounds(&mut self) -> Result<(), Mp4Error> {
        self.reader.bounds(&self.target, Some((self.min(), self.max())))
    }
}

/// Parses bytes as ISO8859-1 string,
/// (single-byte encoding, "extended" ascii).
#[inline]
fn bytes2iso8859_1(bytes: &[u8]) -> String {
    bytes.iter()
        .filter_map(|n| match n {
            0 => None,
            _ => Some(*n as char)
        })
        .collect::<String>()
        .trim()
        .to_owned()
}
