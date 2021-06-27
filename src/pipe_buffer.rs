use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::ptr;
// use crate::crypto::load_public_key;
// use age::Recipient;

pub struct TestBuffer {
    counter: usize,
    size: usize,
}

impl TestBuffer {
    pub fn new(size: usize) -> TestBuffer {
        TestBuffer {
            counter: 0,
            size: size,
        }
    }
}

impl Read for TestBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut new_counter: usize = self.counter + buf.len();
        if new_counter > self.size {
            new_counter = self.size;
        }

        let mut tmp_counter: usize = 0;
        for current_count in self.counter..new_counter {
            let i = current_count - self.counter;
            // 48 is the DEC value for the CHAR '0',
            // so it is used to convert to str/char
            buf[i] = (current_count % 10 + 48) as u8;

            tmp_counter += 1;
        }
        self.counter = new_counter;

        return Ok(tmp_counter);
    }
}

pub struct PipeBuffer<R> {
    inner: R,
    upload_counter: usize,
    max_size: usize,
    eop: bool,
    eop_cache: Option<[u8; 1]>,
    count_nulls: usize,
    //pub encrypt_public_key: Box<Recipient>,
}

impl<R: Read> PipeBuffer<R> {
    pub fn new(buf: R, size: usize) -> PipeBuffer<R>
    where
        R: Read,
    {
        PipeBuffer {
            inner: buf,
            upload_counter: 0,
            max_size: size,
            eop: false,
            eop_cache: None,
            count_nulls: 0,
            //encrypt_public_key: Box::new(load_public_key()),
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
        // If you give a zero-size buf, because what is the point ;)
        if buf.len() == 0 {
            return Ok(0);
        }

        self.upload_counter += buf.len();
        if self.upload_counter < self.max_size {
            trace!(
                "Total size: {} - Upload counter: {}",
                self.max_size,
                self.upload_counter
            )
        } else {
            trace!(
                "Everything is uploaded - \
                    Total size: {} - Upload counter: {}",
                self.max_size,
                self.upload_counter
            )
        }

        let eop_cache_is_some = self.eop_cache.is_some();
        if eop_cache_is_some {
            buf[0] = self.eop_cache.unwrap()[0];
            self.eop_cache = None;
            trace!("eop_cache: is_some()");
        }

        let result = match self
            .inner
            .read(&mut buf[(if eop_cache_is_some { 1 } else { 0 })..])
        {
            Ok(r) => {
                if !self.eop && r == 0 {
                    self.eop = true;
                }
                // If there was an `eop_cache`
                let r = r + if eop_cache_is_some { 1 } else { 0 };

                // Filling (the remain part of) the buffer with 0x00 if there is
                // no more data from the buffer (Stdin)
                if self.eop {
                    let mut buf_ptr = buf.as_mut_ptr();
                    unsafe {
                        buf_ptr = buf_ptr.offset(r as isize);
                        ptr::write_bytes(buf_ptr, 0x00, buf.len() - r as usize);
                    }
                    self.count_nulls += buf.len() - r;

                    Ok(buf.len())
                } else {
                    Ok(r)
                }
            }
            Err(e) => Err(e),
        };

        // Checking if there is more date and safe it for later
        if !self.eop {
            let mut _eop_cache: [u8; 1] = [0];
            let _eop_cache_result = match self.inner.read(&mut _eop_cache) {
                Ok(_eop_cache_len) => {
                    if _eop_cache_len == 0 {
                        self.eop = true;
                    } else if _eop_cache_len == 1 {
                        self.eop_cache = Some(_eop_cache);
                    } else {
                        unimplemented!(
                            "The _eop_cache_len value most only be 0 or 1 not {}",
                            _eop_cache_len
                        );
                    }
                }
                Err(e) => return Err(e),
            };
        }

        return result;
    }
}

impl<R: Read> Seek for PipeBuffer<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(_) => Ok(0),
            SeekFrom::End(_) => Ok(self.max_size as u64),
            SeekFrom::Current(_) => Ok(self.upload_counter as u64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PipeBuffer, TestBuffer};
    use std::io::Read;

    #[test]
    fn test_10_test_buffer() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(6);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(6, count);
        assert_eq!("012345".as_bytes(), buffer);

        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(10);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(10, count);
        assert_eq!("0123456789".as_bytes(), buffer);

        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(13);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(13, count);
        assert_eq!("0123456789012".as_bytes(), buffer);

        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(20);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(20, count);
        assert_eq!("01234567890123456789".as_bytes(), buffer);

        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(27);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(27, count);
        assert_eq!("012345678901234567890123456".as_bytes(), buffer);
    }

    #[test]
    fn test_20_pipe_buffer() {
        const BUFFER_SIZE: usize = 23;
        let mut count: usize;
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let mut pipe_stream = PipeBuffer::new(TestBuffer::new(30), 23);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!("01234567890123456789012".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(7, count);
        assert_eq!("3456789".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            &buffer[0..count]
        );

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!([0; BUFFER_SIZE], &buffer[0..count]);
    }
}
