//! A few convenience MP4 atom structures and methods for deriving offsets and timing.

mod stco;
mod stsz;
mod stts;
mod tmcd;
mod udta;
mod hdlr;
mod atom;
mod header;

pub use atom::Atom;
pub use header::AtomHeader;
pub use stco::{Stco, Co64};
pub use stsz::Stsz;
pub use stts::Stts;
pub use tmcd::Tmcd;
pub use hdlr::Hdlr;
pub use udta::{Udta, UdtaField};