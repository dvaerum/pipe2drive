use age::stream::StreamWriter;
use ringbuf::Producer;
use std::io;
use std::io::Write;

type UnencryptType = Producer<u8>;
type EncryptType = StreamWriter<UnencryptType>;

pub struct HandleWriter {
    unencrypt: Option<UnencryptType>,
    encrypt: Option<EncryptType>,
}

impl HandleWriter {
    pub fn new(
        unencrypt: Option<UnencryptType>,
        encrypt: Option<EncryptType>,
    ) -> HandleWriter {
        HandleWriter {
            unencrypt: unencrypt,
            encrypt: encrypt,
        }
    }

    // Implemented an wrapper, because the `StreamWriter` in `self.encrypt`
    // requires this method call to write out the leftover data in its inner caches
    pub fn finish(&mut self) -> Option<io::Error> {
        if self.unencrypt.is_some() {
            None

        } else if self.encrypt.is_some() {
            let enc: EncryptType = ::std::mem::replace(
                &mut self.encrypt, None).unwrap();

            match enc.finish() {
                Ok(_r) => {
                    // The writer `UnencryptType` there was wrapped in the 
                    // `StreamWriter` type is moved from `self.encrypt` to
                    // `self.unencrypt` because the encryption stream is closed
                    self.unencrypt = Some(_r);
                    None
                },
                Err(e) => {
                    Some(e)
                },
            }

        } else {
            unimplemented!("Both self.unencrypt & self.encrypt cannot be None")
        }
    }
}


impl Write for HandleWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(unencrypt) = &mut self.unencrypt {
            unencrypt.write(buf)
        } else if let Some(encrypt) = &mut self.encrypt {
            encrypt.write(buf)
        } else {
            unimplemented!("Both self.unencrypt & self.encrypt cannot be None")
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(unencrypt) = &mut self.unencrypt {
            unencrypt.flush()
        } else if let Some(encrypt) = &mut self.encrypt {
            encrypt.flush()
        } else {
            unimplemented!("Both self.unencrypt & self.encrypt cannot be None")
        }
    }
}
