#[macro_use]
extern crate log;
extern crate atty;
extern crate bytesize;
#[macro_use]
extern crate lazy_static;
extern crate regex;
#[macro_use]
extern crate prettytable;

use pipe_buffer::TestBuffer;
use prettytable::Table;

mod arguments;
mod auth;
mod crypto;
mod drive;
mod logger;
mod misc;
mod pipe_buffer;

use crate::misc::print_info;
use log::Level;
use std::{
    io::{stdin, StdinLock},
    process::exit,
};

#[tokio::main]
async fn main() {
    let matches = arguments::get_parsed_arguments();

    if matches.is_present("debug") {
        logger::init_with_level(Level::Debug).unwrap();
    } else if matches.is_present("quiet") {
        logger::init_with_level(Level::Error).unwrap()
    } else if matches.is_present("info") {
        logger::init_with_level(Level::Info).unwrap();
    } else {
        logger::init_with_level(Level::Warn).unwrap();
    }
    crypto::load_public_key();
    let hub = auth::auth(
        matches.value_of("client_secret_file"),
        matches.value_of("client_token_file"),
    );

    if let Some(id) = matches.subcommand_matches("info") {
        let info = drive::info(&hub.await, id.value_of("id").unwrap()).await;
        print_info(&info);
        exit(0);
    }

    if let Some(folder) = matches.subcommand_matches("list") {
        let files = drive::list(&hub.await, folder.value_of("folder_id")).await;

        let mut table = Table::new();
        table.set_titles(row!["Type", "Name", "ID"]);
        table.set_format(*prettytable::format::consts::FORMAT_CLEAN);

        for file in files {
            table.add_row(row![
                if file.mime_type.unwrap() == "application/vnd.google-apps.folder" {
                    "Folder"
                } else {
                    "File  "
                },
                file.name.unwrap(),
                file.id.unwrap()
            ]);
        }
        table.printstd();
        exit(0);
    }

    if let Some(download) = matches.subcommand_matches("download") {
        drive::download(&hub.await, download.value_of("file_id").unwrap()).await;
        exit(0);
    }

    if let Some(upload) = matches.subcommand_matches("upload") {
        if atty::is(atty::Stream::Stdin) && !upload.is_present("testing") {
            error!("You need to pipe something to this program otherwise it has nothing to upload");
            exit(misc::EXIT_CODE_001);
        }

        let mut encryption_pub_key = None;
        if upload.is_present("encrypt") {
            encryption_pub_key = Some(crypto::load_public_key())
        }

        if upload.is_present("testing") {
            drive::upload::<TestBuffer>(
                &hub.await,
                TestBuffer::new(1024 * 1024 * 100),
                misc::parse_data_size(upload.value_of("data_size").unwrap()).as_u64() as usize,
                upload.value_of("file_name"),
                upload.value_of("parent_folder_id"),
                upload.is_present("duplicate"),
                upload.is_present("replace"),
                encryption_pub_key,
            )
            .await;
        } else {
            drive::upload::<StdinLock>(
                &hub.await,
                stdin().lock(),
                misc::parse_data_size(upload.value_of("data_size").unwrap()).as_u64() as usize,
                upload.value_of("file_name"),
                upload.value_of("parent_folder_id"),
                upload.is_present("duplicate"),
                upload.is_present("replace"),
                encryption_pub_key,
            )
            .await;
        }

        exit(0);
    }
}
