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
    use crate::misc::tests::{verify_test_buffer_data, verify_test_buffer_data_and_count_nulls};

    #[test]
    fn test_10_test_buffer() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(6);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(6, count);
        assert_eq!("012345".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 6));

        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(10);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(10, count);
        assert_eq!("0123456789".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 10));

        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(13);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(13, count);
        assert_eq!("0123456789012".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 13));

        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(20);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(20, count);
        assert_eq!("01234567890123456789".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 20));

        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(27);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(27, count);
        assert_eq!("012345678901234567890123456".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 27));
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
        assert!(verify_test_buffer_data(&buffer, 0, 23));


        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(7, count);
        assert_eq!(30, pipe_stream.get_upload_counter());
        assert_eq!("3456789".as_bytes(), &buffer[0..count]);
        assert!(verify_test_buffer_data(&buffer, 3, 7));

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
        const BUFFER_SIZE: usize = 1024 * 1024 * 1;
        let mut count: usize;
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let mut pipe_stream = PipeBuffer::new(TestBuffer::new(1024 * 1024 * 31), 1024 * 1024 * 32, None, 1024 * 13);

        for i in 0..30 {
            count = pipe_stream.read(&mut buffer).unwrap();
            assert_eq!(1024 * 1024 * 1, count);
            assert_eq!(1024 * 1024 * (1 + i), pipe_stream.get_upload_counter());
            assert!(verify_test_buffer_data(&buffer, (i * 6 % 10) as u8, 1024 * 1024 * 1));
        }

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(1024 * 1024 * 1, count);
        assert_eq!(1024 * 1024 * 31, pipe_stream.get_upload_counter());
        assert!(verify_test_buffer_data(&buffer, 0, 1024 * 1024 * 1));

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(1024 * 1024 * 1, count);
        assert_eq!(1024 * 1024 * 31, pipe_stream.get_upload_counter());
        assert!(verify_test_buffer_data_and_count_nulls(&buffer, 0, 0, true));

        count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(1024 * 1024 * 1, count);
        assert_eq!(1024 * 1024 * 31, pipe_stream.get_upload_counter());
        assert!(verify_test_buffer_data_and_count_nulls(&buffer, 0, 0, true));
    }

    #[test]
    fn test_22_pipe_buffer() {
        const TOTAL_SIZE: usize = 1024 * 1024 * 10;

        let mut pipe_stream = PipeBuffer::new(
            TestBuffer::new(TOTAL_SIZE),
            TOTAL_SIZE,
            None,
            1024 * 1024 * 4
        );

        let mut buffer: Vec<u8>;

        buffer = vec![0; 32];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 0, 32));

        buffer = vec![0; 32];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 2, 32));

        buffer = vec![0; 64];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 4, 64));

        buffer = vec![0; 128];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 8, 128));

        buffer = vec![0; 256];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 6, 256));

        buffer = vec![0; 512];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 2, 512));

        buffer = vec![0; 1024];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 4, 1024));

        buffer = vec![0; 2048];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 8, 2048));

        buffer = vec![0; 4096];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 6, 4096));

        buffer = vec![0; 8192];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 2, 8192));

        buffer = vec![0; 16384];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 4, 16384));

        buffer = vec![0; 32768];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 8, 32768));

        buffer = vec![0; 65536];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 6, 65536));

        buffer = vec![0; 131072];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 2, 131072));

        buffer = vec![0; 262144];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 4, 262144));

        buffer = vec![0; 524288];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 8, 524288));

        buffer = vec![0; 1048576];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 6, 1048576));

        buffer = vec![0; 2097152];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 2, 2097152));

        buffer = vec![0; 4194304];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 4, 4194304));

        buffer = vec![0; 32];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 8, 32));

        buffer = vec![0; 32];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 0, 32));

        buffer = vec![0; 64];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 2, 64));

        buffer = vec![0; 128];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 6, 128));

        buffer = vec![0; 256];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 4, 256));

        buffer = vec![0; 512];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 0, 512));

        buffer = vec![0; 1024];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 2, 1024));

        buffer = vec![0; 2048];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 6, 2048));

        buffer = vec![0; 4096];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 4, 4096));

        buffer = vec![0; 8192];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 0, 8192));

        buffer = vec![0; 16384];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 2, 16384));

        buffer = vec![0; 32768];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 6, 32768));

        buffer = vec![0; 65536];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 4, 65536));

        buffer = vec![0; 131072];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 0, 131072));

        buffer = vec![0; 262144];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 2, 262144));

        buffer = vec![0; 524288];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 6, 524288));

        buffer = vec![0; 1048575];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data(&buffer, 4, 1048575));

        buffer = vec![0; 11];
        pipe_stream.read(&mut buffer).unwrap();
        assert!(verify_test_buffer_data_and_count_nulls(&buffer, 9, 1, true));
        assert_eq!([57,0,0,0,0,0,0,0,0,0,0], &buffer[0..11]);

        buffer = vec![0; 11];
        pipe_stream.read(&mut buffer).unwrap();
        assert_eq!([0,0,0,0,0,0,0,0,0,0,0], &buffer[0..11]);
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
        assert_eq!(BUFFER_SIZE, _count);
        full_buffer.extend_from_slice(&buffer[.._count]);

        _count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(BUFFER_SIZE, _count);
        full_buffer.extend_from_slice(&buffer[.._count]);

        _count = pipe_stream.read(&mut buffer).unwrap();
        assert_eq!(BUFFER_SIZE, _count);
        full_buffer.extend_from_slice(&buffer[.._count]);

        // Loop and read the rest. The actual size is unknown,
        // but in should be less then 500 bytes
        while pipe_stream.is_there_more() {
            _count = pipe_stream.read(&mut buffer).unwrap();
            full_buffer.extend_from_slice(&buffer[.._count]);
        }

        // Should succeeded at decrypting data
        let decryptor = match age::Decryptor::new(&full_buffer[..pipe_stream.get_upload_counter()]).unwrap() {
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
            decrypted
        );

        // Should fail at decrypting data, because of a missing byte,
        // gives a 'decryption error' error msg
        let decryptor = match age::Decryptor::new(&full_buffer[..pipe_stream.get_upload_counter() - 1]).unwrap() {
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
        full_buffer.push(0);
        let decryptor = match age::Decryptor::new(&full_buffer[..pipe_stream.get_upload_counter() + 1]).unwrap() {
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