use crate::auth::HubType;
use crate::drive::{delete, list, rename, set_description};
use crate::misc;
use crate::pipe_buffer::PipeBuffer;
use age::x25519::Recipient;
use google_drive3::api::{File, Scope};
use std::io::Read;
use std::process::exit;

pub struct UploadStatus {
    pub upload_ids: Vec<String>,
    pub delete_ids: Vec<String>,
}

pub async fn upload<T>(
    hub: &HubType,
    buffer: T,
    size: usize,
    file_name: Option<&str>,
    parent_folder_id: Option<&str>,
    duplicate: bool,
    replace: bool,
    _encryption_pub_key: Option<Recipient>,
) -> UploadStatus
where
    T: Read,
{
    let mut upload_status = UploadStatus {
        upload_ids: vec![],
        delete_ids: vec![],
    };

    let file_name = file_name.map_or("Untitled".to_owned(), |x| x.to_owned());

    let mut file_filter = Vec::new();
    if !duplicate {
        file_filter = misc::file_filter(
            format!(r#"^{}(\.[0-9]+)?$"#, regex::escape(file_name.as_ref())).as_str(),
            &list(&hub, parent_folder_id).await,
        );
        if file_filter.len() > 0 && !replace {
            error!("The file '{}' already exist, use the replace flag (--replace) if you want to replace this file, or use the duplicate flag (--duplicate) if you don't care that multiple files have the same file_name",
                   file_name);
            exit(misc::EXIT_CODE_008);
        }
    }

    if replace {
        for file in &file_filter {
            upload_status.delete_ids.push(file.id.clone().unwrap());
            delete(hub, file).await;
        }
    }

    let mut buffer = PipeBuffer::new(buffer, size);

    let mut count = 0;
    while buffer.is_there_more() {
        if count == 1 {
            let rename_result = rename(
                &hub,
                upload_status.upload_ids.first().unwrap(),
                format!("{}.000", file_name),
            )
            .await;

            match rename_result {
                Ok(_) => info!("Renamed file: '{0}' to '{0}.000'", file_name),
                Err(e) => error!("Failed at renaming the file '{}' - {}", file_name, e),
            }
        }

        let mut req = File::default();
        if parent_folder_id.is_some() {
            req.parents = Some(vec![parent_folder_id.unwrap().to_owned()]);
        }

        if count > 0 {
            req.name = Some(format!("{}.{count:0>3}", file_name, count = count));
        } else {
            req.name = Some(file_name.to_owned());
        }

        let result = hub
            .files()
            .create(req.clone())
            .supports_all_drives(true)
            .add_scope(Scope::Full)
            .upload_resumable(&mut buffer, "application/octet-stream".parse().unwrap())
            .await;

        if buffer.is_there_more() {
            count += 1
        }
        match result {
            Ok(r) => {
                info!("Uploaded file: '{}'", r.1.name.unwrap());
                upload_status.upload_ids.push(r.1.id.unwrap());

                match set_description(hub, upload_status.upload_ids.last().unwrap(), buffer.nulls().to_string()).await {
                    // TODO: Fix info.rs message so it doesn't add '.000' to the file name of the 'count' is '0'
                    Ok(_) => info!("Set the number of concatenated null (0x00) bytes in the description - '{}.{count:0>3}'", file_name, count = count),
                    Err(e) => warn!("Set the number of concatenated null (0x00) bytes in the description - '{}' - {}", file_name, e)
                }
                info!(r#"FILE ID(s) = "{}""#, upload_status.upload_ids.join(" "))
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

        assert_eq!(3, result.upload_ids.len())
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

        assert_eq!(1, result.upload_ids.len())
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

        assert_eq!(1, result.upload_ids.len())
    }
}
