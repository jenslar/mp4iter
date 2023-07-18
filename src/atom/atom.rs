//! MP4 atom.

use std::{io::{Cursor, Read, Seek, SeekFrom, BufReader}, fs::File};

use binrw::{BinReaderExt, BinRead, BinResult, Endian};

use crate::{errors::Mp4Error, fourcc::FourCC};

use super::{AtomHeader, Hdlr, Stco, Stsz, Stts, Udta, UdtaField, stco::Co64, Tmcd, Mvhd};

/// MP4 atom.
pub struct Atom<'a> {
    /// Header
    pub header: AtomHeader,
    /// Reader over MP4, starting
    /// after atom header.
    reader: &'a mut BufReader<File>
}

impl <'a> Atom<'a> {
    pub fn new(
        header: &AtomHeader,
        reader: &'a mut BufReader<File>
    ) -> Self {
        Self {
            header: header.to_owned(),
            reader
        }
    }

    // pub fn find(&self) {}

    /// Total size of the atom in bytes.
    pub fn size(&self) -> u64 {
        self.header.size
    }

    pub fn name(&self) -> FourCC {
        self.header.name.to_owned()
    }

    pub fn data_size(&self) -> u64 {
        self.header.data_size()
    }

    /// Seek from current position
    pub fn seek(&mut self, offset_from_current: i64) -> Result<u64, std::io::Error> {
        self.reader.seek(SeekFrom::Current(offset_from_current))
    }

