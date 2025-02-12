# v0.5.0
- BREAKING: Added `Track::samples()` which replaces `Track::data()` (deprecated) and yields `Sample`. `Sample` implements `Read` + `Seek` (wraps a `Cursor<Vec<u8>>`) and also holds duration and the relative timestamp for the sample (`Sample::duration()`,`Sample::relative()`).
- FIX: Added `bool` state `AtomHeader::size_64bit` to indicate whether the atom size was specified in 32bit area or 64bit area in the atom header. Some devices seem to log atom size to 64bit area even for MP4 files below the 32bit size limit, which resulted in incorrect offsets with the previous implementation.
- NEW: Added error for missing track.

# v0.4.5
- FIX Atom order agnostic method for deriving sample information for a track (offsets, sizes, etc). Some MP4-files seem to juggle the relevant atoms around (`stco`, `stts`, `stco`/`co64`, `stsz`).

# v0.4.0
- BREAKING added optional position to all read methods for `Mp4Reader`, `Mp4`, and `Atom`.
- NEW `Mp4::new()` now reads `moov` into memory (`Cursor<Vec<u8>>`) in a (naive?) attempt to raise performance by speeding up seek times for especially "spinning platter" disks.
- NEW added `Track` struct, which represents a single track in the MP4. Convenience compilation of data from various atoms in a `trak`. Has `data()` and `timestamps()` methods, which return iterators over the samples as raw bytes and increasing timestamps, respectively, for the track.
- NEW added methods for finding various attributes such as frame rate, time scale
- NEW attempt to bounds check `Atom` on read (not seek).
- NEW Added flag to store whether atom size was set as 64bit since this can not be determined after parse if atom is not of 64 bit sized (some cameras seem to exclusively use 64bit size, even for sub-64bit atom sizes)
- FIX Three letter language code in `mdhd` atom should now be correct.
- FIX Deriving sample offset, size, duration should now be correct.
- FIX Fixed incorrect timing, and frame rate calculations.
- FIX Changed and fixed how `tmcd` section (if present) inside `stsd` is read
