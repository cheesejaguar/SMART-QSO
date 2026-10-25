//! Bounded writer over a byte slice, for building frames without allocation.

use core::fmt;

/// Writes into `buf`, failing (never truncating silently) when it is full
pub(crate) struct SliceWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> SliceWriter<'a> {
    pub(crate) fn new(buf: &'a mut [u8]) -> Self {
        SliceWriter { buf, len: 0 }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn push(&mut self, bytes: &[u8]) -> Result<(), Full> {
        let end = self.len.checked_add(bytes.len()).ok_or(Full)?;
        let dst = self.buf.get_mut(self.len..end).ok_or(Full)?;
        dst.copy_from_slice(bytes);
        self.len = end;
        Ok(())
    }
}

/// The output buffer is too small
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Full;

impl fmt::Write for SliceWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

/// Big-endian u32 from the first four bytes, if there are four
pub(crate) fn be_u32(bytes: &[u8]) -> Option<u32> {
    let word: [u8; 4] = bytes.get(..4)?.try_into().ok()?;
    Some(u32::from_be_bytes(word))
}
