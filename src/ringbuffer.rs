use std::io::ErrorKind;

/// A ringbuffer that can read from std::io::Read and write to std::io::Write.
pub struct Ringbuffer<const C: usize> {
    buf: [u8; C],
    write_idx: usize,
    read_idx: usize,
}

pub enum IoResult<'b> {
    Ok(Bytes<'b>),
    EOF(Bytes<'b>),
    Err {
        bytes: Bytes<'b>,
        #[expect(dead_code, reason = "may want to use the base io error later")]
        err: std::io::Error,
    },
}

impl IoResult<'_> {
    pub fn bytes(&self) -> Bytes<'_> {
        match self {
            IoResult::Ok(bytes) => *bytes,
            IoResult::EOF(bytes) => *bytes,
            IoResult::Err { bytes, .. } => *bytes,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Bytes<'b> {
    buf: &'b [u8],
    to: usize,
    cur: usize,
}

impl<'b> Iterator for Bytes<'b> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.to {
            None
        } else {
            let idx = self.cur;
            self.cur += 1;
            if self.cur == self.buf.len() {
                self.cur = 0;
            }
            Some(self.buf[idx])
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = if self.to < self.cur {
            self.to + self.buf.len() - self.cur
        } else {
            self.to - self.cur
        };
        (len, Some(len))
    }
}

impl ExactSizeIterator for Bytes<'_> {}

impl<const C: usize> Ringbuffer<C> {
    pub fn new() -> Self {
        Ringbuffer {
            buf: [0; C],
            write_idx: 0,
            read_idx: 0,
        }
    }

    /// Get a contiguous range of unwritten bytes.
    fn unfilled(&self) -> std::ops::Range<usize> {
        let start = self.write_idx;
        let end = if self.read_idx <= self.write_idx {
            C
        } else {
            self.read_idx
        };

        start..end
    }

    /// Get a contiguous range of unread bytes.
    fn filled(&self) -> std::ops::Range<usize> {
        let start = self.read_idx;
        let end = if self.read_idx <= self.write_idx {
            self.write_idx
        } else {
            C
        };

        start..end
    }

    fn bytes(&self, from: usize, to: usize) -> Bytes<'_> {
        Bytes {
            buf: &self.buf,
            to,
            cur: from,
        }
    }

    pub fn capacity(&self) -> usize {
        C
    }

    /// The number of unread bytes written to the ringbuffer.
    pub fn len(&self) -> usize {
        if self.write_idx < self.read_idx {
            self.write_idx + C - self.read_idx
        } else {
            self.write_idx - self.read_idx
        }
    }

    /// Returns `true` if the ringbuffer contains no unread bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the ringbuffer is full.
    pub fn is_full(&self) -> bool {
        self.len() == C
    }

    /// Pull bytes from the source into the ringbuffer. This performs up to two `read` operations
    /// on the source. Returns an iterator over the bytes read and a status indicating the number
    /// of bytes read as well as whether EOF was reached.
    ///
    /// IO errors `ErrorKind::WouldBlock` and `ErrorKind::Interrupted` are ignored. Other errors
    /// are returned.
    pub fn read<'b>(&'b mut self, read: &mut impl std::io::Read) -> IoResult<'b> {
        let mut bytes_read = 0;
        let mut eof = false;
        let from = self.write_idx;

        // in case enough data is available we may wrap around the ringbuffer immediately
        for _ in 0..2 {
            let unfilled = {
                let range = self.unfilled();
                &mut self.buf[range]
            };
            if unfilled.is_empty() {
                break;
            }

            match read.read(unfilled) {
                Ok(n) => {
                    bytes_read += n;
                    self.write_idx += n;
                    if self.write_idx == C {
                        self.write_idx = 0;
                    }

                    if n == 0 {
                        eof = true;
                        break;
                    }

                    if bytes_read < unfilled.len() {
                        break;
                    }
                }
                Err(err) => {
                    // ignore some errors
                    if matches!(err.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) {
                        break;
                    } else {
                        return IoResult::Err {
                            bytes: self.bytes(from, self.write_idx),
                            err,
                        };
                    }
                }
            }
        }

        let bytes = self.bytes(from, self.write_idx);
        if eof {
            IoResult::EOF(bytes)
        } else {
            IoResult::Ok(bytes)
        }
    }

    /// Write bytes from the ringbuffer into the writer. This performs up to two `write` operations
    /// on the writer. Returns an iterator over the bytes written and a status indicating the
    /// number of bytes written as well as whether EOF was reached.
    ///
    /// IO errors `ErrorKind::WouldBlock` and `ErrorKind::Interrupted` are ignored. Other errors
    /// are returned.
    pub fn write<'b>(&'b mut self, write: &mut impl std::io::Write) -> IoResult<'b> {
        let mut bytes_written = 0;
        let mut eof = false;
        let from = self.read_idx;

        // in case enough data is available we may wrap around the ringbuffer immediately
        for _ in 0..2 {
            let filled = {
                let range = self.filled();
                &self.buf[range]
            };
            if filled.is_empty() {
                break;
            }

            match write.write(filled) {
                Ok(n) => {
                    bytes_written += n;
                    self.read_idx += n;
                    if self.read_idx == C {
                        self.read_idx = 0;
                    }

                    if n == 0 {
                        eof = true;
                        break;
                    }

                    if bytes_written < filled.len() {
                        break;
                    }
                }
                Err(err) => {
                    // ignore some errors
                    if matches!(err.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) {
                        break;
                    } else {
                        return IoResult::Err {
                            bytes: self.bytes(from, self.read_idx),
                            err,
                        };
                    }
                }
            }
        }

        let bytes = self.bytes(from, self.read_idx);
        if eof {
            IoResult::EOF(bytes)
        } else {
            IoResult::Ok(bytes)
        }
    }
}
