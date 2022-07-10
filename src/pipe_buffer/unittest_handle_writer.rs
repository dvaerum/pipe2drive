
#[cfg(test)]
mod tests_async {
    use age::{Encryptor, x25519};
    use tokio_util::compat::TokioAsyncWriteCompatExt;
    use futures::AsyncWriteExt;
    use futures::executor::block_on;
    use ringbuf::Producer;
    use crate::pipe_buffer::handle_writer::DoNotCloseAsyncWriter;

    use super::super::handle_writer::HandleWriterAsync;

    #[test]
    fn test_210_DoNotCloseAsyncWriter() {
        let w = vec![];

        let (receiver, mut writer) = DoNotCloseAsyncWriter::wrap_async_writer(w);

        let r = block_on(writer.write(&[20 as u8; 10]));
        assert_eq!(10, r.unwrap());

        let c = block_on(writer.close());
        assert_eq!(true, c.is_ok());

        eprintln!("tt2");

        let old_w = block_on(receiver);
        assert_eq!(true, old_w.is_ok());

        eprintln!("tt2");

        assert_eq!(&old_w.unwrap(), &[20, 20, 20, 20, 20, 20, 20, 20, 20, 20])
    }

    fn test_110_handle_writer() {
        block_on(async {
            let key = x25519::Identity::generate();
            let pubkey = key.to_public();
            let encryptor = Encryptor::with_recipients(
                vec![Box::new(pubkey)]);

            
            let (mut r, mut producer) = tokio_pipe::pipe().unwrap();
            let producer = producer.compat_write();

            let handler = HandleWriterAsync::new(
                None,
                Some(encryptor.wrap_async_output(producer).await
                        .unwrap_or_else(|err| {
                            unimplemented!("No code written to handle the error: {:?}", err)
                        }),
                ),
            );
        });
    }
}
