#[cfg(test)]
mod tests_async {
    use super::super::test_buffer::TestBuffer;
    use futures::{io::AsyncReadExt, executor::block_on};
    use crate::misc::tests::{verify_test_buffer_data};

    #[test]
    fn test_110_test_buffer() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(6);
        let count = block_on(test_stream.read_to_end(&mut buffer)).unwrap();
        assert_eq!(6, count);
        assert_eq!("012345".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 6));
    }

    #[test]
    fn test_120_test_buffer() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(10);
        let count = block_on(test_stream.read_to_end(&mut buffer)).unwrap();
        assert_eq!(10, count);
        assert_eq!("0123456789".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 10));
    }

    #[test]
    fn test_130_test_buffer() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(13);
        let count = block_on(test_stream.read_to_end(&mut buffer)).unwrap();
        assert_eq!(13, count);
        assert_eq!("0123456789012".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 13));
    }
    
    #[test]
    fn test_140_test_buffer() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(20);
        let count = block_on(test_stream.read_to_end(&mut buffer)).unwrap();
        assert_eq!(20, count);
        assert_eq!("01234567890123456789".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 20));
    }

    #[test]
    fn test_150_test_buffer() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(27);
        let count = block_on(test_stream.read_to_end(&mut buffer)).unwrap();
        assert_eq!(27, count);
        assert_eq!("012345678901234567890123456".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 27));
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_buffer::TestBuffer;
    use std::io::Read;
    use crate::misc::tests::{verify_test_buffer_data};

    #[test]
    fn test_110_test_buffer_read() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(6);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(6, count);
        assert_eq!("012345".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 6));
    }

    #[test]
    fn test_120_test_buffer_read() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(10);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(10, count);
        assert_eq!("0123456789".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 10));
    }

    #[test]
    fn test_130_test_buffer_read() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(13);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(13, count);
        assert_eq!("0123456789012".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 13));
    }

    #[test]
    fn test_140_test_buffer_read() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(20);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(20, count);
        assert_eq!("01234567890123456789".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 20));
    }

    #[test]
    fn test_150_test_buffer_read() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut test_stream = TestBuffer::new(27);
        let count = test_stream.read_to_end(&mut buffer).unwrap();
        assert_eq!(27, count);
        assert_eq!("012345678901234567890123456".as_bytes(), buffer);
        assert!(verify_test_buffer_data(&buffer, 0, 27));
    }
}
