use crate::misc::config_file;
use age::x25519::{Identity, Recipient};
use std::fs;
use std::str::FromStr;
use secrecy::ExposeSecret;

pub fn load_public_key() -> Recipient {
    let path = config_file(None, "pipe2drive.pub");

    let private_key = age::x25519::Identity::from(load_private_key());
    private_key.to_public()
}

pub fn load_private_key() -> Identity {
    let path = config_file(None, "pipe2drive.key");

    let private_key: Identity;
    if path.exists() {
        let private_key_armor = fs::read_to_string(path.as_path()).expect(
            format!(
                "Failed at reading the private key: {}",
                path.to_str().unwrap()
            )
            .as_str(),
        );
        private_key = age::x25519::Identity::from_str(private_key_armor.as_str()).expect(
            format!(
                "Failed at reading the private key: {}",
                path.to_str().unwrap()
            )
            .as_str(),
        );
        println!("read from file: {}", private_key.to_string().expose_secret());
    } else {
        private_key = age::x25519::Identity::generate();
        
        println!("create the nothing: {}", private_key.to_string().expose_secret());
    }


    private_key
}
