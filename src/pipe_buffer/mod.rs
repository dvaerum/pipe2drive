mod handle_writer;
mod pipe_buffer;
mod test_buffer;

use handle_writer::HandleWriter;
pub use pipe_buffer::PipeBuffer;
pub use test_buffer::TestBuffer;

#[cfg(test)]
mod tests {
    use super::test_buffer::TestBuffer;
    use super::PipeBuffer;
    use std::io;
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

        let mut pipe_stream = PipeBuffer::new(TestBuffer::new(30), 23, None, 3);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!(23, pipe_stream.get_upload_counter());
        assert_eq!("01234567890123456789012".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(7, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!("3456789".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            &buffer[0..count]
        );

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!([0; BUFFER_SIZE], &buffer[0..count]);
    }

    #[test]
    fn test_21_pipe_buffer() {
        const BUFFER_SIZE: usize = 1024 * 1024 * 20;
        println!("BUFFER_SIZE: {}", BUFFER_SIZE);
        let mut count: usize;
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let mut pipe_stream = PipeBuffer::new(TestBuffer::new(30), 23, None, 3);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!(23, pipe_stream.get_upload_counter());
        assert_eq!("01234567890123456789012".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(7, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!("3456789".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            &buffer[0..count]
        );

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!([0; BUFFER_SIZE], &buffer[0..count]);
    }

    #[test]
    fn test_30_pipe_buffer() {
        const BUFFER_SIZE: usize = 23;
        let mut count: usize;
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let mut pipe_stream = PipeBuffer::new(TestBuffer::new(30), 23, None, 27);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!(23, pipe_stream.get_upload_counter());
        assert_eq!("01234567890123456789012".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(7, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!("3456789".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            &buffer[0..count]
        );

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(23, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!([0; BUFFER_SIZE], &buffer[0..count]);
    }

    #[test]
    fn test_40_pipe_buffer() {
        const BUFFER_SIZE: usize = 14;
        let mut count: usize;
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let mut pipe_stream = PipeBuffer::new(TestBuffer::new(30), 23, None, 27);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(14, count);
        assert_eq!(14, pipe_stream.get_upload_counter());
        assert_eq!("01234567890123".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(14, count);
        assert_eq!(28, pipe_stream.get_upload_counter());
        assert_eq!("45678901234567".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(2, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!("89".as_bytes(), &buffer[0..count]);

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(14, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            &buffer[0..count]
        );

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(14, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!([0; BUFFER_SIZE], &buffer[0..count]);
    }

    #[test]
    fn test_50_pipe_buffer_encryption() {
        use std::str::FromStr;
        const BUFFER_SIZE: usize = 50;
        let mut _count: usize;
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut full_buffer: Vec<u8> = Vec::new();
        let private_key = age::x25519::Identity::from_str(
            "AGE-SECRET-KEY-15RAENVRSHDVGQ6XZXPUWZK4235AVF6EXFQTS3WG8XMHW0RMSD4EQ492LZ5",
        )
        .unwrap();
        let public_key = private_key.to_public();

        let mut pipe_stream = PipeBuffer::new(TestBuffer::new(30), 500, Some(public_key), 500);

        _count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(50, _count);
        full_buffer.extend_from_slice(&buffer[.._count]);

        _count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(50, _count);
        full_buffer.extend_from_slice(&buffer[.._count]);

        _count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(50, _count);
        full_buffer.extend_from_slice(&buffer[.._count]);

        // Loop and read the rest. The actual size is unknown,
        // but in should be less then 500 bytes
        while full_buffer.len() < 500 {
            _count = pipe_stream.read(&mut buffer).unwrap();
            full_buffer.extend_from_slice(&buffer[.._count]);
        }

        // Should succeeded at decrypting data
        let decryptor =
            match age::Decryptor::new(&full_buffer[..pipe_stream.get_upload_counter()]).unwrap() {
                age::Decryptor::Recipients(d) => d,
                _ => unreachable!(),
            };

        let mut decrypted: Vec<u8> = Vec::new();
        let mut reader = decryptor
            .decrypt(::std::iter::once(&private_key as &dyn age::Identity))
            .unwrap();
        reader.read_to_end(&mut decrypted).unwrap();

        assert_eq!(
            "012345678901234567890123456789".as_bytes(),
            decrypted.as_slice()
        );

        // Should fail at decrypting data, because of a missing byte,
        // gives a 'decryption error' error msg
        let decryptor = match age::Decryptor::new(
            &full_buffer[..pipe_stream.get_upload_counter() - 1],
        )
        .unwrap()
        {
            age::Decryptor::Recipients(d) => d,
            _ => unreachable!(),
        };

        let mut decrypted: Vec<u8> = Vec::new();
        let mut reader = decryptor
            .decrypt(::std::iter::once(&private_key as &dyn age::Identity))
            .unwrap();
        let err = reader.read_to_end(&mut decrypted).unwrap_err();
        let custom_err = io::Error::new(io::ErrorKind::InvalidData, "decryption error");
        assert_eq!(format!("{:?}", custom_err), format!("{:?}", err));

        // Should fail at decrypting data, because of a extra byte,
        // gives a 'decryption error' error msg
        let decryptor = match age::Decryptor::new(
            &full_buffer[..pipe_stream.get_upload_counter() + 1],
        )
        .unwrap()
        {
            age::Decryptor::Recipients(d) => d,
            _ => unreachable!(),
        };

        let mut decrypted: Vec<u8> = Vec::new();
        let mut reader = decryptor
            .decrypt(::std::iter::once(&private_key as &dyn age::Identity))
            .unwrap();
        let err = reader.read_to_end(&mut decrypted).unwrap_err();
        let custom_err = io::Error::new(io::ErrorKind::InvalidData, "decryption error");
        assert_eq!(format!("{:?}", custom_err), format!("{:?}", err));
    }
}
