use crate::misc::config_file;
use age::x25519::{Identity, Recipient};
use secrecy::ExposeSecret;
use std::fs;
use std::str::FromStr;

pub fn load_public_key() -> Recipient {
    let public_key: Recipient;
    let path = config_file(None, "pipe2drive.pub");

    if path.exists() {
        let public_key_string = fs::read_to_string(path.as_path()).expect(
            format!(
                "Failed at reading the public key: {}",
                path.to_str().unwrap()
            )
            .as_str(),
        );
        public_key = age::x25519::Recipient::from_str(public_key_string.as_str()).expect(&format!(
            "Failed at reading the public key: {}",
            path.to_str().unwrap()
        ));
        info!("Read the key from: {}", path.as_path().to_str().unwrap());
    } else {
        public_key = load_private_key().to_public();

        fs::write(path.as_path(), public_key.to_string().as_bytes()).expect(&format!(
            "Failed at reading the public key: {}",
            path.to_str().unwrap()
        ));

        info!("Write the public key to: {}", path.to_str().unwrap());
    }

    return public_key;
}

pub fn load_private_key() -> Identity {
    let private_key: Identity;
    let path = config_file(None, "pipe2drive.key");

    if path.exists() {
        let private_key_string = fs::read_to_string(path.as_path()).expect(
            format!(
                "Failed at reading the private key: {}",
                path.to_str().unwrap()
            )
            .as_str(),
        );
        private_key =
            age::x25519::Identity::from_str(private_key_string.as_str()).expect(&format!(
                "Failed at reading the private key: {}",
                path.to_str().unwrap()
            ));
        info!("Read the key from: {}", path.as_path().to_str().unwrap());
    } else {
        private_key = age::x25519::Identity::generate();

        fs::write(
            path.as_path(),
            private_key.to_string().expose_secret().as_bytes(),
        )
        .expect(&format!(
            "Failed at reading the private key: {}",
            path.to_str().unwrap()
        ));

        info!("Write the private key to: {}", path.to_str().unwrap());
    }

    return private_key;
}
