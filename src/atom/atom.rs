//! MP4 atom.

use std::io::{Cursor, Read, Seek, SeekFrom};

use binread::{BinReaderExt, BinRead, BinResult, Endian};

use crate::{errors::Mp4Error, fourcc::FourCC, Offset};

use super::{AtomHeader, Hdlr, Stco, Stsz, Stts, Udta, UdtaField, stco::Co64, hdlr::ComponentType, Tmcd};

/// MP4 atom.
pub struct Atom {
    /// Header
    pub header: AtomHeader,
    /// Raw data load, excluding 8 byte header (size + name).
    pub cursor: Cursor<Vec<u8>>
}

impl Atom {
    // pub fn new(cursor: &mut Cursor<Vec<u8>>) {

    // }

    pub fn find(&self) {}

    pub fn size(&self) -> u64 {
        self.header.size
    }

    /// Seek from current position
    pub fn seek(&mut self, offset_from_current: i64) -> Result<u64, std::io::Error> {
        self.cursor.seek(SeekFrom::Current(offset_from_current))
    }

    /// Read single Big Endian value.
    pub fn read<T: Sized + BinRead>(&mut self) -> BinResult<T> {
        self.cursor.read_be::<T>()
    }

    /// Read multiple Big Endian values of the same primal type.
    pub fn iter_read<T: Sized + BinRead>(&mut self, repeats: usize) -> BinResult<Vec<T>> {
        (0..repeats).into_iter()
            .map(|_| self.read::<T>())
            .collect()
    }

    /// Read cursor to string.
    pub fn read_to_string(&mut self) -> std::io::Result<String> {
        // get number of bytes (NOT number of UTF-8 graphemes).
        let len = self.cursor.get_ref().len();
        let mut string = String::with_capacity(len);
        self.cursor.read_to_string(&mut string)?;
        Ok(string)
    }

    /// Seek to next atom if nested.
    pub fn next(&mut self) -> Result<u64, Mp4Error> {
        let size = self.read::<u32>()?;
        self.cursor.seek(SeekFrom::Current(size as i64 - 4))
            .map_err(|e| Mp4Error::IOError(e))
    }

    pub fn pos(&self) -> u64 {
        self.cursor.position()
    }

    /// Set atom cursor position to start of cursor.
    pub fn reset(&mut self) {
        self.cursor.set_position(0)
    }

