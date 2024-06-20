use std::io::ErrorKind;

/// A ringbuffer that can read from std::io::Read and write to std::io::Write.
pub struct Ringbuffer<const C: usize> {
    buf: [u8; C],
    write_idx: usize,
    read_idx: usize,
}

pub enum IoResult {
    Ok(usize),
    EOF(usize),
}

struct Iter<'b> {
    buf: &'b [u8],
    to: usize,
    cur: usize,
}

impl<'b> Iterator for Iter<'b> {
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
}

impl<const C: usize> Ringbuffer<C> {
    pub fn new() -> Self {
        Ringbuffer {
            buf: [0; C],
            write_idx: 0,
            read_idx: 0,
        }
    }

    fn unfilled(&self) -> std::ops::Range<usize> {
        let start = self.write_idx;
        let end = if self.read_idx <= self.write_idx {
            C
        } else {
            self.read_idx
        };

        start..end
    }

    fn filled(&self) -> std::ops::Range<usize> {
        let start = self.read_idx;
        let end = if self.read_idx <= self.write_idx {
            self.write_idx
        } else {
            C
        };

        start..end
    }

    /// Pull bytes from the source into the ringbuffer. This performs up to two `read` operations
    /// on the source. Returns an iterator over the bytes read and a status indicating the number
    /// of bytes read as well as whether EOF was reached.
    ///
    /// IO errors `ErrorKind::WouldBlock` and `ErrorKind::Interrupted` are ignored. Other errors
    /// are returned.
    pub fn read<'b>(
        &'b mut self,
        read: &mut impl std::io::Read,
    ) -> std::io::Result<(impl Iterator<Item = u8> + 'b, IoResult)> {
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
                        return Err(err);
                    }
                }
            }
        }

        let res = if eof {
            IoResult::EOF(bytes_read)
        } else {
            IoResult::Ok(bytes_read)
        };

        let iter = Iter {
            buf: &self.buf,
            to: self.write_idx,
            cur: from,
        };

        Ok((iter, res))
    }

    /// Write bytes from the ringbuffer into the writer. This performs up to two `write` operations
    /// on the writer. Returns an iterator over the bytes written and a status indicating the
    /// number of bytes written as well as whether EOF was reached.
    ///
    /// IO errors `ErrorKind::WouldBlock` and `ErrorKind::Interrupted` are ignored. Other errors
    /// are returned.
    pub fn write<'b>(
        &'b mut self,
        write: &mut impl std::io::Write,
    ) -> std::io::Result<(impl Iterator<Item = u8> + 'b, IoResult)> {
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
                        return Err(err);
                    }
                }
            }
        }

        let res = if eof {
            IoResult::EOF(bytes_written)
        } else {
            IoResult::Ok(bytes_written)
        };

        let iter = Iter {
            buf: &self.buf,
            to: self.read_idx,
            cur: from,
        };

        Ok((iter, res))
    }
}
