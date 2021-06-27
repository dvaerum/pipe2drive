use crate::auth::HubType;
use crate::misc;
use google_drive3::api::{File, Scope};
use std::process::exit;

pub async fn delete(hub: &HubType, file: &File) {
    hub.files()
        .delete(file.id.as_ref().unwrap())
        .supports_all_drives(true)
        .add_scope(Scope::Full)
        .doit()
        .await
        .unwrap_or_else(|e| {
            error!(
                "Failed at deleting the file '{}' - {}",
                file.name.as_ref().unwrap(),
                e
            );
            exit(misc::EXIT_CODE_009);
        });
    info!("Deleted '{}", file.name.as_ref().unwrap())
}
