use crate::auth::HubType;
use crate::drive::info::FIELDS;
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
    filename: String,
    parent_folder_id: Option<String>,
    duplicate: bool,
    replace: bool,
    encryption_pub_key: Option<Recipient>,
) -> UploadResult where T: Read + std::marker::Send {
    let mut upload_status = UploadResult {
        uploaded_files: vec![],
        deleted_files: vec![],
    };

    let mut filename = filename.clone();

    if encryption_pub_key.is_some() {
        filename.push_str(".age")
    }

    let mut file_filter = Vec::new();

    // Check if there already exist files with
    if !duplicate {
        file_filter = misc::file_filter(
            format!(r#"^{}(\.[0-9]+)?$"#, regex::escape(filename.as_ref())).as_str(),
            &list(&hub, parent_folder_id.clone()).await,
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
            req.parents = Some(vec![parent_folder_id.as_ref().unwrap().to_owned()]);
        }

        req.name = Some(format!("{}.{count:0>3}", &filename, count = count));

        let result = hub
            .files()
            .create(req.clone())
            .supports_all_drives(true)
            .param("fields", FIELDS)
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

                if count == 0 {
                    let rename_result = rename(
                        &hub,
                        file.id.as_ref().unwrap(),
                        filename.clone(),
                    )
                    .await;

                    match rename_result {
                        Ok(r) => {
                            let (_, file) = r;
                            info!("Renamed file: '{0}.000' to '{0}'", &file.name.unwrap())
                        },
                        Err(e) => error!("Failed at renaming the file '{}' - {}", &filename, e),
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
                                   bytes to {nulls} in the description for '{filename}{suffix}'",
                                  nulls = buffer.nulls(),
                                  filename = filename,
                                  suffix = if count == 0 { "".to_owned() } else { format!(".{count:0>3}", count = count) })
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
