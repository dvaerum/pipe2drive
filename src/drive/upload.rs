use crate::auth::HubType;
use crate::drive::{delete, list, rename, set_description};
use crate::misc;
use crate::pipe_buffer::PipeBuffer;
use age::x25519::Recipient;
use google_drive3::api::{File, Scope};
use std::io::Read;
use std::process::exit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UploadResult {
    pub uploaded_files: Vec<File>,
    pub deleted_files: Vec<File>,
}

pub async fn upload<T>(
    hub: &HubType,
    buffer: T,
    size: usize,
    filename: Option<&str>,
    parent_folder_id: Option<&str>,
    duplicate: bool,
    replace: bool,
    encryption_pub_key: Option<Recipient>,
) -> UploadResult where T: Read + std::marker::Send {
    let mut upload_status = UploadResult {
        uploaded_files: vec![],
        deleted_files: vec![],
    };

    // Select file name if nothing is defined
    let filename = filename.map_or("Untitled".to_owned(), |x| x.to_owned());

    let mut file_filter = Vec::new();

    // Check if there already exist files with 
    if !duplicate {
        file_filter = misc::file_filter(
            format!(r#"^{}(\.[0-9]+)?$"#, regex::escape(filename.as_ref())).as_str(),
            &list(&hub, parent_folder_id).await,
        );
        if file_filter.len() > 0 && !replace {
            error!("The file '{}' already exist, use the replace flag (--replace) \
                    if you want to replace this file, or use the duplicate flag (--duplicate) \
                    if you don't care that multiple files have the same filename",
                   filename);
            exit(misc::EXIT_CODE_008);
        }
    }

    if replace {
        for file in &file_filter {
            upload_status.deleted_files.push(file.clone());
            delete(hub, file).await;
        }
    }

    let mut buffer = PipeBuffer::new(
        buffer,
        size,
        encryption_pub_key,
        1024 * 1024 * 4);

    let mut count = 0;
    while buffer.is_there_more() {
        let mut req = File::default();
        if parent_folder_id.is_some() {
            req.parents = Some(vec![parent_folder_id.unwrap().to_owned()]);
        }

        if count == 0 {
            req.name = Some(filename.to_owned());
        } else {
            req.name = Some(format!("{}.{count:0>3}", filename, count = count));
        }

        let result = hub
            .files()
            .create(req.clone())
            .supports_all_drives(true)
            .add_scope(Scope::Full)
            .upload_resumable(
                &mut buffer,
                "application/octet-stream".parse().unwrap())
            .await;

        if buffer.is_there_more() {
            count += 1
        }

        match result {
            Ok(r) => {
                let (_, mut file) = r;

                info!("Uploaded file: '{}'", file.name.as_ref().unwrap());

                if count == 1 {
                    let new_filename = format!("{}.000", filename);

                    let rename_result = rename(
                        &hub,
                        file.id.as_ref().unwrap(),
                        new_filename.clone(),
                    )
                    .await;

                    match rename_result {
                        Ok(_) => {
                            file.name = Some(new_filename);
                            info!("Renamed file: '{0}' to '{0}.000'", filename)
                        },
                        Err(e) => error!("Failed at renaming the file '{}' - {}", filename, e),
                    }
                }

                if !buffer.is_there_more() {
                    let desc_result = set_description(
                        hub,
                        file.id.as_ref().unwrap(),
                        buffer.nulls().to_string()).await;

                    match desc_result {
                        Ok(_) => {
                            file.description = Some(buffer.nulls().to_string());
                            info!("Set the number of concatenated nulls (0x00) \
                                   bytes to {nulls} in the description for '{filename}.{count:0>3}'",
                                  nulls = buffer.nulls(),
                                  filename = filename,
                                  count = count)
                        },
                        Err(e) => warn!("Set the number of concatenated null (0x00) \
                                         bytes in the description - '{}' - {}",
                                        filename, e)
                    }
                }

                debug!(r#"FILE ID = "{}" - NAME = {}"#,
                       file.id.as_ref().unwrap(),
                       file.name.as_ref().unwrap());

                upload_status.uploaded_files.push(file);
            }
            Err(e) => {
                error!("Failed at uploading '{}'", e);
                break;
            }
        }
    }

    return upload_status;
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use crate::auth::{CLIENT_SECRET_FILE, CLIENT_TOKEN_FILE};
    use crate::misc::{config_file, parse_data_size};
    use crate::pipe_buffer::TestBuffer;
    use crate::{auth, drive, misc};

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
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
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(parse_data_size("5 KiB").as_u64() as usize),
            misc::parse_data_size("2 KiB").as_u64() as usize,
            Option::from("test_010_upload_3_files_set_diff_size.txt"),
            None,
            false,
            true,
            None,
        ));

        assert_eq!(3, result.uploaded_files.len())
    }

    #[test]
    fn test_020_upload_1_files_set_diff_size() {
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(parse_data_size("1 KiB").as_u64() as usize),
            misc::parse_data_size("2 KiB").as_u64() as usize,
            Option::from("test_020_upload_1_files_set_diff_size.txt"),
            None,
            false,
            true,
            None,
        ));

        assert_eq!(1, result.uploaded_files.len())
    }

    #[test]
    fn test_030_upload_1_file_set_exact_size() {
        let hub = aw!(auth::auth(None, None));

        let result = aw!(drive::upload::<TestBuffer>(
            &hub,
            TestBuffer::new(parse_data_size("1 KiB").as_u64() as usize),
            misc::parse_data_size("1 KiB").as_u64() as usize,
            Option::from("test_030_upload_1_file_set_exact_size.txt"),
            None,
            false,
            true,
            None,
        ));

        assert_eq!(1, result.uploaded_files.len())
    }
}