    /// Read single Big Endian value.
    pub fn read<T>(&mut self) -> BinResult<T>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        self.reader.read_be::<T>()
    }

    /// Read multiple Big Endian values of the same primal type.
    pub fn iter_read<T>(&mut self, repeats: usize) -> BinResult<Vec<T>>
        where
            T: BinRead,
            <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        (0..repeats).into_iter()
            .map(|_| self.read::<T>())
            .collect()
    }

    /// Attempt to read FourCC as `String` at current position.
    pub fn read_name(&mut self) -> BinResult<String> {
        Ok(self.reader.read_type::<[u8; 4]>(Endian::Big)?
            .iter()
            .map(|n| *n as char)
            .collect())
    }

    /// Attempt to read `FourCC` at current position.
    pub fn read_fourcc(&mut self) -> BinResult<FourCC> {
        Ok(FourCC::from_slice(self.reader.read_type::<[u8; 4]>(Endian::Big)?.as_slice()))
    }

    
    /// Read entire atom data load to string.
    pub fn read_to_string(&mut self) -> std::io::Result<String> {
        // limit read to size of atom data load
        let len = self.data_size();
        let mut buf = Vec::with_capacity(len as usize);
        self.reader.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&mut buf).to_string())
    }

    /// Seek to next atom if nested.
    pub fn next(&mut self) -> Result<u64, Mp4Error> {
        let size = self.read::<u32>()?;
        // TODO should probably reset instead then set
        // TODO offset to start of atom, *then* seek to next
        self.reader.seek(SeekFrom::Current(size as i64 - 4))
            .map_err(|e| e.into())
    }

    /// Current position of the reader.
    pub fn pos(&mut self) -> std::io::Result<u64>  {
        self.reader.seek(SeekFrom::Current(0))
    }

    /// The absolute byte offset to the atom's start in MP4 file.
    pub fn start(&self) -> u64 {
        self.header.offset
    }

    /// The absolute byte offset to the atom's end in MP4 file.
    pub fn end(&self) -> u64 {
        self.header.offset + self.size()
    }

    /// Returns number of bytes left to read for this atom.
    pub fn remaining(&mut self) -> std::io::Result<u64> {
        Ok(self.end() - self.pos()?)
    }

    /// The absolute byte offset for this atom's data load
    /// (i.e. after header) in the MP4 file.
    pub fn data_offset(&self) -> u64 {
        self.header.data_offset()
    }

    /// Set reader position to start of atom data load
    /// (after header).
    pub fn reset(&mut self) -> std::io::Result<u64> {
        self.reader.seek(SeekFrom::Start(self.data_offset()))
    }

    /// Ensures user specified name (Four CC),
    /// matches that of current `Atom`.
    fn match_name(&self, name: &FourCC) -> Result<(), Mp4Error> {
        if &self.header.name != name {
            Err(Mp4Error::AtomMismatch{
                got: self.header.name.to_str().to_owned(),
                expected: name.to_str().to_owned()
            })
        } else {
            Ok(())
        }
    }

    /// Parse the atom into `Stts` (sample to time) if `Atom.name` is `stts`,
    pub fn stts(&mut self) -> Result<Stts, Mp4Error> {
        self.match_name(&FourCC::Stts)?;
        self.reader.read_ne::<Stts>().map_err(|e| e.into())
    }

    /// Parse the atom into `Stsz` (sample to size in bytes) if `Atom.name` is `stsz`,
    pub fn stsz(&mut self) -> Result<Stsz, Mp4Error> {
        self.match_name(&FourCC::Stsz)?;
        self.reader.read_ne::<Stsz>().map_err(|e| e.into())
    }

    /// Parse the atom into `Stco` (sample to offset in bytes)
    /// if `Atom.name` is `stco`.
    pub fn stco(&mut self) -> Result<Stco, Mp4Error> {
        self.match_name(&FourCC::Stco)?;
        self.reader.read_ne::<Stco>().map_err(|e| e.into())
    }

    /// Parse the atom into `Co64` (sample to size in bytes)
    /// if `Atom.name` is `co64`. 64-bit equivalent to `stco` for
    /// file sizes above 32bit limit.
    pub fn co64(&mut self) -> Result<Co64, Mp4Error> {
        self.match_name(&FourCC::Co64)?;
        self.reader.read_ne::<Co64>().map_err(|e| e.into())
    }

    /// Parse the atom into `Hdlr` if `Atom.name` is `hdlr`,
    pub fn hdlr(&mut self) -> Result<Hdlr, Mp4Error> {
        self.match_name(&&FourCC::Hdlr)?;
        // hdlr atom without component name set,
        // since parsing depends on what generated the mp4.
        // E.g. gopro has a counted string
        // (first byte in component name contains its length in bytes)
        // that ends with space \x20, whereas sound handler in old (?)
        // Apple mp4/quicktimes is not counted and seems to be a null terminated string.
        let mut hdlr = self.reader.read_ne::<Hdlr>()?;

        // determine how many bytes left to read in atom
        let rem = self.remaining()?;

        // create a Vec<u8> with remaining bytes as basis for string
        let name_vec = self.iter_read::<u8>(rem as usize)?;

        // !!! check if first byte is alphanumeric 0_u8.is_alphanumeric(), if not and less than remainder use as count?
        // let b = b"\x0BGoPro AVC\x20\x20"; // -> do iter windows(b.len()) and a name<&str>.as_bytes() comparins for each iter -> bool

        // Workaround for older MP4 (or Quicktimes) with handler names that are not counted strings.
        hdlr.component_name = if let Some(maybe_count) = name_vec.first() {
            if (*maybe_count as u64) <= rem {
                name_vec[1 .. *maybe_count as usize + 1].iter()
                    .filter_map(|n| match n {
                        0 => None,
                        _ => Some(*n as char)
                    })
                    .collect::<String>()
                    .trim()
                    .to_owned()
            } else {
                name_vec.iter()
                    .filter_map(|n| match n {
                        0 => None,
                        _ => Some(*n as char)
                    })
                    .collect::<String>()
                    .trim()
                    .to_owned()
            }
        } else { // !!! isnt this for a vec with len = 0 so not needed???
            name_vec.iter()
                .filter_map(|n| match n {
                    0 => None,
                    _ => Some(*n as char)
                })
                .collect::<String>()
                .trim()
                .to_owned()
        };

        Ok(hdlr)
    }

    /// Parse the atom into `Mvhd` if `Atom.name` is `mvhd`,
    pub fn mvhd(&mut self) -> Result<Mvhd, Mp4Error> {
        self.match_name(&&FourCC::Mvhd)?;
        self.reader.read_ne::<Mvhd>().map_err(|e| e.into())
    }

    /// Timecode entry in a sample description atom (`stsd`).
    /// Layout mimcs that of atoms: `size | fourcc | data load`.
    /// Found in `trak` atoms with the media handler type `tmcd`.
    /// 
    /// Does not set offsets.
    /// 
    /// Can be used to e.g. sort clipschronologically if part of the same
    /// recording session.
    /// 
    /// For a GoPro MP4, find a `trak` where the `hdlr` atom has
    /// component type `tmcd`, and component name `GoPro TCD`.
    pub fn tmcd(&mut self) -> Result<Tmcd, Mp4Error>  {
        let _ = self.seek(6)?; // seek past 'reserved' 6-byte section
        self.reader.read_ne::<Tmcd>().map_err(|e| e.into())
    }

    /// User data atom. Contains custom data depending on vendor.
    pub fn udta(&mut self) -> Result<Udta, Mp4Error> {
        self.match_name(&&FourCC::Udta)?;

        let mut fields: Vec<UdtaField> = Vec::new();

        // Atom::size includes 8 byte header
        // TODO remove header bytes from cursor instead? Cleaner, consistent
        // while self.reader.position() < self.header.size - 8 {
        while self.pos()? < self.end() {
            let size = self.read::<u32>()?;
            let name = self.read_fourcc()?;

            let mut buf: Vec<u8> = vec![0; size as usize - 8];
            self.reader.read_exact(&mut buf)?;

            let field = UdtaField {
                name,
                size,
                data: Cursor::new(buf) // can not lend bufreader here
            };

            fields.push(field)
        }

        Ok(Udta{fields})
    }
}
