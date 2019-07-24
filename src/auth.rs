extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
extern crate google_drive3 as drive3;
extern crate dirs;

use std::process::exit;

use oauth2::{Authenticator, DefaultAuthenticatorDelegate, ApplicationSecret};
use oauth2::{FlowType, DiskTokenStorage, read_application_secret};

use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

use drive3::DriveHub;

use hyper::Client;

use super::misc;


const CLIENT_SECRET_FILE: &'static str = "client_secret.json";
const CLIENT_TOKEN_FILE: &'static str = "token_store.json";


// reads the provided example client secret, the quick and dirty way.
fn read_client_secret(file: Option<&str>) -> ApplicationSecret {
    let path = misc::config_file(file, CLIENT_SECRET_FILE);
    read_application_secret(
        path.as_path()
    ).unwrap_or_else(|e| {
        error!("{} - {}", e, path.to_string_lossy().to_string());
        exit(misc::EXIT_CODE_004)
    })
}


fn store_client_token(file: Option<&str>) -> DiskTokenStorage {
    let path = misc::config_file(file, CLIENT_TOKEN_FILE).to_string_lossy().to_string();
    DiskTokenStorage::new(
        &path
    ).unwrap_or_else(|e| {
        error!("{} - {}", e, &path);
        exit(misc::EXIT_CODE_005)
    })
}


pub type HubType = DriveHub<Client, Authenticator<DefaultAuthenticatorDelegate, DiskTokenStorage, Client>>;

pub fn auth(client_secret_file: Option<&str>, client_token_file: Option<&str>) -> HubType {
    let secret = read_client_secret(client_secret_file);
    let token = store_client_token(client_token_file);

    let client = hyper::Client::with_connector(
        HttpsConnector::new(NativeTlsClient::new().unwrap())
    );

    let authenticator = Authenticator::new(
        &secret,
        DefaultAuthenticatorDelegate,
        client,
        token,
        Some(FlowType::InstalledInteractive),
    );

    let client = hyper::Client::with_connector(HttpsConnector::new(NativeTlsClient::new().unwrap()));
    DriveHub::new(client, authenticator)
}