extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
extern crate google_drive3 as drive3;

use self::hyper::client::Response;

use drive3::{Result, File};

use super::pipe_buffer::PipeBuffer;

use super::auth::HubType;
use std::process::exit;
use super::misc;
use std::borrow::BorrowMut;
use std::io::{Write, Read, Stdout};
use std::io;
use std::path::PathBuf;
use crate::misc::file_filter;


pub fn upload(hub: &HubType,
              size: usize,
              file_name: Option<&str>,
              parent_folder_id: Option<&str>,
              duplicate: bool,
              replace: bool)
{
    let file_name = file_name.map_or("Untitled".to_owned(), |x| x.to_owned());

    let mut file_filter = Vec::new();
    if !duplicate {
        file_filter = misc::file_filter(
            format!(r#"^{}(\.[0-9]+)?$"#, regex::escape(file_name.as_ref())).as_str(),
            &list(&hub, parent_folder_id),
        );
        if file_filter.len() > 0 && !replace {
            error!("The file '{}' already exist, use the replace flag (--replace) if you want to replace this file, or use the duplicate flag (--duplicate) if you don't care that multiple files have the same file_name",
                   file_name);
            exit(misc::EXIT_CODE_008);
        }
    }

    if replace {
        for file in &file_filter {
            delete(hub, file)
        }
    }

    let stdin = io::stdin();
    let mut buffer = PipeBuffer::new(stdin.lock(), size);
    let mut count = 0;
    let mut file_ids: Vec<String> = Vec::new();
    while buffer.is_there_more() {
        if count == 1 {
            match rename(&hub, file_ids.first().unwrap(), format!("{}.000", file_name)) {
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
            .upload_resumable(&mut buffer,
                              "application/octet-stream".parse().unwrap(),
            );

        if buffer.is_there_more() { count += 1 }
        match result {
            Ok(r) => {
                info!("Uploaded file: '{}'", r.1.name.unwrap());
                file_ids.push(r.1.id.unwrap())
            }
            Err(e) => {
                error!("Failed at uploading '{}'", e);
                break;
            }
        }
    }

    match set_description(hub, file_ids.last().unwrap(), buffer.nulls().to_string()) {
        Ok(_) => info!("Set the number of concatenated null (0x00) bytes in the description - '{}.{count:0>3}'", file_name, count = count),
        Err(e) => warn!("Set the number of concatenated null (0x00) bytes in the description - '{}' - {}", file_name, e)
    }

    info!(r#"FILE ID(s) = "{}""#, file_ids.join(" "))
}


pub fn rename(hub: &HubType, file_id: &str, new_name: String) -> Result<(Response, File)> {
    let mut file = File::default();
    file.name = Some(new_name);

    hub.files()
        .update(file, file_id)
        .supports_all_drives(true)
        .doit_without_upload()
}


pub fn set_description(hub: &HubType, file_id: &str, description: String) -> Result<(Response, File)> {
    let mut file = File::default();
    file.description = Some(description);

    hub.files()
        .update(file, file_id)
        .supports_all_drives(true)
        .doit_without_upload()
}


pub fn info(hub: &HubType, id: &str) -> File {
    let (_, file) = hub.files().get(id)
        .supports_all_drives(true)
        .acknowledge_abuse(false)
        .param("fields", "mimeType,id,kind,teamDriveId,name,driveId,description,size,parents,trashed")
        .doit()
        .unwrap_or_else(|e| {
            error!("{}", e);
            exit(misc::EXIT_CODE_010);
        });

    file
}


pub fn list(hub: &HubType, parent_folder_id: Option<&str>) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();

    info!("Loading file list");

    let mut next_page_token: Option<String> = None;
    while {
        let mut build = hub.files().list()
            .param("fields", "files(mimeType,id,kind,teamDriveId,name,driveId,description,size,parents,trashed)");

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

        let (_, file_list) = build.doit().unwrap_or_else(|e| {
            error!("List request failed - {}", e);
            exit(misc::EXIT_CODE_007)
        });

        next_page_token = file_list.next_page_token;
        debug!("Next Page Token: {:?}", next_page_token.as_ref().unwrap_or(&"None".to_owned()));
        files.append(file_list.files.unwrap().borrow_mut());

        next_page_token.is_some()
    } {}

    files
}


fn delete(hub: &HubType, file: &File) {
    hub.files().delete(file.id.as_ref().unwrap())
        .supports_all_drives(true)
        .doit()
        .unwrap_or_else(|e| {
            error!("Failed at deleting the file '{}' - {}", file.name.as_ref().unwrap(), e);
            exit(misc::EXIT_CODE_009);
        });
    info!("Deleted '{}", file.name.as_ref().unwrap())
}


pub fn create_file_list(hub: &HubType, file: File) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();
    let tmp_path = file.name.as_ref().unwrap().parse::<PathBuf>().unwrap();

    if let Some(file_ext) = tmp_path.extension() {
        let file_ext = file_ext.to_str().unwrap();

        if file_ext.len() >= 3 && file_ext.chars().all(|c| c.is_digit(10)) {
            for p in file.parents.as_ref().unwrap() {
                files = file_filter(
                    format!(r#"^{}(\.[0-9]+)?$"#, regex::escape(tmp_path.file_stem().unwrap().to_str().unwrap())).as_str(),
                    &list(hub, Some(p)),
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

pub fn download(hub: &HubType, file_id: &str) {
    let info = info(hub, file_id);
    if info.trashed.is_some() && info.trashed.unwrap() {
        error!("Cannot download the file '{}' because it is trashed", info.name.as_ref().unwrap());
        exit(misc::EXIT_CODE_012)
    }

    let files = create_file_list(hub, info);


    let file_name;
    if files.len() == 1 {
        file_name = files.first().unwrap().name.as_ref().unwrap().to_owned()
    } else {
        let _tmp = files.first().unwrap()
            .name.as_ref().unwrap()
            .parse::<PathBuf>().clone();
        file_name = _tmp.unwrap().file_stem().unwrap().to_str().unwrap().to_string();
    };

    let mut pipe: Option<Stdout> = None;
    let mut file: Option<::std::fs::File> = None;
    if atty::is(atty::Stream::Stdout) {
        file = Some(::std::fs::File::create(&file_name).expect("Unable to open file"));
    } else {
        pipe = Some(io::stdout());
    }

    let total_size = files.iter()
        .map(|file| file.size.as_ref().unwrap_or_else(|| {
            error!("The ID '{}' is not a file", file.id.as_ref().unwrap());
            exit(misc::EXIT_CODE_011)
        }).parse::<usize>().unwrap()).sum();

    let mut zeros = files.last().unwrap().description.as_ref().map_or(0, |s| {
        s.trim().parse::<usize>().unwrap_or(0)
    });

    if zeros > total_size {
        zeros = 0;
    }

    let file_size = total_size - zeros;


    debug!("File size: {}", file_size);
    info!("Starting to download the file: {}", file_name);
    let mut written = 0;
    let mut buf = [0; 2 * 1024 * 1024];
    for _file in files {
        let (mut response, _) = hub.files().get(_file.id.as_ref().unwrap())
            .supports_all_drives(true)
            .acknowledge_abuse(false)
            .param("alt", "media")
            .add_scope(drive3::Scope::Full)
            .doit()
            .unwrap_or_else(|e| {
                error!("{}", e);
                exit(misc::EXIT_CODE_010);
            });


        loop {
            let len = match response.read(&mut buf) {
                Ok(0) => break,  // EOF.
                Ok(len) => {
                    if written >= file_size {
                        break;
                    } else if len + written > file_size {
                        file_size - written
                    } else {
                        len
                    }
                }
                Err(ref err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) => return error!("{}: Download failed: {}", "", err),
            };

            if let Some(writer) = pipe.as_mut() {
                writer.lock().write_all(&buf[..len]).expect("failed writing to file");
            }
            if let Some(writer) = file.as_mut() {
                writer.write_all(&buf[..len]).expect("failed writing to file");
            }

            written += len;
        }
    }

    info!("Download of '{}' Completed", file_name);
}
