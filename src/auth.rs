//extern crate hyper;
extern crate dirs;
extern crate google_drive3 as drive3;

use std::{env};
use std::process::exit;

use drive3::hyper_rustls::HttpsConnectorBuilder;
use drive3::hyper_rustls::HttpsConnector;
use drive3::hyper::client::HttpConnector;
use drive3::hyper::Client;

use drive3::oauth2::parse_application_secret;
use drive3::oauth2::read_application_secret;
use drive3::oauth2::ApplicationSecret;
use drive3::oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

use drive3::DriveHub;
use tokio::io::AsyncWriteExt;

use super::misc;

// Mainly use for testing.
pub const CLIENT_SECRET_ENV: &'static str = "PIPE2DRIVE_CLIENT_SECRET_DATA";
pub const CLIENT_TOKEN_ENV: &'static str = "PIPE2DRIVE_CLIENT_TOKEN_DATA";

pub const CLIENT_SECRET_FILE: &'static str = "client_secret.json";
pub const CLIENT_TOKEN_FILE: &'static str = "client_token.json";

// reads the provided example client secret, the quick and dirty way.
async fn read_client_secret(file: Option<String>) -> ApplicationSecret {
    let client_secret_path = misc::config_file(file, CLIENT_SECRET_FILE);

    let var_name = read_application_secret(&client_secret_path).await;
    var_name.unwrap_or_else(|err| {
        error!("Failed at reading the Google Drive secret - Error: {err} - client_secret_path: {:?}", &client_secret_path);
        exit(misc::EXIT_CODE_004)
    })
}

pub type HubType = DriveHub<HttpsConnector<HttpConnector>>;

pub (crate) async fn auth(client_secret_file: Option<String>, client_token_file: Option<String>) -> HubType {
    let client_secret = match env::var(CLIENT_SECRET_ENV) {
        Ok(data) => {
            parse_application_secret(data.as_bytes()).unwrap_or_else(|e| {
                error!("Error reading the data from the environment variable ({CLIENT_SECRET_ENV}) - Error: {e} - Data: {:?}", &data);
                exit(misc::EXIT_CODE_004)
            })
        }
        Err(_) => read_client_secret(client_secret_file).await
    };

    let client_token_path = misc::config_file(client_token_file, CLIENT_TOKEN_FILE);

    if !client_token_path.exists() {
        debug!("No client token file found - Creating one");
        if let Ok(data) = env::var(CLIENT_TOKEN_ENV) {
            tokio::fs::File::create(&client_token_path).await.unwrap_or_else(|err| {
                error!("Error creating the client token file - Error: {err} - Path: {:?}", &client_token_path);
                exit(misc::EXIT_CODE_004)
            }).write_all(data.as_ref()).await.unwrap_or_else(|err|{
                error!("Error creating the client token file - Error: {err} - Path: {:?}", &client_token_path);
                exit(misc::EXIT_CODE_004)
            });
        };
    }

    let auth_result =
        InstalledFlowAuthenticator::builder(client_secret, InstalledFlowReturnMethod::HTTPRedirect)
            .persist_tokens_to_disk(&client_token_path)
            .build()
            .await;
    let auth = auth_result.unwrap_or_else(|err| {
        error!("Failed at auth to Google Drive - Error: {err} - client_token_path: {:?}", &client_token_path);
        exit(misc::EXIT_CODE_005)
    });

    let client = Client::builder().build(
        HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http2()
            .build()
    );
    DriveHub::new(client, auth)
}
