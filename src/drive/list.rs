use crate::auth::HubType;
use crate::misc;
use crate::misc::file_filter;
use google_drive3::api::{File, Scope};
use std::path::PathBuf;
use std::process::exit;

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
                .q(format!(
                    "'{}' in parents and trashed = false",
                    parent_folder_id.unwrap()
                )
                .as_str())
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
        debug!(
            "Next Page Token: {:?}",
            next_page_token.as_ref().unwrap_or(&"None".to_owned())
        );
        let mut tmp = file_list.files.unwrap();
        files.append(tmp.as_mut());

        next_page_token.is_some()
    } {}

    files
}

pub async fn create_file_list(hub: &HubType, file: File) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();

    let tmp_path = file.name.as_ref().unwrap().parse::<PathBuf>().unwrap();

    if let Some(file_ext) = tmp_path.extension() {
        let file_ext = file_ext.to_str().unwrap();

        if file_ext.len() >= 3 && file_ext.chars().all(|c| c.is_digit(10)) {
            for p in file.parents.as_ref().unwrap() {
                files = file_filter(
                    format!(
                        r#"^{}(\.[0-9]+)?$"#,
                        regex::escape(tmp_path.file_stem().unwrap().to_str().unwrap())
                    )
                    .as_str(),
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
