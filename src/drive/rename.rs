use crate::auth::HubType;
use google_drive3::api::{File, Scope};
use google_drive3::Result;
use google_drive3::hyper::{Body, Response};

pub async fn rename(
    hub: &HubType,
    file_id: &str,
    new_name: String,
) -> Result<(Response<Body>, File)> {
    let mut file = File::default();
    file.name = Some(new_name);

    hub.files()
        .update(file, file_id)
        .supports_all_drives(true)
        .add_scope(Scope::Full)
        .doit_without_upload()
        .await
}
