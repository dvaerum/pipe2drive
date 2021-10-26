use age::x25519::Recipient;
use age::Encryptor;
use ringbuf::{Consumer, RingBuffer};
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::ptr;

use super::HandleWriter;

pub struct PipeBuffer<R> {
    inner: R,
    upload_counter: usize,
    max_size: usize,
    eop: bool,
    count_nulls: usize,
    streamer_reader: Consumer<u8>,
    streamer_writer: HandleWriter,
}

impl<R: Read> PipeBuffer<R> {
    pub fn new(
        buf: R,
        file_size: usize,
        encrypt_public_key: Option<Recipient>,
        ring_buffer_size: usize,
    ) -> PipeBuffer<R>
    where
        R: Read,
    {
        if encrypt_public_key.is_none() && ring_buffer_size < 3 {
            panic!("The ring_buffer_size cannot be less when 3, without encryption")
        } else if encrypt_public_key.is_some() && ring_buffer_size < 500 {
            panic!("The ring_buffer_size cannot be less when 500, with encryption")
        }

        let ring_buffer = RingBuffer::<u8>::new(ring_buffer_size);
        let (producer, consumer) = ring_buffer.split();
        PipeBuffer {
            inner: buf,
            upload_counter: 0,
            max_size: file_size,
            eop: false,
            count_nulls: 0,
            streamer_reader: consumer,
            streamer_writer: if encrypt_public_key.is_some() {
                HandleWriter::new(
                    None,
                    Some(
                        Encryptor::with_recipients(vec![Box::new(encrypt_public_key.unwrap())])
                            .wrap_output(producer)
                            .unwrap_or_else(|err| {
                                unimplemented!("No code written to handle the error: {:?}", err)
                            }),
                    ),
                )
            } else {
                HandleWriter::new(
                    Some(producer), 
                    None)
            },
        }
    }

    // Return true if there is more data in the inner buffer
    pub fn is_there_more(&self) -> bool {
        !self.eop
    }

    // Return the amount of data there have been reading from
    // the inner buffer
    #[allow(dead_code)]
    pub fn get_upload_counter(&self) -> usize {
        return self.upload_counter
    }

    // Return the amount of filler nulls there have been counted
    pub fn nulls(&self) -> u64 {
        self.count_nulls as u64
    }
}

impl<R: Read> Read for PipeBuffer<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        use io::Write;

        let buffer_len = buffer.len();

        // If you give a zero-size buf, because what is the point ;)
        if buffer_len == 0 {
            return Ok(0);
        }

        // Create tmp_buffer to read data into
        let mut tmp_buffer: Vec<u8> = if self.streamer_reader.len() > buffer_len {
            vec![0; 0]
        } else {
            vec![0; buffer_len]
        };

        // Start reading data from inner buffer
        let result = match self
            .inner
            .read(&mut tmp_buffer)
        {
            Ok(r) => {
                // Filling (the remain part of) the buffer with 0x00 if there is
                // no more data from the buffer (Stdin)
                if self.eop {
                    let mut buf_ptr = buffer.as_mut_ptr();
                    unsafe {
                        buf_ptr = buf_ptr.offset(r as isize);
                        ptr::write_bytes(buf_ptr, 0x00, buffer_len - r as usize);
                    }
                    self.count_nulls += buffer_len - r;

                    trace!("Buffer Size Returned: {}", buffer_len);
                    Ok(buffer_len)

                } else {
                    let mut writer_counter: usize = 0;
                    let mut reader_counter: usize = 0;

                    // Values for debugging
                    #[cfg(debug)]
                    {
                        let _debug_streamer_reader_len = self.streamer_reader.len();
                        let _debug_streamer_reader_remaning = self.streamer_reader.remaining();
                    }

                    while buffer_len > reader_counter && 
                          !(self.streamer_reader.len() == 0 && writer_counter == r) {

                        // Write data to the inner buffer ring
                        match self.streamer_writer.write(
                            &tmp_buffer[writer_counter..r]) {
                            Ok(size) => {
                                writer_counter += size;
                                trace!("Wrote {} bytes to the ring buffer", size)
                            }
                            Err(e) => {
                                todo!("Need to implement error handling for ring buffer: {:?}", e)
                            }
                        }

                        // Values for debugging
                        #[cfg(debug)]
                        {
                            let _debug_streamer_reader_len = self.streamer_reader.len();
                            let _debug_streamer_reader_remaning = self.streamer_reader.remaining();
                        }

                        // Read data from the inner ring buffer
                        match self.streamer_reader.read(&mut buffer[reader_counter..buffer_len]) {
                            Ok(size) => {
                                reader_counter += size;
                                trace!("Read {} bytes from the ring buffer", size)
                            }
                            Err(e) => {
                                 todo!("Need to implement error handling for ring buffer: {:?}", e)
                            }
                        }

                        // Values for debugging
                        #[cfg(debug)]
                        {
                            let _debug_streamer_reader_len = self.streamer_reader.len();
                            let _debug_streamer_reader_remaning = self.streamer_reader.remaining();
                        }
                    }

                    trace!("Buffer Size Returned: {}", reader_counter);
                    self.upload_counter += reader_counter;
                    Ok(reader_counter)
                }
            }
            Err(e) => Err(e),
        };

        // Log the progess to trace!
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

        // Checking if there is more date and safe it for later
        if !self.eop && self.streamer_reader.len() == 0 {
            match self.streamer_writer.finish() {
                Some(e) => {
                    todo!("Need to implement error handling for finish: {:?}", e);
                },
                None => {
                    let streamer_reader_len = self.streamer_reader.len();
                    
                    // If streamer_reader_len is 0, we are going to check if there are
                    // more data in the inner buffer
                    if streamer_reader_len == 0 {
                        let mut _eop_cache: [u8; 1] = [0];
                        let _eop_cache_result = match self.inner.read(&mut _eop_cache) {
                            Ok(_eop_cache_len) => {
                                // If we cannot even read a single byte, that means that the
                                // inner buffer is not getting anymore data (aka. consider closed)
                                if _eop_cache_len == 0 {
                                    self.eop = true;

                                } else if _eop_cache_len == 1 {
                                    match self.streamer_writer.write(&_eop_cache) {
                                        Ok(_) => (),
                                        Err(e) => {
                                            todo!("Need to implement error handling for _eop_cache: {:?}", e)
                                        }
                                    }

                                // This should never be possible
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
                },
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


