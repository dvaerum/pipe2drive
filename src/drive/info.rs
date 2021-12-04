use crate::auth::HubType;
use crate::misc;
use google_drive3::api::{File, Scope};
use std::process::exit;

pub const FIELDS: &str = "mimeType,id,kind,teamDriveId,name,driveId,description,size,md5Checksum,parents,trashed";

pub async fn info(hub: &HubType, id: &str) -> File {
    let (_, file) = hub.files().get(id)
        .supports_all_drives(true)
        .acknowledge_abuse(false)
        .param("fields", FIELDS)
        .add_scope(Scope::Full)
        .doit()
        .await
        .unwrap_or_else(|e| {
            error!("{}", e);
            exit(misc::EXIT_CODE_010);
        });

    file
}
