use age::stream::StreamWriter;
use tokio_util::compat::Compat;
use futures::AsyncWrite;
use pin_project::pin_project;
use ringbuf::Producer;
use tokio_pipe::PipeWrite;
use std::{io, mem};
use std::io::{Error, ErrorKind, Write};
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use yup_oauth2::read_service_account_key;
use core::option::Option;
use std::fmt::Debug;
use futures::channel::oneshot;

type UnencryptType = Producer<u8>;
type EncryptType = StreamWriter<UnencryptType>;


// #[pin_project(project = DoNotCloseAsyncWriterProj)]
// pub struct DoNotCloseAsyncWriter<W>{
//     #[pin]
//     inner: Option<W>,
//     inner_return_channel: oneshot::Sender<W>,
// }

pub struct DoNotCloseAsyncWriter<W>{
    inner: Pin<Box<Option<W>>>,
    inner_return_channel: oneshot::Sender<W>,
}

impl <W: AsyncWrite> DoNotCloseAsyncWriter<W> {
    pub fn wrap_async_writer(writer: W) -> (oneshot::Receiver<W>, Self) {
        let (sender, receiver) = oneshot::channel::<W>();
        let instance = Self {
            inner: Box::pin(Some(writer)),
            inner_return_channel: sender,
        };
        (receiver, instance)
    }
}

impl<W: AsyncWrite + Debug> AsyncWrite for DoNotCloseAsyncWriter<W> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        if self.inner.is_none() {
            return Poll::Ready(Err(Error::new(ErrorKind::BrokenPipe,
                                              "writer already closed")))
        }

        self.inner.as_pin_mut().

        // self.project().inner.as_pin_mut().unwrap().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if self.inner.is_none() {
            return Poll::Ready(Err(Error::new(ErrorKind::BrokenPipe,
                                              "writer already closed")))
        }

        self.inner.as_pin_mut().unwrap().poll_flush(cx)
        // self.project().inner.as_pin_mut().unwrap().poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let address1 = &self.inner as *const _;

        eprint!("\n");
        eprintln!("address1: {:?}", address1);

        let address2 = &self.inner.as_pin_mut() as *const _;
        //let result = _address2.as_pin_ref().unwrap().poll_flush(cx);

        eprintln!("address2: {:?}", address2);

        // let test = mem::replace(self.inner.as_pin_mut(), None);

        //let result = inner.as_mut().unwrap().poll_flush(cx);

        eprintln!("test1");
        Poll::Ready(Ok(()))
        // match result {
        //     Poll::Pending => Poll::Pending,
        //     Poll::Ready(Ok(())) => {
        //         eprintln!("test2");
        //         //let inner = mem::replace(&mut self.inner, None);
        //         //self.inner_return_channel.send(inner.unwrap());
        //         Poll::Ready(Ok(()))
        //     },
        //     Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        // }
    }
}


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


type UnencryptTypeAsync = Compat<PipeWrite>;
type EncryptTypeAsync = StreamWriter<UnencryptTypeAsync>;
#[pin_project(project = HandleWriterAsyncProj)]
pub struct HandleWriterAsync {
    #[pin]
    unencrypt: Option<UnencryptTypeAsync>,
    #[pin]
    encrypt: Option<EncryptTypeAsync>,
}

impl HandleWriterAsync {
    pub fn new(
        unencrypt: Option<UnencryptTypeAsync>,
        encrypt: Option<EncryptTypeAsync>,
    ) -> HandleWriterAsync {
        HandleWriterAsync {
            unencrypt: unencrypt,
            encrypt: encrypt,
        }
    }
}

impl AsyncWrite for HandleWriterAsync {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8]) -> Poll<io::Result<usize>> {
            if self.unencrypt.is_some() {
                return self.project().unencrypt.as_pin_mut().unwrap().poll_write(cx, buf)
            } else if self.encrypt.is_some() {
                return self.project().encrypt.as_pin_mut().unwrap().poll_write(cx, buf)
            }

            unimplemented!("HandleWriterAsync.poll_write")
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            if self.unencrypt.is_some() {
                return self.project().unencrypt.as_pin_mut().unwrap().poll_flush(cx)
            } else if self.encrypt.is_some() {
                return self.project().encrypt.as_pin_mut().unwrap().poll_flush(cx)
            }

            unimplemented!("HandleWriterAsync.poll_write")
    }

    fn poll_close(
        self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        todo!("fix close");
        // if self.unencrypt.is_some() {
        //     return self.project().unencrypt.as_pin_mut().unwrap().poll_flush(cx)
        // } else if self.encrypt.is_some() {
        //     let this = self.project().encrypt.as_pin_mut().unwrap();
        //     ready!(this.poll_flush(cx));
        //
        //     if !self.stream.is_complete() {
        //         // Finish the stream.
        //         let this = self.as_mut().project();
        //         *this.encrypted_chunk = Some(EncryptedChunk {
        //             bytes: this.stream.encrypt_chunk(&this.chunk, true)?,
        //             offset: 0,
        //         });
        //     }
        //
        //     // Flush the final chunk (if we didn't in the first call).
        //     ready!(self.as_mut().poll_flush_chunk(cx))?;
        //
        //
        //     return self.project().encrypt.as_pin_mut().unwrap().poll_flush(cx)
        // }
        // unimplemented!("HandleWriterAsync.poll_write")
    }
}