    /// Get name of atom (Four CC) at current offset.
    /// Supports Four CC with byte values above 127 (non-standard ASCII)
    /// if the numerical values map to ISO8859-1,
    /// e.g. GoPro uses Four CC `©xyz` in `udta` atom.
    pub fn name(&mut self, pos: u64) -> Result<String, Mp4Error> {
        // let mut cursor = self.read_at(self.offset + 4, 4)?;
        let bytes: Vec<u8> = (pos..pos+4).into_iter()
            .map(|_| self.cursor.read_be::<u8>())
            .collect::<BinResult<Vec<u8>>>()?;
        let name: String = bytes.iter()
            .map(|b| *b as char)
            .collect();
        Ok(name)
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

    /// Parse `Atom` into `Stts` (time-to sample) if `Atom.name` is `stts`,
    pub fn stts(&mut self) -> Result<Stts, Mp4Error> {
        self.match_name(&FourCC::Stts)?;

        // Seek past version (1 byte) + flags (3 bytes)
        self.cursor.seek(SeekFrom::Current(4))?;
        let no_of_entries = self.read::<u32>()?;

        let mut time_to_sample_table: Vec<u32> = Vec::new();
        for _ in 0..no_of_entries {
            let sample_count = self.read::<u32>()?;
            let sample_duration = self.read::<u32>()?;
            time_to_sample_table.extend(vec![sample_duration; sample_count as usize])
            // time_to_sample_table.append(&mut vec![sample_duration; sample_count as usize])
        }

        Ok(Stts::new(time_to_sample_table))
    }

    /// Parse `Atom` into `Stsz` (sample to size in bytes) if `Atom.name` is `stsz`,
    pub fn stsz(&mut self) -> Result<Stsz, Mp4Error> {
        self.match_name(&FourCC::Stsz)?;

        // Seek past version (1 byte) + flags (3 bytes)
        self.cursor.seek(SeekFrom::Current(4))?;

        let sample_size = self.read::<u32>()?;
        let no_of_entries = self.read::<u32>()?;

        let sizes = match sample_size {
            0 => {
                (0..no_of_entries).into_iter()
                    .map(|_| self.read::<u32>())
                    .collect()
            }
            // Is below really correct? If all samples have the same size
            // is no_of_entries still representative?
            _ => Ok(vec![sample_size; no_of_entries as usize]),
        };

        Ok(Stsz::new(sizes?))
    }

    /// Parse `Atom` into `Stco` (sample to offset in bytes)
    /// if `Atom.name` is `stco`.
    pub fn stco(&mut self) -> Result<Stco, Mp4Error> {
        self.match_name(&FourCC::Stco)?;

        // Seek past version (1 byte) + flags (3 bytes)
        self.cursor.seek(SeekFrom::Current(4))?;

        // let sample_size = self.read::<u32>()?;
        let no_of_entries = self.read::<u32>()?;

        let offsets: BinResult<Vec<u32>> = (0..no_of_entries).into_iter()
            .map(|_| self.read::<u32>())
            .collect();

        Ok(Stco::new(offsets?))
    }

    /// Parse `Atom` into `Co64` (sample to size in bytes)
    /// if `Atom.name` is `co64`. Equivalent to `stco` for
    /// file sizes above 32bit limit.
    pub fn co64(&mut self) -> Result<Co64, Mp4Error> {
        self.match_name(&FourCC::Co64)?;

        // Seek past version (1 byte) + flags (3 bytes)
        self.cursor.seek(SeekFrom::Current(4))?;

        // let sample_size = self.read::<u32>()?;
        let no_of_entries = self.read::<u32>()?;

        let offsets: BinResult<Vec<u64>> = (0..no_of_entries).into_iter()
            .map(|_| self.read::<u64>())
            .collect();

        Ok(Co64::new(offsets?))
    }

    /// Parse `Atom` into `Hdlr` if `Atom.name` is `hdlr`,
    pub fn hdlr(&mut self) -> Result<Hdlr, Mp4Error> {
        self.match_name(&&FourCC::Hdlr)?;

        // Seek past version (1 byte) + flags (3 bytes)
        self.cursor.seek(SeekFrom::Current(4))?;

        let component_type = ComponentType::from(self.cursor.read_be::<u32>()?);
        let component_sub_type = self.cursor.read_be::<u32>()?;
        let component_manufacturer = self.cursor.read_be::<u32>()?;
        let component_flags = self.cursor.read_be::<u32>()?;
        let component_flags_mask = self.cursor.read_be::<u32>()?;
        let component_name_size = self.cursor.read_be::<u8>()?;
        let mut component_name = String::with_capacity(component_name_size as usize);
        let read_bytes = self.cursor.read_to_string(&mut component_name)?;
        if read_bytes != component_name_size as usize {
            return Err(Mp4Error::ReadMismatch{got: read_bytes as u64, expected: component_name_size as u64})
        }

        Ok(Hdlr{
            component_type,
            component_sub_type,
            component_manufacturer,
            component_flags,
            component_flags_mask,
            component_name: component_name.trim().to_owned()
        })
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
        let tmcd: Tmcd = BinRead::read(&mut self.cursor)?;
        // tmcd.offsets = self.offsets_at_current_pos()?;
        // println!("read tmcd offsets: {:?}", tmcd.offsets);
        Ok(tmcd) // offsets not set
    }

    /// User data atom. Contains custom data depending on vendor.
    pub fn udta(&mut self) -> Result<Udta, Mp4Error> {
        self.match_name(&&FourCC::Udta)?;

        let mut fields: Vec<UdtaField> = Vec::new();

        // Atom::size includes 8 byte header
        // TODO remove header bytes from cursor instead? Cleaner, consistent
        while self.cursor.position() < self.header.size - 8 {
            let size = self.read::<u32>()?;
            let name = FourCC::from_str(&self.name(self.cursor.position())?);

            let mut buf: Vec<u8> = vec![0; size as usize - 8];
            self.cursor.read_exact(&mut buf)?;

            let field = UdtaField {
                name,
                size,
                data: Cursor::new(buf)
            };

            fields.push(field)
        }

        Ok(Udta{fields})
    }

    /// Internal. Returns offsets for current `trak`.
    /// I.e. assumes upcoming atoms are `stts` (sample to time),
    /// `stsz` (sample to size), `stco` (sample to offset) atoms, and process these.
    fn offsets_at_current_pos(&mut self) -> Result<Vec<Offset>, Mp4Error> {
        // TODO 230112 order of 'st..' atoms consistent?
        // TODO        possible solution: find hdlr, read hdlr as atom, then inside hdlr cursor find stts etc
        println!("getting offsets at pos {}", self.pos());
        let stts = self.stts()?;
        println!("{stts:?}");
        let stsz = self.stsz()?;
        println!("{stsz:?}");
        // Check if file size > 32bit limit,
        // but always output 64bit offsets
        let stco = Co64::from(self.stco()?);
        // let stco = match self.cursor.len() > u32::MAX as u64 {
        //     true => self.co64()?,
        //     false => Co64::from(self.stco()?),
        // };
        println!("{stsz:?}");

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
