use crate::*;

/// The `Read` trait allows for reading bytes from a source.
pub trait Read {
    /// Pull some bytes from this source into the specified buffer, returning
    /// how many bytes were read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Read all bytes until EOF in this source, placing them into `buf`.
    fn read_all(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut start_len = buf.len();
        let mut count = 0;
        loop {
            // FIXME: read data into the buffer
            let mut tmp_buf = vec![0; 1024];
            match self.read(&mut tmp_buf) {
                Ok(0) => break,
                Ok(n) => {
                    tmp_buf.truncate(n);
                //      - extend the buffer if it's not big enough
                    buf.extend_from_slice(&tmp_buf);
                //      - update the length of the buffer if data was read
                    count += n;
                }
                //      - break if the read returns 0 or Err
                Err(e) => return Err(e),
            }
        }
        Ok(count)
    }
}

/// The `Write` trait allows for writing bytes to a source.
///
/// NOTE: Leave here to ensure flexibility for the optional lab task.
pub trait Write {
    /// Write a buffer into this writer, returning how many bytes were written.
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Flush this output stream, ensuring that all intermediately buffered
    /// contents reach their destination.
    fn flush(&mut self) -> Result<()>;

    /// Attempts to write an entire buffer into this writer.
    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        // not required for lab
        todo!()
    }
}

/// Enumeration of possible methods to seek within an I/O object.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    Start(usize),

    /// Sets the offset to the size of this object plus the offset.
    End(isize),

    /// Sets the offset to the current position plus the offset.
    Current(isize),
}

/// The `Seek` trait provides a cursor within byte stream.
pub trait Seek {
    /// Seek to an offset, in bytes, in a stream.
    fn seek(&mut self, pos: SeekFrom) -> Result<usize>;
}

pub trait FileIO: Read + Write + Seek + Send + Sync {}

impl<T: Read + Write + Seek + Send + Sync> FileIO for T {}
