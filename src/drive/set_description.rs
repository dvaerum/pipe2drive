use crate::auth::HubType;
use google_drive3::api::{File, Scope};
use google_drive3::Result;
use google_drive3::hyper::{Body, Response};

pub async fn set_description(
    hub: &HubType,
    file_id: &str,
    description: String,
) -> Result<(Response<Body>, File)> {
    let mut file = File::default();
    file.description = Some(description);

    hub.files()
        .update(file, file_id)
        .supports_all_drives(true)
        .add_scope(Scope::Full)
        .doit_without_upload()
        .await
}
