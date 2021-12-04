mod delete;
mod download;
mod info;
mod list;
mod rename;
mod set_description;
mod upload;

pub use delete::delete;
pub use download::download;
pub use info::info;
pub use list::list;
pub use rename::rename;
pub use set_description::set_description;
pub use upload::{upload, UploadResult};


#[cfg(test)]
mod tests {
    use std::io::{BufWriter, Write};
    use tokio::runtime::Runtime;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use crate::auth::{CLIENT_SECRET_FILE, CLIENT_TOKEN_FILE};
    use crate::misc::{config_file, parse_data_size};
    use crate::pipe_buffer::TestBuffer;
    use crate::{auth, drive};
    use crate::misc::tests::{verify_test_buffer_data};

    macro_rules! aw {
        ($e:expr) => {
            Runtime::new().unwrap().block_on($e)
        };
    }

    #[test]
    fn test_000_check_needed_files_exists() {
        let client_secret_path = config_file(None, CLIENT_SECRET_FILE);
        assert!(client_secret_path.is_file());

        let client_token_path = config_file(None, CLIENT_TOKEN_FILE);
        assert!(client_token_path.is_file());
    }

    #[test]
    fn test_010_upload_3_files_set_diff_size() {
        let data_size = parse_data_size("5 Kib").as_u64();
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("2 KiB").as_u64() as usize,
            Option::from("test_010_upload_3_files_set_diff_size.txt"),
            None,
            false,
            true,
            None,
        ));

        assert_eq!(3, result.uploaded_files.len());

        // Testing upload by downloading
        let mut buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut buffer);
        aw!(drive::download(
            &hub,
            result.uploaded_files.first().as_ref().expect("The Vec is emply"),
            Some(stream.get_mut())
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        assert_eq!(data_size as usize, buffer.len());
        assert_eq!(verify_test_buffer_data(&buffer, 0, data_size as usize), true);
    }

    #[test]
    fn test_020_upload_1_files_set_diff_size() {
        let data_size = parse_data_size("1 Kib").as_u64();
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("2 KiB").as_u64() as usize,
            Option::from("test_020_upload_1_files_set_diff_size.txt"),
            None,
            false,
            true,
            None,
        ));

        assert_eq!(1, result.uploaded_files.len());

        // Testing upload by downloading
        let mut buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut buffer);
        aw!(drive::download(
            &hub,
            result.uploaded_files.first().as_ref().expect("The Vec is emply"),
            Some(stream.get_mut())
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        assert_eq!(data_size as usize, buffer.len());
        assert_eq!(verify_test_buffer_data(&buffer, 0, data_size as usize), true);
    }

    #[test]
    fn test_030_upload_1_file_set_exact_size() {
        let data_size = parse_data_size("1 Kib").as_u64();
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("1 KiB").as_u64() as usize,
            Option::from("test_030_upload_1_file_set_exact_size.txt"),
            None,
            false,
            true,
            None,
        ));

        assert_eq!(1, result.uploaded_files.len());

        // Testing upload by downloading
        let mut buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut buffer);
        aw!(drive::download(
            &hub,
            result.uploaded_files.first().as_ref().expect("The Vec is emply"),
            Some(stream.get_mut())
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        assert_eq!(data_size as usize, buffer.len());
        assert_eq!(verify_test_buffer_data(&buffer, 0, data_size as usize), true);
    }

    #[test]
    fn test_040_upload_big_3_files_set_diff_size() {
        let data_size = parse_data_size("50 MiB").as_u64();
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("20 MiB").as_u64() as usize,
            Option::from("test_big_040_upload_3_files_set_diff_size.txt"),
            None,
            false,
            true,
            None,
        ));

        assert_eq!(3, result.uploaded_files.len());

        // Testing upload by downloading
        let mut buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut buffer);
        aw!(drive::download(
            &hub,
            result.uploaded_files.first().as_ref().expect("The Vec is emply"),
            Some(stream.get_mut())
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        assert_eq!(verify_test_buffer_data(&buffer, 0, data_size as usize), true);
        assert_eq!(data_size as usize, buffer.len());
    }

    #[test]
    fn test_050_upload_big_1_files_set_diff_size() {
        let data_size = parse_data_size("30 MiB").as_u64();
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("60 MiB").as_u64() as usize,
            Option::from("test_big_050_upload_1_files_set_diff_size.txt"),
            None,
            false,
            true,
            None,
        ));

        assert_eq!(1, result.uploaded_files.len());

        // Testing upload by downloading
        let mut buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut buffer);
        aw!(drive::download(
            &hub,
            result.uploaded_files.first().as_ref().expect("The Vec is emply"),
            Some(stream.get_mut())
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        assert_eq!(verify_test_buffer_data(&buffer, 0, data_size as usize), true);
        assert_eq!(data_size as usize, buffer.len());
    }

    #[test]
    fn test_060_upload_big_1_file_set_exact_size() {
        let data_size = parse_data_size("50 MiB").as_u64();
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("50 MiB").as_u64() as usize,
            Option::from("test_big_060_upload_1_file_set_exact_size.txt"),
            None,
            false,
            true,
            None,
        ));

        assert_eq!(1, result.uploaded_files.len());

        // Testing upload by downloading
        let mut buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut buffer);
        aw!(drive::download(
            &hub,
            result.uploaded_files.first().as_ref().expect("The Vec is emply"),
            Some(stream.get_mut())
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        assert_eq!(verify_test_buffer_data(&buffer, 0, data_size as usize), true);
        assert_eq!(data_size as usize, buffer.len());
    }
}
