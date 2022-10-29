//extern crate hyper;
extern crate dirs;
extern crate google_drive3 as drive3;

use std::process::exit;

use drive3::hyper_rustls::HttpsConnectorBuilder;
use drive3::hyper_rustls::HttpsConnector;
use drive3::hyper::client::HttpConnector;
use drive3::hyper::Client;

use drive3::oauth2::read_application_secret;
use drive3::oauth2::ApplicationSecret;
use drive3::oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

use drive3::DriveHub;

use super::misc;

pub const CLIENT_SECRET_FILE: &'static str = "client_secret.json";
pub const CLIENT_TOKEN_FILE: &'static str = "client_token.json";

// reads the provided example client secret, the quick and dirty way.
async fn read_client_secret(file: Option<String>) -> ApplicationSecret {
    let client_secret_path = misc::config_file(file, CLIENT_SECRET_FILE);

    let var_name = read_application_secret(&client_secret_path).await;
    var_name.unwrap_or_else(|e| {
        error!("{} - {:?}", e, &client_secret_path);
        exit(misc::EXIT_CODE_004)
    })
}

pub type HubType = DriveHub<HttpsConnector<HttpConnector>>;

pub async fn auth(client_secret_file: Option<String>, client_token_file: Option<String>) -> HubType {
    let secret = read_client_secret(client_secret_file);
    let client_token_path = misc::config_file(client_token_file, CLIENT_TOKEN_FILE);

    let auth_result =
        InstalledFlowAuthenticator::builder(secret.await, InstalledFlowReturnMethod::HTTPRedirect)
            .persist_tokens_to_disk(&client_token_path)
            .build()
            .await;
    let auth = auth_result.unwrap_or_else(|err| {
        error!("{} - {:?}", err, &client_token_path);
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
