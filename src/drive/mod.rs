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
use download::download_overwrite_options;

#[cfg(test)]
mod tests {
    use std::io::{BufWriter, Write, Read};
    use std::str::FromStr;
    // use futures::future::ok;
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
            "test_010_upload_3_files_set_diff_size.txt".to_owned(),
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
            "test_020_upload_1_files_set_diff_size.txt".to_owned(),
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
            "test_030_upload_1_file_set_exact_size.txt".to_owned(),
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
    fn test_040_upload_3_big_files_set_diff_size() {
        let data_size = parse_data_size("50 MiB").as_u64();
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("20 MiB").as_u64() as usize,
            "test_big_040_upload_3_files_set_diff_size.txt".to_owned(),
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
    fn test_050_upload_1_big_files_set_diff_size() {
        let data_size = parse_data_size("30 MiB").as_u64();
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("60 MiB").as_u64() as usize,
            "test_big_050_upload_1_files_set_diff_size.txt".to_owned(),
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
    fn test_060_upload_1_big_file_set_exact_size() {
        let data_size = parse_data_size("50 MiB").as_u64();
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("50 MiB").as_u64() as usize,
            "test_big_060_upload_1_file_set_exact_size.txt".to_owned(),
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
    fn test_100_upload_encrypted_file_and_download_decrypted_file() {
        // Encryption keys
        let private_key = age::x25519::Identity::from_str(
            "AGE-SECRET-KEY-15RAENVRSHDVGQ6XZXPUWZK4235AVF6EXFQTS3WG8XMHW0RMSD4EQ492LZ5",
        ).unwrap();
        let public_key = private_key.to_public();

        let data_size = parse_data_size("5 Kib").as_u64();

        // Upload file
        let hub = aw!(auth::auth(None, None));
        let upload_result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("8 KiB").as_u64() as usize,
            "test_110_upload_encrypted_file.txt".to_owned(),
            None,
            false,
            true,
            Some(public_key),
        ));

        // Verify that only one file was uploaded
        assert_eq!(1, upload_result.uploaded_files.len());

        // Testing upload by downloading it and decrypting the data
        let mut encrypted_buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut encrypted_buffer);
        aw!(drive::download_overwrite_options(
            &hub,
            upload_result.uploaded_files.first().as_ref().expect("The Vec is empty"),
            Some(stream.get_mut()),
            Some(0),
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        // Print the encrypted data
        println!("enc_buf ({}): {:?}", encrypted_buffer.len(), encrypted_buffer);

        // Count the zeroes at the end of the downloaded and verify that it is correct
        // It was done to help verify to that there was nothing wrong with
        // the length of the uploaded data, and left because it could help in case of debugging
        let mut _guest_null_count = 0;
        for index in 0..encrypted_buffer.len()-1 {
            let _test = encrypted_buffer[encrypted_buffer.len()-1 - index];
            if encrypted_buffer[encrypted_buffer.len()-1 - index] != 0 {
                _guest_null_count = index;
                break
            }
        };

        // Get the null count
        let _description_null_count = upload_result.uploaded_files.iter().last().unwrap().description.as_ref().map_or(0, |s| s.trim().parse::<i64>().unwrap_or(0));

        // Should succeeded at decrypting data,
        // by loop throught the encrypted data and try to decrypt it.
        // This is to test if the data can be decrypted, even if the decrypted data is incorrect.
        // It was you for debugging at one point and left, because it could be useful later
        let mut decrypted_buffer: Vec<u8> = Vec::new();
        for index in 1..encrypted_buffer.len() {
            let age_decryptor = match age::Decryptor::new(&encrypted_buffer[0..index]) {
                Ok(x) => x,
                Err(e) => {
                    println!("descrypt key error - index: {} - Error: {}", index, e);
                    continue;
                },
            };

            let decryptor = match age_decryptor {
                age::Decryptor::Recipients(d) => d,
                _ => unreachable!(),
            };

            decrypted_buffer.clear();
            let mut reader = decryptor
                .decrypt(::std::iter::once(&private_key as &dyn age::Identity))
                .unwrap();

            match reader.read_to_end(&mut decrypted_buffer) {
                Ok(_t) => break,
                Err(e) => println!("descryption error - index: {} - Error: {}", index, e),
            }
        }

        // Print the decrypted data
        println!("decrypted_buffer ({}): {:?}", decrypted_buffer.len(), decrypted_buffer);

        // Verify the decrypted data
        assert_eq!(data_size as usize, decrypted_buffer.len());
        assert_eq!(verify_test_buffer_data(&decrypted_buffer, 0, data_size as usize), true);
    }

    #[test]
    fn test_110_upload_3_encrypted_files_set_diff_size() {
        // Encryption keys
        let private_key = age::x25519::Identity::from_str(
            "AGE-SECRET-KEY-15RAENVRSHDVGQ6XZXPUWZK4235AVF6EXFQTS3WG8XMHW0RMSD4EQ492LZ5",
        ).unwrap();
        let public_key = private_key.to_public();

        let data_size = parse_data_size("15 kiB").as_u64();

        // Upload file
        let hub = aw!(auth::auth(None, None));
        let upload_result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("6 kiB").as_u64() as usize,
            "test_110_upload_encrypted_file.txt".to_owned(),
            None,
            false,
            true,
            Some(public_key),
        ));

        // Verify that only one file was uploaded
        assert_eq!(3, upload_result.uploaded_files.len());

        // Testing upload by downloading it and decrypting the data
        let mut encrypted_buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut encrypted_buffer);
        aw!(drive::download(
            &hub,
            upload_result.uploaded_files.first().as_ref().expect("The Vec is empty"),
            Some(stream.get_mut()),
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        // Should succeeded at decrypting data,
        let mut decrypted_buffer: Vec<u8> = Vec::new();
        let age_decryptor = age::Decryptor::new(&encrypted_buffer[..]).unwrap_or_else(|err| {panic!("descrypt key error - Error: {}", err)});

        let decryptor = match age_decryptor {
            age::Decryptor::Recipients(d) => d,
            _ => unreachable!(),
        };

        decrypted_buffer.clear();
        let mut reader = decryptor
            .decrypt(::std::iter::once(&private_key as &dyn age::Identity))
            .unwrap();

        reader.read_to_end(&mut decrypted_buffer).unwrap_or_else(|err| {panic!("descryption error - Error: {}", err)});

        // Print the decrypted data
        println!("decrypted_buffer ({}): {:?}", decrypted_buffer.len(), decrypted_buffer);

        // Verify the decrypted data
        assert_eq!(data_size as usize, decrypted_buffer.len());
        assert_eq!(verify_test_buffer_data(&decrypted_buffer, 0, data_size as usize), true);
    }

    #[test]
    fn test_120_upload_1_encrypted_files_set_diff_size() {
        // Encryption keys
        let private_key = age::x25519::Identity::from_str(
            "AGE-SECRET-KEY-15RAENVRSHDVGQ6XZXPUWZK4235AVF6EXFQTS3WG8XMHW0RMSD4EQ492LZ5",
        ).unwrap();
        let public_key = private_key.to_public();

        let data_size = parse_data_size("6 kiB").as_u64();

        // Upload file
        let hub = aw!(auth::auth(None, None));
        let upload_result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("15 kiB").as_u64() as usize,
            "test_110_upload_encrypted_file.txt".to_owned(),
            None,
            false,
            true,
            Some(public_key),
        ));

