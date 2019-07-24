#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate clap;
extern crate bytesize;
extern crate atty;
#[macro_use]
extern crate lazy_static;
extern crate regex;

mod pipe_buffer;
mod auth;
mod drive;
mod misc;

use log::Level;
use clap::{App, Arg};

fn main() {
    simple_logger::init_with_level(Level::Info).unwrap();

    let matches = App::new("Pipe2Google")
        .version(env!("CARGO_PKG_VERSION"))
        .about("If you pipe data (doesn't matter what data) to this program and then select a name for that data and declare it size, it will be uploaded to Google Drive")
        .arg(Arg::with_name("data_size")
            .value_name("size")
            .help("The size of the data you want to upload.\nExample: 100mib, 1gb or 1048576 aka. 1mib)\nSupported Sizes: b, kb, kib, mb, mib, gb, gib, tb and tib")
            .required(true)
            .index(1))
        .arg(Arg::with_name("file_name")
            .short("n")
            .long("name")
            .value_name("FILE NAME")
            .help("The name of the file uploaded to Google Drive")
            .takes_value(true))
        .arg(Arg::with_name("parent_folder_id")
            .long("folder")
            .value_name("ID")
            .help("The ID of the folder where you want the file to be uploaded to.\nIf this is not defined, the file will be uploaded to 'My Drive'")
            .takes_value(true))
        .arg(Arg::with_name("client_secret_file")
            .long("secret")
            .value_name("FILE")
            .help("Select the file containing the client secret. If you don't have one go here\nhttps://console.developers.google.com/apis/credentials")
            .takes_value(true))
        .arg(Arg::with_name("client_token_file")
            .long("token")
            .value_name("FILE")
            .help("Select the file/there the file containing the client token is/should be saved")
            .takes_value(true))
        .arg(Arg::with_name("replace")
            .long("replace")
            .conflicts_with("duplicate")
            .help("If a file exists with the same name it will be replaced"))
        .arg(Arg::with_name("duplicate")
            .long("duplicate")
            .conflicts_with("replace")
            .help("Allow multiple files to have the same name"))
        .get_matches();


    if atty::is(atty::Stream::Stdin) {
        error!("You need to pipe something to this program otherwise it has nothing to upload");
        ::std::process::exit(misc::EXIT_CODE_001);
    }

    let hub = auth::auth(
        matches.value_of("client_secret_file"),
        matches.value_of("client_token_file"),
    );

    let data_size = misc::parse_data_size(matches.value_of("data_size").unwrap()).as_u64();
    drive::upload(
        &hub,
        data_size as usize,
        matches.value_of("file_name"),
        matches.value_of("parent_folder_id"),
        matches.is_present("duplicate"),
        matches.is_present("replace")
    )
}