//! Track sample. Wrapper over in-memory buffer `Cursor<Vec<u8>>`,
//! complete with sample duration and relative timestamp.

use std::io::{BufRead, Cursor, Read, Seek, SeekFrom};

use time::Duration;

use crate::{Mp4Error, Mp4Reader, TargetReader};

use super::SampleOffset;

#[derive(Debug, Default, Clone)]
pub struct Sample {
    relative_time: Duration,
    sample_duration: Duration,
    reader: Cursor<Vec<u8>>
}

impl Into<Cursor<Vec<u8>>> for Sample {
    fn into(self) -> Cursor<Vec<u8>> {
        self.reader
    }
}

impl Seek for Sample {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

impl Read for Sample {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl BufRead for Sample {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt)
    }
}

impl Sample {
    pub(crate) fn new(
        reader: &mut Mp4Reader,
        sample_offset: SampleOffset,
    ) -> Result<Sample, Mp4Error> {

        let file_pos_abs = i64::try_from(reader.pos(&TargetReader::File)?)?;
        let file_pos_abs_next = i64::try_from(sample_offset.position)?;
        // Possibly wrong assumption that relative seek may be faster
        // on at least spinning disks...
        let seek = SeekFrom::Current(file_pos_abs_next - file_pos_abs);

        let mut sample = Self::default();
        sample.reader = reader.cursor(
            &TargetReader::File,
            sample_offset.size as usize,
            Some(seek),
            None
        )?;
        sample.sample_duration = sample_offset.duration;

        Ok(sample)
    }

    /// Set relative timestamp,
    /// counted from video start.
    pub fn with_time(self, relative_time: Duration) -> Self {
        Self {
            relative_time,
            ..self
        }
    }

    /// Returns sample duration.
    pub fn duration(&self) -> Duration {
        self.sample_duration
    }

    /// Returns relative time since start of video.
    pub fn relative(&self) -> Duration {
        self.relative_time
    }

    /// Returns relative time since start of video
    /// sample duration as the tuple
    /// `(RELATIVE_TIME, SAMPLE_DURATION)`.
    pub fn time(&self) -> (Duration, Duration) {
        (self.relative(), self.duration())
    }

    /// Returns the raw bytes as a slice.
    pub fn raw(&self) -> &[u8] {
        self.reader.get_ref()
    }

    /// Sample size in bytes.
    pub fn len(&self) -> usize {
        self.reader.get_ref().len()
    }
}
