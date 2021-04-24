extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
extern crate google_drive3 as drive3;

use self::hyper::{Response, Body};
use self::hyper::body::HttpBody;

use drive3::Result;
use drive3::api::{File, Scope};
use hyper::body::Bytes;

use super::log::{error};

use super::pipe_buffer::{PipeBuffer};

use super::auth::HubType;
use std::process::exit;
use super::misc;
use std::io::{Write, Stdout};
use std::io;
use std::path::PathBuf;
use crate::misc::file_filter;
use std::io::Read;

pub async fn upload<T>(
    hub: &HubType,
    buffer: T,
    size: usize,
    file_name: Option<&str>,
    parent_folder_id: Option<&str>,
    duplicate: bool,
    replace: bool) where T: Read
{
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
            delete(hub, file).await
        }
    }

    let mut buffer = PipeBuffer::new(buffer, size);

    let mut count = 0;
    let mut file_ids: Vec<String> = Vec::new();
    while buffer.is_there_more() {
        if count == 1 {
            match rename(&hub, file_ids.first().unwrap(), format!("{}.000", file_name)).await {
                Ok(_) => info!("Renamed file: '{0}' to '{0}.000'", file_name),
                Err(e) => warn!("Failed at renaming the file '{}' - {}", file_name, e)
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

        let result = hub.files()
            .create(req.clone())
            .supports_all_drives(true)
            .add_scope(Scope::Full)
            .upload_resumable(&mut buffer,
                              "application/octet-stream".parse().unwrap(),
            ).await;

        if buffer.is_there_more() {
            count += 1
        }
        match result {
            Ok(r) => {
                info!("Uploaded file: '{}'", r.1.name.unwrap());
                file_ids.push(r.1.id.unwrap());

                match set_description(hub, file_ids.last().unwrap(), buffer.nulls().to_string()).await {
                    Ok(_) => info!("Set the number of concatenated null (0x00) bytes in the description - '{}.{count:0>3}'", file_name, count = count),
                    Err(e) => warn!("Set the number of concatenated null (0x00) bytes in the description - '{}' - {}", file_name, e)
                }
                info!(r#"FILE ID(s) = "{}""#, file_ids.join(" "))
            }
            Err(e) => {
                error!("Failed at uploading '{}'", e);
                break;
            }
        }
    }
}


pub async fn rename(hub: &HubType, file_id: &str, new_name: String) -> Result<(Response<Body>, File)> {
    let mut file = File::default();
    file.name = Some(new_name);

    hub.files()
        .update(file, file_id)
        .supports_all_drives(true)
        .add_scope(Scope::Full)
        .doit_without_upload().await
}


pub async fn set_description(hub: &HubType, file_id: &str, description: String) -> Result<(Response<Body>, File)> {
    let mut file = File::default();
    file.description = Some(description);

    hub.files()
        .update(file, file_id)
        .supports_all_drives(true)
        .add_scope(Scope::Full)
        .doit_without_upload().await
}


pub async fn info(hub: &HubType, id: &str) -> File {
    let (_, file) = hub.files().get(id)
        .supports_all_drives(true)
        .acknowledge_abuse(false)
        .param("fields", "mimeType,id,kind,teamDriveId,name,driveId,description,size,md5Checksum,parents,trashed")
        .add_scope(Scope::Full)
        .doit()
        .await
        .unwrap_or_else(|e| {
            error!("{}", e);
            exit(misc::EXIT_CODE_010);
        });

    file
}


pub async fn list(hub: &HubType, parent_folder_id: Option<&str>) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();

    info!("Loading file list");

    let mut next_page_token: Option<String> = None;
    while {
        let mut build = hub.files().list()
            .param("fields", "files(mimeType,id,kind,teamDriveId,name,driveId,description,size,md5Checksum,parents,trashed)");

        if parent_folder_id.is_some() {
            build = build
                .corpora("allDrives")
                .include_items_from_all_drives(true)
                .supports_all_drives(true)
                .q(format!("'{}' in parents and trashed = false",
                           parent_folder_id.unwrap()).as_str())
        } else {
            build = build.q("'root' in parents")
        }
        if next_page_token.is_some() {
            build = build.page_token(next_page_token.unwrap().as_str())
        }

        let (_, file_list) = build
            .add_scope(Scope::Full)
            .doit()
            .await
            .unwrap_or_else(|e| {
                error!("List request failed - {}", e);
                exit(misc::EXIT_CODE_007)
            });

        next_page_token = file_list.next_page_token;
        debug!("Next Page Token: {:?}", next_page_token.as_ref().unwrap_or(&"None".to_owned()));
        let mut tmp = file_list.files.unwrap();
        files.append(tmp.as_mut());

        next_page_token.is_some()
    } {}

    files
}


async fn delete(hub: &HubType, file: &File) {
    hub.files().delete(file.id.as_ref().unwrap())
        .supports_all_drives(true)
        .add_scope(Scope::Full)
        .doit()
        .await
        .unwrap_or_else(|e| {
            error!("Failed at deleting the file '{}' - {}", file.name.as_ref().unwrap(), e);
            exit(misc::EXIT_CODE_009);
        });
    info!("Deleted '{}", file.name.as_ref().unwrap())
}


pub async fn create_file_list(hub: &HubType, file: File) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();


    let tmp_path = file.name.as_ref().unwrap().parse::<PathBuf>().unwrap();

    if let Some(file_ext) = tmp_path.extension() {
        let file_ext = file_ext.to_str().unwrap();

        if file_ext.len() >= 3 && file_ext.chars().all(|c| c.is_digit(10)) {
            for p in file.parents.as_ref().unwrap() {
                files = file_filter(
                    format!(r#"^{}(\.[0-9]+)?$"#, regex::escape(tmp_path.file_stem().unwrap().to_str().unwrap())).as_str(),
                    &list(hub, Some(p)).await,
                );
                files.sort_by(|f1, f2| f1.name.as_ref().unwrap().cmp(f2.name.as_ref().unwrap()));
            }
        } else {
            files.push(file);
        }
    } else {
        files.push(file);
    }

    files
}

pub async fn download(hub: &HubType, file_id: &str) {
    // Get info about file
    let info = info(hub, file_id).await;

    // If the file is trashed, don't download
    if info.trashed.is_some() && info.trashed.unwrap() {
        error!("Cannot download the file '{}' because it is trashed", info.name.as_ref().unwrap());
        exit(misc::EXIT_CODE_012)
    }

    // Get all the sibling to this file, if it has any
    let files = create_file_list(hub, info).await;

    // Figure out what the filename is
    let file_name: String;
    if files.len() == 1 {
        file_name = files.first().unwrap().name.as_ref().unwrap().to_owned()
    } else {
        let _tmp = files.first().unwrap()
            .name.as_ref().unwrap()
            .parse::<PathBuf>().clone();
        file_name = _tmp.unwrap().file_stem().unwrap().to_str().unwrap().to_string();
    };

    // Figure out if there should be written to file or stdout
    let mut pipe: Option<Stdout> = None;
    let mut file: Option<::std::fs::File> = None;
    if atty::is(atty::Stream::Stdout) {
        file = Some(::std::fs::File::create(&file_name).expect("Unable to open file"));
    } else {
        pipe = Some(io::stdout());
    }

    // Calulate the total size of all the files
    let total_size = files.iter()
        .map(|file| file.size.as_ref().unwrap_or_else(|| {
            error!("The ID '{}' is not a file", file.id.as_ref().unwrap());
            exit(misc::EXIT_CODE_011)
        }).parse::<usize>().unwrap()).sum::<usize>();


    // Figure out, how much of the last file can be skipped, because of it just being fill'er bytes (0x00)
    let zeros = files.last().unwrap().description.as_ref().map_or(0, |s| {
        s.trim().parse::<usize>().unwrap_or(0)
    });

    // Some validation of the value found in the description
    let size_of_the_last_file = files.last().unwrap().size.as_ref().unwrap().parse::<usize>().unwrap();
    if zeros > size_of_the_last_file {
        error!("The value found in the description of '{}' (ID: {}) which represents the amount of filler bytes (0x00), is biggere then the actual size of the file. There is clearly something wrong.",
               files.last().unwrap().name.as_ref().unwrap(),
               files.last().unwrap().id.as_ref().unwrap(),
            );
        exit(misc::EXIT_CODE_013);
    }

    let actually_file_size = total_size - zeros;


    debug!("File size: {}", actually_file_size);
    info!("Starting to download the file: {}", file_name);
    let mut written = 0;

    let mut write_data = |data: Bytes| {
        if let Some(writer) = pipe.as_mut() {
            writer.lock().write_all(&data.to_vec()).expect("failed at sending data to stdout");
        }
        if let Some(writer) = file.as_mut() {
            writer.write_all(&data.to_vec()).expect("failed writing to file");
        }
    };

    // let mut buf = [0; 2 * 1024 * 1024];
    for _file in files {
        let (response, _) = hub.files().get(_file.id.as_ref().unwrap())
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
                        let _chunk_len = _chunk.len();
                        if written >= actually_file_size {
                            break;
                        } else if _chunk_len + written > actually_file_size {
                            let remaining_len = actually_file_size - written;
                            write_data(_chunk.slice(..remaining_len))
                        } else {
                            write_data(_chunk);
                        }

                        written += _chunk_len
                    },
                    Err(err) => {
                        error!("Download failed: {}", err);
                        exit(misc::EXIT_CODE_014)
                    }
                }
            }
        }
    }

    info!("Download of '{}' Completed", file_name);
}
