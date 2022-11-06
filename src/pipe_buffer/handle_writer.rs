use age::stream::StreamWriter;
use ringbuf::Producer;
use std::{io, mem};
use std::io::{Write};
use core::option::Option;

type UnencryptType = Producer<u8>;
type EncryptType = StreamWriter<UnencryptType>;

pub enum SelectEncryption {
    Unencrypt(UnencryptType),
    Encrypt(EncryptType),
    TempNone,
}

pub struct HandleWriter {
    inner: SelectEncryption,
}

impl HandleWriter {
    pub fn new(
        unencrypt: Option<UnencryptType>,
        encrypt: Option<EncryptType>,
    ) -> HandleWriter {
        if unencrypt.is_some() {
            HandleWriter {
                inner: SelectEncryption::Unencrypt(unencrypt.unwrap())
            }
        } else if encrypt.is_some() {
            HandleWriter {
                inner: SelectEncryption::Encrypt(encrypt.unwrap())
            }
        } else {
            panic!("Both the unencrypt and encrypt variable cannot be None")
        }
    }

    // Implemented an wrapper, because the `StreamWriter` in `self.encrypt`
    // requires this method call to write out the leftover data in its inner caches
    pub fn finish(&mut self) -> Option<io::Error> {
        let select_enc = mem::replace(&mut self.inner, SelectEncryption::TempNone);
        match select_enc {
            SelectEncryption::Unencrypt(v) => {
                self.inner = SelectEncryption::Unencrypt(v);
                None
            },
            SelectEncryption::Encrypt(enc) => {
                let result = enc.finish();
                match result {
                    Ok(unencrypt) => {
                        // The writer `UnencryptType` there was wrapped in the
                        // `StreamWriter` type and is returned up-on executing the `finish` method.
                        // It is when saved back into the `self.inner` variable as
                        // the `Unencrypt` enum.
                        self.inner = SelectEncryption::Unencrypt(unencrypt);
                        None
                    },
                    Err(e) => {
                        Some(e)
                    },
                }
            }
            SelectEncryption::TempNone => {
                panic!("The `SelectEncryption::TempNone` is not allowed to be used")
            }
        }
    }
}

impl Write for HandleWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.inner {
            SelectEncryption::Unencrypt(v) => {
                v.write(buf)
            },
            SelectEncryption::Encrypt(v) => {
                v.write(buf)
            }
            SelectEncryption::TempNone => {
                panic!("The `SelectEncryption::TempNone` is not allowed to be used")
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.inner {
            SelectEncryption::Unencrypt(v) => {
                v.flush()
            },
            SelectEncryption::Encrypt(v) => {
                v.flush()
            }
            SelectEncryption::TempNone => {
                panic!("The `SelectEncryption::TempNone` is not allowed to be used")
            }
        }
    }
}

