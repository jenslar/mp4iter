//! Custom user data atom (`udta`). Fields are in raw form as `Cursor<Vec<u8>>`.
//! 
//! See: <https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-SW1>

use std::io::{Cursor, Read};

use binrw::{Endian, BinReaderExt, BinResult, BinRead};

use crate::fourcc::FourCC;

/// User data atom.
/// Field content differs between recording devices or encoders.
#[derive(Debug)]
pub struct Udta {
    pub fields: Vec<UdtaField>
}

impl Udta {
    pub fn iter(&self) -> impl Iterator<Item = &UdtaField> {
        self.fields.iter()
    }

    pub fn find(&self, fourcc: &str) -> Option<&UdtaField> {
        let fcc = FourCC::from_str(fourcc);
        self.fields.iter().find(|f| f.name == fcc)
    }
}

/// User data (`udta`) field.
/// Each field is formatted as an atom.
/// Data content types differ between recording devices or encoders.
/// E.g. GoPro cameras mix "normal" MP4 atom structure with
/// device information embedded as GPMF.
#[derive(Debug, Clone)]
pub struct UdtaField{
    /// Four CC
    pub name: FourCC,
    /// Totalt size in bytes
    pub size: u32,
    /// Data, excluding 8 byte header
    pub data: Cursor<Vec<u8>>
    // pub data: Vec<u8>
}

impl UdtaField {
    /// Number of data values.
    pub fn len(&self) -> usize {
        self.data.get_ref().len()
    }

    /// Check if the field's FourCC matches
    /// specified FourCC.
    pub fn matches(&self, fourcc: &str) -> bool {
        self.name == FourCC::from_str(fourcc)
    }
    
    /// Returns data as a string. Note that `udta` fields,
    /// may contain values of more than one type, in which
    /// case this method will fail.
    /// 
    /// E.g. GoPro `FIRM` (firmware version, model identifier):
    /// "H22.01.02.01.00", "LENS" (lens info/serial?): "LAJ8052832102184".
    pub fn to_string(&self) -> Option<String> {
        let mut buf = String::new();
        self.data.to_owned().read_to_string(&mut buf).ok()?;
        return Some(buf)
    }

    /// Returns data as `Vec<T>`. Note that `udta` fields,
    /// may contain values of more than one type, in which
    /// case this method will fail.
    pub fn to_type<T>(&self, endian: Endian) -> BinResult<Vec<T>>
        where
        T: BinRead,
        <T as BinRead>::Args<'static>: Sized + Clone + Default
    {
        let n = self.len() / std::mem::size_of::<T>();
        match endian {
            Endian::Big => (0..n).into_iter()
                .map(|_| self.data.to_owned().read_be::<T>())
                .collect(),
            Endian::Little => (0..n).into_iter()
                .map(|_| self.data.to_owned().read_le::<T>())
                .collect(),
        }
    }
}

// udta, See: https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-SW1
// GoPro 
// ©xyz: len 22, interpret last 18 as ascii char -> coords
// FIRM/LENS: ascii char -> e.g. FIRM:
//      julia> join(Char.([72, 50, 50, 46, 48, 49, 46, 48, 50, 46, 48, 49, 46, 48, 48]))
//      "H22.01.02.01.00" hero11
//      julia> join(Char.([72, 68, 53, 46, 48, 50, 46, 48, 50, 46, 48, 48, 46, 48, 48]))
//      "HD5.02.02.00.00" hero5
//      FIRM udta raw = FMWR udta gpmf
//      LENS udta raw = LINF udta gpmf
//      CAME udta raw = CINF udta gpmf, binary/not string
//      SETT udta raw = ???, binary/not string