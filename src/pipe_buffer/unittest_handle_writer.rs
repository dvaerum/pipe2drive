
// #[cfg(test)]
// mod tests_async {
//     use age::{Encryptor, x25519};
//     use async_compat::CompatExt;
//     use tokio_util::compat::TokioAsyncWriteCompatExt;
//     use futures::{AsyncWriteExt, TryFutureExt};
//     use tokio::runtime::Runtime;
//     use do_not_close_async_writer::DoNotCloseAsyncWriter;
//
//     use super::super::handle_writer::HandleWriterAsync;
//
//     #[test]
//     fn test_110_handle_writer() {
//         let rt = Runtime::new().unwrap();
//         let rt = rt.handle();
//
//         let key = x25519::Identity::generate();
//         let pubkey = key.to_public();
//         let encryptor = Encryptor::with_recipients(
//             vec![Box::new(pubkey)]);
//
//
//         let (mut r, mut producer) = tokio_pipe::pipe().unwrap();
//         let producer = producer;
//
//         let mut handler = HandleWriterAsync::new(
//             None,
//             Some(rt.block_on(encryptor.wrap_async_output(producer.compat()).compat())
//                     .unwrap_or_else(|err| {
//                         unimplemented!("No code written to handle the error: {:?}", err)
//                     }),
//             ),
//         );
//
//         let result = rt.block_on(
//             handler.write(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
//         ).unwrap();
//     }
// }
