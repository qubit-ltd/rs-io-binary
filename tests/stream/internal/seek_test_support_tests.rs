use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

pub(crate) struct RejectingCurrentSeekReader {
    bytes: Vec<u8>,
    position: usize,
}

impl RejectingCurrentSeekReader {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, position: 0 }
    }
}

impl Read for RejectingCurrentSeekReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self.bytes.len() - self.position;
        let count = remaining.min(buffer.len());
        buffer[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        Ok(count)
    }
}

impl Seek for RejectingCurrentSeekReader {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        match position {
            SeekFrom::Start(position) => {
                self.position = usize::try_from(position)
                    .map_err(|_| Error::new(ErrorKind::InvalidInput, "seek position exceeds usize"))?;
                Ok(position)
            }
            SeekFrom::Current(_) => Err(Error::other("current seek rejected")),
            SeekFrom::End(_) => Err(Error::new(ErrorKind::Unsupported, "unsupported seek")),
        }
    }
}
