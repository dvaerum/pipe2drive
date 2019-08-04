use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::ptr;

pub struct PipeBuffer<R> {
    inner: R,
    upload_counter: usize,
    max_size: usize,
    eop: bool,
    count_nulls: usize,
}

impl<R: Read> PipeBuffer<R> {
    pub fn new(buf: R, size: usize) -> PipeBuffer<R> where R: Read {
        PipeBuffer {
            inner: buf,
            upload_counter: 0,
            max_size: size,
            eop: false,
            count_nulls: 0,
        }
    }

    pub fn is_there_more(&self) -> bool {
        !self.eop
    }

    pub fn nulls(&self) -> u64 {
        self.count_nulls as u64
    }
}

impl<R: Read> Read for PipeBuffer<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.upload_counter += buf.len();
        if self.upload_counter < self.max_size {
            trace!("Total size: {} - Upload counter: {}", self.max_size, self.upload_counter)
        } else {
            trace!("Everything is uploaded - Total size: {} - Upload counter: {}", self.max_size, self.upload_counter)
        }

        match self.inner.read(buf) {
            Ok(r) => {
                if !self.eop && r == 0 {
                    self.eop = true;
                }

                // Filling (the remain part of) the buffer with 0x00 if there is
                // no more data from Stdin
                if self.eop {
                    let mut buf_ptr = buf.as_mut_ptr();
                    unsafe {
                        buf_ptr = buf_ptr.offset(r as isize);
                        ptr::write_bytes(buf_ptr, 0x00, buf.len() - 1 - r as usize);
                    }
                    self.count_nulls += buf.len() - r;

                    Ok(buf.len())
                } else {
                    Ok(r)
                }
            }
            Err(e) => Err(e)
        }
    }
}

impl<R: Read> Seek for PipeBuffer<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(0) => Ok(0),
            SeekFrom::Start(_) => unimplemented!(),
            SeekFrom::Current(_) => unimplemented!(),
            SeekFrom::End(0) => Ok(self.max_size as u64),
            SeekFrom::End(_) => unimplemented!(),
        }
    }
}