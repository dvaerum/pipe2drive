#[macro_use]
extern crate log;
extern crate clap;
extern crate bytesize;
extern crate atty;
#[macro_use]
extern crate lazy_static;
extern crate regex;
#[macro_use]
extern crate prettytable;

use prettytable::{Table};

mod pipe_buffer;
mod auth;
mod drive;
mod misc;
mod logger;

use log::Level;
use clap::{App, Arg, SubCommand, AppSettings};
use crate::misc::print_info;
use std::process::exit;

fn main() {
    let matches = App::new("Pipe2Drive")
        .version(env!("CARGO_PKG_VERSION"))
        .about("If you pipe data (doesn't matter what data) to this program and then select a name for that data and declare it size, it will be uploaded to Google Drive")
        .setting(AppSettings::SubcommandRequiredElseHelp)
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
        .arg(Arg::with_name("debug")
            .long("debug")
            .help("Will display Debug and Info logs"))
        .arg(Arg::with_name("info")
            .long("info")
            .help("Will display Info logs"))
        .arg(Arg::with_name("quiet")
            .long("quiet")
            .help("Will only display Error logs"))
        .subcommand(SubCommand::with_name("upload")
            .about("Upload a file to Google Drive")
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
            .arg(Arg::with_name("replace")
                .long("replace")
                .conflicts_with("duplicate")
                .help("If a file exists with the same name it will be replaced"))
            .arg(Arg::with_name("duplicate")
                .long("duplicate")
                .conflicts_with("replace")
                .help("Allow multiple files to have the same name")))
        .subcommand(SubCommand::with_name("list")
            .about("List files and there ID")
            .arg(Arg::with_name("folder_id")
                .long("folder")
                .value_name("ID")
                .help("If a folder ID is provided the content of that folder will be listed,\notherwise the content of 'My Drive' will be listed")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("info")
            .about("Get info about ID")
            .arg(Arg::with_name("id")
                .value_name("ID")
                .required(true)
                .help("Provided the ID of the content of that you want more info about")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("download")
            .about("Download a file from Google Drive")
            .arg(Arg::with_name("file_id")
                .long("file")
                .value_name("ID")
                .required(true)
                .help("Provided the ID of the file (or one of the split files) you want to download")
                .takes_value(true)))
        .get_matches();

    if matches.is_present("debug") {
        logger::init_with_level(Level::Debug).unwrap();
    } else if matches.is_present("quiet") {
        logger::init_with_level(Level::Error).unwrap()
    } else if matches.is_present("info") {
        logger::init_with_level(Level::Info).unwrap();
    } else {
        logger::init_with_level(Level::Warn).unwrap();
    }

    let hub = auth::auth(
        matches.value_of("client_secret_file"),
        matches.value_of("client_token_file"),
    );

    if let Some(id) = matches.subcommand_matches("info") {
        let info = drive::info(&hub,id.value_of("id").unwrap());
        print_info(&info);
        exit(0);
    }

    if let Some(folder) = matches.subcommand_matches("list") {
        let files = drive::list(&hub, folder.value_of("folder_id"));

        let mut table = Table::new();
        table.set_titles(row!["Type", "Name", "ID"]);
        table.set_format(*prettytable::format::consts::FORMAT_CLEAN);

        for file in files {
            table.add_row(row![
                     if file.mime_type.unwrap() == "application/vnd.google-apps.folder" { "Folder" } else { "File  " },
                     file.name.unwrap(),
                     file.id.unwrap()
            ]);
        }
        table.printstd();
        exit(0);
    }

    if let Some(id) = matches.subcommand_matches("download") {
        drive::download(&hub,id.value_of("file_id").unwrap());
        exit(0);
    }

    if let Some(upload) = matches.subcommand_matches("upload") {
        if atty::is(atty::Stream::Stdin) {
            error!("You need to pipe something to this program otherwise it has nothing to upload");
            exit(misc::EXIT_CODE_001);
        }

        drive::upload(
            &hub,
            misc::parse_data_size(upload.value_of("data_size").unwrap()).as_u64() as usize,
            upload.value_of("file_name"),
            upload.value_of("parent_folder_id"),
            upload.is_present("duplicate"),
            upload.is_present("replace"),
        );
        exit(0);
    }
}