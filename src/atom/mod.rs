//! A few convenience MP4 atom structures and methods for deriving offsets and timing.

mod stco;
mod stsz;
mod stts;
mod ctts;
mod tmcd;
mod udta;
mod hdlr;
mod mvhd;
mod atom;
mod header;

pub use atom::Atom;
pub use header::AtomHeader;
pub use stco::{Stco, Co64};
pub use stsz::Stsz;
pub use stts::Stts;
pub use ctts::Ctts;
pub use tmcd::Tmcd;
pub use hdlr::Hdlr;
pub use mvhd::Mvhd;
pub use udta::{Udta, UdtaField};