        // Verify that only one file was uploaded
        assert_eq!(1, upload_result.uploaded_files.len());

        // Testing upload by downloading it and decrypting the data
        let mut encrypted_buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut encrypted_buffer);
        aw!(drive::download(
            &hub,
            upload_result.uploaded_files.first().as_ref().expect("The Vec is empty"),
            Some(stream.get_mut()),
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        // Should succeeded at decrypting data,
        let mut decrypted_buffer: Vec<u8> = Vec::new();
        let age_decryptor = age::Decryptor::new(&encrypted_buffer[..]).unwrap_or_else(|err| {panic!("descrypt key error - Error: {}", err)});

        let decryptor = match age_decryptor {
            age::Decryptor::Recipients(d) => d,
            _ => unreachable!(),
        };

        decrypted_buffer.clear();
        let mut reader = decryptor
            .decrypt(::std::iter::once(&private_key as &dyn age::Identity))
            .unwrap();

        reader.read_to_end(&mut decrypted_buffer).unwrap_or_else(|err| {panic!("descryption error - Error: {}", err)});

        // Print the decrypted data
        println!("decrypted_buffer ({}): {:?}", decrypted_buffer.len(), decrypted_buffer);

        // Verify the decrypted data
        assert_eq!(data_size as usize, decrypted_buffer.len());
        assert_eq!(verify_test_buffer_data(&decrypted_buffer, 0, data_size as usize), true);
    }

    #[test]
    fn test_140_upload_3_big_encrypted_files_set_diff_size() {
        // Encryption keys
        let private_key = age::x25519::Identity::from_str(
            "AGE-SECRET-KEY-15RAENVRSHDVGQ6XZXPUWZK4235AVF6EXFQTS3WG8XMHW0RMSD4EQ492LZ5",
        ).unwrap();
        let public_key = private_key.to_public();

        let data_size = parse_data_size("50 MiB").as_u64();

        // Upload file
        let hub = aw!(auth::auth(None, None));
        let upload_result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("20 MiB").as_u64() as usize,
            "test_110_upload_encrypted_file.txt".to_owned(),
            None,
            false,
            true,
            Some(public_key),
        ));

        // Verify that only one file was uploaded
        assert_eq!(3, upload_result.uploaded_files.len());

        // Testing upload by downloading it and decrypting the data
        let mut encrypted_buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut encrypted_buffer);
        aw!(drive::download(
            &hub,
            upload_result.uploaded_files.first().as_ref().expect("The Vec is empty"),
            Some(stream.get_mut()),
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        // Should succeeded at decrypting data,
        let mut decrypted_buffer: Vec<u8> = Vec::new();
        let age_decryptor = age::Decryptor::new(&encrypted_buffer[..]).unwrap_or_else(|err| {panic!("descrypt key error - Error: {}", err)});

        let decryptor = match age_decryptor {
            age::Decryptor::Recipients(d) => d,
            _ => unreachable!(),
        };

        decrypted_buffer.clear();
        let mut reader = decryptor
            .decrypt(::std::iter::once(&private_key as &dyn age::Identity))
            .unwrap();

        reader.read_to_end(&mut decrypted_buffer).unwrap_or_else(|err| {panic!("descryption error - Error: {}", err)});

        // Print the decrypted data
        // println!("decrypted_buffer ({}): {:?}", decrypted_buffer.len(), decrypted_buffer);

        // Verify the decrypted data
        assert_eq!(data_size as usize, decrypted_buffer.len());
        assert_eq!(verify_test_buffer_data(&decrypted_buffer, 0, data_size as usize), true);
    }

    #[test]
    fn test_150_upload_1_big_encrypted_file_set_diff_size() {
        // Encryption keys
        let private_key = age::x25519::Identity::from_str(
            "AGE-SECRET-KEY-15RAENVRSHDVGQ6XZXPUWZK4235AVF6EXFQTS3WG8XMHW0RMSD4EQ492LZ5",
        ).unwrap();
        let public_key = private_key.to_public();

        let data_size = parse_data_size("30 MiB").as_u64();

        // Upload file
        let hub = aw!(auth::auth(None, None));
        let upload_result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(data_size as usize),
            parse_data_size("60 MiB").as_u64() as usize,
            "test_110_upload_encrypted_file.txt".to_owned(),
            None,
            false,
            true,
            Some(public_key),
        ));

        // Verify that only one file was uploaded
        assert_eq!(1, upload_result.uploaded_files.len());

        // Testing upload by downloading it and decrypting the data
        let mut encrypted_buffer: Vec<u8> = Vec::new();
        let mut stream = BufWriter::new(&mut encrypted_buffer);
        aw!(drive::download(
            &hub,
            upload_result.uploaded_files.first().as_ref().expect("The Vec is empty"),
            Some(stream.get_mut()),
        ));
        stream.flush().expect("Failed at flushing the last data from the buffer: stream");
        drop(stream);

        // Should succeeded at decrypting data,
        let mut decrypted_buffer: Vec<u8> = Vec::new();
        let age_decryptor = age::Decryptor::new(&encrypted_buffer[..]).unwrap_or_else(|err| {panic!("descrypt key error - Error: {}", err)});

        let decryptor = match age_decryptor {
            age::Decryptor::Recipients(d) => d,
            _ => unreachable!(),
        };

        decrypted_buffer.clear();
        let mut reader = decryptor
            .decrypt(::std::iter::once(&private_key as &dyn age::Identity))
            .unwrap();

        reader.read_to_end(&mut decrypted_buffer).unwrap_or_else(|err| {panic!("descryption error - Error: {}", err)});

        // Print the decrypted data
        // println!("decrypted_buffer ({}): {:?}", decrypted_buffer.len(), decrypted_buffer);

        // Verify the decrypted data
        assert_eq!(data_size as usize, decrypted_buffer.len());
        assert_eq!(verify_test_buffer_data(&decrypted_buffer, 0, data_size as usize), true);
    }
}
