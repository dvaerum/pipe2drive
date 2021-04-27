//extern crate hyper;
extern crate dirs;
extern crate google_drive3 as drive3;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;

use std::process::exit;

use hyper::Client;
use hyper_rustls::HttpsConnector;

use oauth2::read_application_secret;
use oauth2::ApplicationSecret;
use oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

use drive3::DriveHub;

use super::misc;

const CLIENT_SECRET_FILE: &'static str = "client_secret.json";
const CLIENT_TOKEN_FILE: &'static str = "client_token.json";

fn select_path(file: Option<&str>, default_path: &str) -> String {
    let path = misc::config_file(file, default_path)
        .to_string_lossy()
        .to_string();
    path
}

// reads the provided example client secret, the quick and dirty way.
async fn read_client_secret(file: Option<&str>) -> ApplicationSecret {
    let client_secret_path = select_path(file, CLIENT_SECRET_FILE);

    let var_name = read_application_secret(&client_secret_path).await;
    var_name.unwrap_or_else(|e| {
        error!("{} - {}", e, &client_secret_path);
        exit(misc::EXIT_CODE_004)
    })
}

pub type HubType = DriveHub;

pub async fn auth(client_secret_file: Option<&str>, client_token_file: Option<&str>) -> HubType {
    let secret = read_client_secret(client_secret_file);
    let client_token_path = select_path(client_token_file, CLIENT_TOKEN_FILE);

    let auth_result =
        InstalledFlowAuthenticator::builder(secret.await, InstalledFlowReturnMethod::Interactive)
            .persist_tokens_to_disk(&client_token_path)
            .build()
            .await;
    let auth = auth_result.unwrap_or_else(|err| {
        error!("{} - {}", err, &client_token_path);
        exit(misc::EXIT_CODE_005)
    });

    let client = Client::builder().build(HttpsConnector::with_native_roots());
    DriveHub::new(client, auth)
}
