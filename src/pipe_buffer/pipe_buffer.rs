use age::x25519::Recipient;
use age::Encryptor;
use ringbuf::{SharedRb};
use std::{io, cmp};
use std::io::{Read, SeekFrom};
use std::io::{Seek};
use std::ptr;
use std::thread::sleep;
use std::time::Duration;

use super::HandleWriter;
use super::ConsumerArcSharedRbType;

pub struct PipeBuffer<R> {
    source_reader: R,
    upload_counter: usize,
    max_size: usize,
    eop: bool,
    eop_cache_size: usize,
    count_nulls: usize,
    ringbuffer_saved_space: usize,
    ringbuffer_consumer: ConsumerArcSharedRbType,
    ringbuffer_producer: HandleWriter,
}

// Sourced the value from the variable `CHUNK_SIZE` in the rage package
// https://github.com/str4d/rage/blob/d3aa905a61c64e5dbdfb788dda402672b93075f3/age/src/primitives/stream.rs#L22
const CHUNK_SIZE: usize = 64 * 1024;
pub (crate) const PIPE_BUFFER_RING_BUFFER_SIZE: usize = CHUNK_SIZE * 2;

impl<R: Read> PipeBuffer<R> {
    pub fn new(
        reader: R,
        file_size: usize,
        encrypt_public_key: Option<Recipient>,
        ring_buffer_size: usize,
    ) -> PipeBuffer<R>
    where
        R: Read,
    {
        if encrypt_public_key.is_none() && ring_buffer_size < 3 {
            panic!("The ring_buffer_size cannot be less when 3, without encryption")
        } else if encrypt_public_key.is_some() && ring_buffer_size < PIPE_BUFFER_RING_BUFFER_SIZE {
            panic!("The ring_buffer_size cannot be less when {}, with encryption", PIPE_BUFFER_RING_BUFFER_SIZE)
        }
        let ring_buffer = SharedRb::<u8, Vec<_>>::new(ring_buffer_size);
        let (producer, consumer) = ring_buffer.split();
        PipeBuffer {
            source_reader: reader,
            upload_counter: 0,
            max_size: file_size,
            eop: false,
            eop_cache_size: if encrypt_public_key.is_some() { CHUNK_SIZE } else { 1 },
            count_nulls: 0,
            ringbuffer_saved_space: if encrypt_public_key.is_some() { CHUNK_SIZE } else { 0 },
            ringbuffer_consumer: consumer,
            ringbuffer_producer: if encrypt_public_key.is_some() {
                HandleWriter::new(
                    None,
                    Some(
                        Encryptor::with_recipients(vec![Box::new(encrypt_public_key.unwrap())]).unwrap()
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
        !self.eop || self.ringbuffer_consumer.len() > 0
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

        // Values for debugging
        #[cfg(any(test, debug))]
        let mut _debug_streamer_reader_len: usize;
        #[cfg(any(test, debug))]
        let mut _debug_streamer_reader_remaining: usize;

        let buffer_len = buffer.len();

        // If you give a zero-size buf, because what is the point ;)
        if buffer_len == 0 {
            return Ok(0);
        }

        // Create tmp_buffer to read data into
        let mut tmp_buffer: Vec<u8> = if self.ringbuffer_consumer.len() > buffer_len {
            vec![0; 0]
        } else {
            vec![0; buffer_len]
        };

        // Start reading data from inner buffer
        let result = match self.source_reader.read(&mut tmp_buffer) {
            Ok(read_size) => {
                // Filling (the remain part of) the buffer with 0x00 if there is
                // no more data from the buffer (Stdin)
                if self.eop && self.ringbuffer_consumer.len() == 0 {
                    let mut buf_ptr = buffer.as_mut_ptr();
                    unsafe {
                        buf_ptr = buf_ptr.offset(read_size as isize);
                        ptr::write_bytes(buf_ptr, 0x00, buffer_len - read_size as usize);
                    }
                    self.count_nulls += buffer_len - read_size;

                    trace!("Buffer Size Returned: {}", buffer_len);
                    Ok(buffer_len)

                } else {
                    let mut writer_counter: usize = 0;
                    let mut reader_counter: usize = 0;

                    // Values for debugging
                    #[cfg(any(test, debug))]
                    {
                        _debug_streamer_reader_len = self.ringbuffer_consumer.len();
                        _debug_streamer_reader_remaining = self.ringbuffer_consumer.free_len();
                    }

                    // Check if the `reader_counter` and `writer_counter` is smaller when
                    // then `buffer_len`, because
                    // if `reader_counter` is less when the `buffer_len`, it
                    // means that there is still more space in the `buffer` (`buffer_len`)
                    // and that we should continue to fill it with data from the ring_buffer.
                    //
                    // If the `writer_counter` is less when `tmp_buffer.len()`, it means that
                    // there is still data in the tmp_buffer, there have not been written to
                    // the ring_buffer, so that needs to happen or the data in tmp_buffer
                    // will be lost.
                    //
                    // If the lenght (len) of the streamer_reader is 0, and the writer_counter
                    // is equal to the read_size, it means that there are no more data
                    // in the ring buffer, and that we have read all the data from tmp_buffer
                    // into the ring_buffer.
                    while (buffer_len > reader_counter || tmp_buffer.len() > writer_counter) &&
                          !(self.ringbuffer_consumer.len() == 0 && writer_counter == read_size) {

                        let end_of_buffer = cmp::min(
                            writer_counter + if self.ringbuffer_consumer.free_len() > self.ringbuffer_saved_space {
                                self.ringbuffer_consumer.free_len() - self.ringbuffer_saved_space } else { self.ringbuffer_consumer.free_len() },
                            read_size,
                        );
                        let sliced_tmp_buffer = &tmp_buffer[writer_counter..end_of_buffer];

                        // Write data to the inner buffer ring
                        match self.ringbuffer_producer.write(
                            sliced_tmp_buffer) {
                            Ok(size) => {
                                writer_counter += size;
                                trace!("Wrote {} bytes to the ring buffer", size)
                            }
                            Err(err) => {
                                match err.kind() {
                                    _ => todo!("Need to implement error handling for ring buffer: {:?}", err),
                                }
                            }
                        }


                        // Values for debugging
                        #[cfg(any(test, debug))]
                        {
                            _debug_streamer_reader_len = self.ringbuffer_consumer.len();
                            _debug_streamer_reader_remaining = self.ringbuffer_consumer.free_len();
                        }

                        // Read data from the inner ring buffer
                        if self.ringbuffer_consumer.len() > 0 {
                            match self.ringbuffer_consumer.read(&mut buffer[reader_counter..buffer_len]) {
                                Ok(size) => {
                                    reader_counter += size;
                                    trace!("Read {} bytes from the ring buffer", size)
                                }
                                Err(e) => {
                                    match e.kind() {
                                        io::ErrorKind::WouldBlock => return Ok(0),
                                        _ => todo!("Need to implement error handling for ring buffer: {:?}", e)
                                    }
                                }
                            }
                        } else {
                            // todo!("what now");
                            sleep(Duration::from_secs(1))
                        }

                        // Values for debugging
                        #[cfg(any(test, debug))]
                        {
                            _debug_streamer_reader_len = self.ringbuffer_consumer.len();
                            _debug_streamer_reader_remaining = self.ringbuffer_consumer.free_len();
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

        // Values for debugging
        #[cfg(any(test, debug))]
        {
            _debug_streamer_reader_len = self.ringbuffer_consumer.len();
            _debug_streamer_reader_remaining = self.ringbuffer_consumer.free_len();
        }

        // Checking if there is more date and safe it for later
        if self.is_there_more() && self.ringbuffer_consumer.len() == 0 {
            let mut streamer_reader_len = self.ringbuffer_consumer.len();

            // If streamer_reader_len is 0, we are going to check if there are
            // more data in the inner buffer

            let mut _loop_count: usize = 0;
            let mut _count_writes: usize = 0;

            let mut _eop_cache: Vec<u8> = vec![0; self.eop_cache_size];

            while streamer_reader_len == 0 {
                let _eop_cache_result = match self.source_reader.read(&mut _eop_cache) {
                    Ok(_eop_cache_len) => {
                        // If we cannot even read a single byte, that means that the
                        // inner buffer is not getting anymore data (aka. consider closed)
                        if _eop_cache_len == 0 && self.is_there_more() {
                            self.eop = true;
                            match self.ringbuffer_producer.finish() {
                                Some(e) => {
                                    todo!("Need to implement error handling for finish: {:?}", e);
                                },
                                None => {
                                    self.eop_cache_size = 1;
                                },
                            }
                        } else if _eop_cache_len == 0 && self.eop {
                            {
                                break;
                            }
                        } else if _eop_cache_len >= 1 {
                            match self.ringbuffer_producer.write(&_eop_cache[0.._eop_cache_len]) {
                                Ok(write_size) => {
                                        if _eop_cache_len != write_size {
                                            panic!("Data was lost, _eop_cache_len and write_size are not equal - _eop_cache_len: {} - write_size: {}",
                                                   _eop_cache_len, write_size);
                                        }
                                    },
                                Err(e) => {
                                    todo!("Need to implement error handling for _eop_cache: {:?}", e)
                                }
                            }
                            _loop_count += 1;

                        // This should never be possible
                        } else {
                            unimplemented!(
                                "The _eop_cache_len value most only be 0 or 1+ not {}",
                                _eop_cache_len
                            );
                        }
                    }
                    Err(e) => return Err(e),
                };

                streamer_reader_len = self.ringbuffer_consumer.len();
            }
        }

        // Values for debugging
        #[cfg(any(test, debug))]
        {
            _debug_streamer_reader_len = self.ringbuffer_consumer.len();
            _debug_streamer_reader_remaining = self.ringbuffer_consumer.free_len();
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


