extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
extern crate google_drive3 as drive3;


use drive3::File;

use super::pipe_buffer::PipeBuffer;

use super::auth::HubType;
use std::process::exit;
use crate::misc;
use std::borrow::BorrowMut;

pub fn upload(hub: &HubType, size: usize, file_name: Option<&str>, parent_folder_id: Option<&str>, duplicate: bool, replace: bool) {
    let mut file_filter = Vec::new();
    if file_name.is_some() && !duplicate {
        file_filter = misc::file_filter(
            format!(r#"^{}(\.0+)?$"#, regex::escape(file_name.as_ref().unwrap())).as_str(),
            &list(&hub, parent_folder_id)
        );
        if file_filter.len() > 0 && !replace {
            error!("The file '{}' already exist, use the replace flag (--replace) if you want to replace this file, or use the duplicate flag (--duplicate) if you don't care that multiple files have the same file_name",
                   file_name.as_ref().unwrap());
            exit(misc::EXIT_CODE_008);
        }
    }

    if replace {
        for file in &file_filter {
            delete(hub,file)
        }
    }

    let stdin = ::std::io::stdin();
    let mut buffer = PipeBuffer::new(stdin.lock(), size);
    let mut count = 0;
    let mut file_ids: Vec<String> = Vec::new();
    while buffer.is_there_more() {
        if count == 1 {
            rename(&hub, file_ids.first().unwrap(), format!("{}.000", file_name.unwrap()), &parent_folder_id)
        }


        let mut req = File::default();
        if parent_folder_id.is_some() {
            req.parents = Some(vec![parent_folder_id.unwrap().to_owned()]);
        }
        if file_name.is_some() {
            if count > 0 {
                req.name = Some(format!("{}.{count:0>3}", file_name.unwrap(), count = count));
            } else {
                req.name = Some(file_name.unwrap().to_owned());
            }
        }

        let result = hub.files()
            .create(req.clone())
            .supports_all_drives(true)
            .upload_resumable(&mut buffer,
                              "application/octet-stream".parse().unwrap(),
            );

        count += 1;

        match result {
            Ok(r) => {
                info!("Uploaded file: {}", r.1.name.unwrap());
                file_ids.push(r.1.id.unwrap())
            },
            Err(e) => {
                error!("{}", e);
                break;
            }
        }
    }

    let mut tmp = String::new();
    for file_id in file_ids {
        tmp = format!(r#"{}" "{}"#, tmp, file_id)
    }
    let trim: &[_] = &[' ', '"'];
    info!(r#"FILE ID(s) = "{}""#, tmp.trim_matches(trim))
}

// TODO: Fix this workaround, if it is ever discovered how to rename files
pub fn rename(hub: &HubType, file_id: &str, new_name: String, parent_folder_id: &Option<&str>) {
    let mut file = File::default();
    file.name = Some(new_name);
    file.parents = Some(vec![parent_folder_id.unwrap().to_owned()]);

    let result = hub.files()
        .copy(file, file_id)
        .supports_all_drives(true)
        .doit();

    debug!("Copy - {:?}", result);

    let result = hub.files().delete(file_id)
        .supports_all_drives(true)
        .doit();

    debug!("Delete Old - {:?}", result);
}

pub fn list(hub: &HubType, parent_folder_id: Option<&str>) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();

    info!("Loading file list");

    let mut next_page_token: Option<String> = None;
    while {
        let mut build = hub.files().list();

        if parent_folder_id.is_some() {
            build = build
                .corpora("allDrives")
                .include_items_from_all_drives(true)
                .supports_all_drives(true)
                .q(format!("'{}' in parents and trashed = false",
                           parent_folder_id.unwrap()).as_str())
        }
        if next_page_token.is_some() {
            build = build.page_token(next_page_token.unwrap().as_str())
        }

        let (_, file_list) = build.doit().unwrap_or_else(|e| {
            error!("{}", e);
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