use crate::auth::HubType;
use crate::drive::list::create_file_list;
use crate::misc;
use google_drive3::api::Scope;
use google_drive3::hyper::body::{HttpBody, Bytes};
use std::borrow::BorrowMut;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use google_drive3::api::{File};

pub async fn download<'a>(hub: &HubType, info: &File, stream: Option<&mut dyn Write>) {
    download_overwrite_options(hub, info, stream, None).await
}

pub (crate) async fn download_overwrite_options<'a>(
    hub: &HubType,
    info: &File,
    mut stream: Option<&mut dyn Write>,
    overwrite_zero_count: Option<i64>,
) {
    // If the file is trashed, don't download
    if info.trashed.is_some() && info.trashed.unwrap() {
        error!(
            "Cannot download the file '{}' because it is trashed",
            info.name.as_ref().unwrap()
        );
        exit(misc::EXIT_CODE_012)
    }

    // Get all the sibling to this file, if it has any
    let files = create_file_list(hub, info).await;

    // Figure out what the filename is
    let file_name: String;
    if files.len() == 1 {
        file_name = files.first().unwrap().name.as_ref().unwrap().to_owned()
    } else {
        let _tmp = files
            .first()
            .unwrap()
            .name
            .as_ref()
            .unwrap()
            .parse::<PathBuf>()
            .clone();
        file_name = _tmp
            .unwrap()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
    };

    // Figure out if there should be written to file or stdout
    let mut file;
    if stream.is_none() {
        file = ::std::fs::File::create(&file_name).expect("Unable to open file");
        stream = Some(file.borrow_mut());
    }

    // Calculate the total size of all the files
    let total_size = files
        .iter()
        .map(|file| {
            file.size
                .as_ref()
                .unwrap_or_else(|| {
                    error!("The ID '{:?}' is not a file", file.id.as_ref());
                    exit(misc::EXIT_CODE_011)
                }).to_owned().parse::<i64>().unwrap()
        }).sum::<i64>();

    // Figure out, how much of the last file can be skipped, because of it just being fill'er bytes (0x00)
    let zero_count = overwrite_zero_count.unwrap_or(
        files.last().unwrap().description.as_ref().map_or(0, |s| s.trim().parse::<i64>().unwrap_or(0))
    );

    // Some validation of the value found in the description
    let size_of_the_last_file = files
        .last()
        .unwrap()
        .size
        .as_ref()
        .unwrap().to_owned().parse::<i64>().unwrap();
    if zero_count > size_of_the_last_file {
        error!("The value found in the description of '{}' (ID: {}) which represents the amount of filler bytes (0x00), is biggere then the actual size of the file. There is clearly something wrong.",
               files.last().unwrap().name.as_ref().unwrap(),
               files.last().unwrap().id.as_ref().unwrap(),
        );
        exit(misc::EXIT_CODE_013);
    }

    let actually_file_size = total_size - zero_count;

    debug!("File size: {}", actually_file_size);
    info!("Starting to download the file: {}", file_name);
    let mut written = 0;

    let mut write_data = |data: Bytes| {
        if let Some(writer) = stream.as_mut() {
            writer
                .write_all(&data.to_vec())
                .expect("failed to write data");
        }
    };

    for _file in files {
        let (response, _) = hub
            .files()
            .get(_file.id.as_ref().unwrap())
            .supports_all_drives(true)
            .acknowledge_abuse(false)
            .param("alt", "media")
            .add_scope(Scope::Full)
            .doit()
            .await
            .unwrap_or_else(|e| {
                error!("{}", e);
                exit(misc::EXIT_CODE_010);
            });

        let mut body = response.into_body();
        loop {
            let _chunk_option = body.data().await;

            if let Some(_chunk_result) = _chunk_option {
                match _chunk_result {
                    Ok(_chunk) => {
                        let _chunk_len = _chunk.len() as i64;
                        if written >= actually_file_size {
                            break;
                        } else if _chunk_len + written > actually_file_size {
                            let remaining_len = actually_file_size - written;
                            write_data(_chunk.slice(..remaining_len as usize))
                        } else {
                            write_data(_chunk);
                        }

                        written += _chunk_len
                    }
                    Err(err) => {
                        error!("Download failed: {}", err);
                        exit(misc::EXIT_CODE_014)
                    }
                }
            } else {
                break;
            }
        }
    }

    info!("Download of '{}' Completed", file_name);
}
