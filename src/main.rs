#[macro_use]
extern crate log;
extern crate atty;
extern crate bytesize;
#[macro_use]
extern crate lazy_static;
extern crate regex;
#[macro_use]
extern crate prettytable;
extern crate ringbuf;

use pipe_buffer::TestBuffer;
mod arguments;
mod auth;
mod crypto;
mod drive;
mod logger;
mod misc;
mod pipe_buffer;

use crate::misc::{parse_data_size, StdinWrapperWithSendSupport};
use log::Level;
use std::process::exit;



#[tokio::main]
async fn main() {
    let matches = arguments::get_parsed_arguments();

    if matches.is_present("trace") {
        logger::init_with_level(Level::Trace).unwrap();
    } else if matches.is_present("debug") {
        logger::init_with_level(Level::Debug).unwrap();
    } else if matches.is_present("quiet") {
        logger::init_with_level(Level::Error).unwrap()
    } else if matches.is_present("info") {
        logger::init_with_level(Level::Info).unwrap();
    } else if let Some(rust_log) = option_env!("RUST_LOG") {
        logger::init_with_level(match rust_log.to_lowercase().as_str() {
            "trace" => Level::Trace,
            "debug" => Level::Debug,
            "info" => Level::Info,
            "warn" => Level::Warn,
            "warning" => Level::Warn,
            "error" => Level::Error,
            "quiet" => Level::Error,
            _ => {
                println!("The envirement variable `RUST_LOG` is set to `{}` \
                          which is a invalid value, set it to one of the following \
                          values: error, warn, info, debug, trace", rust_log);
                Level::Warn
            }
        }).unwrap();
    } else {
        logger::init_with_level(Level::Warn).unwrap();
    }

    let json_output: bool = matches.is_present("json");

    let hub = auth::auth(
        matches.value_of("client_secret_file"),
        matches.value_of("client_token_file"),
    );

    if let Some(id) = matches.subcommand_matches("info") {
        let info = drive::info(&hub.await, id.value_of("id").unwrap()).await;
        misc::print_info(&info, json_output);
        exit(0);
    }

    if let Some(folder) = matches.subcommand_matches("list") {
        let files = drive::list(&hub.await, folder.value_of("folder_id")).await;
        misc::print_list(files, json_output);
        exit(0);
    }

    if let Some(download) = matches.subcommand_matches("download") {
        // Get info about file
        let hub_tmp = hub.await;

        let info = drive::info(
            &hub_tmp,
            download.value_of("file_id").unwrap()
        ).await;

        // If the file is trashed, don't download
        if info.trashed.is_some() && info.trashed.unwrap() {
            error!("Cannot download the file '{}' because it is trashed",
                   info.name.as_ref().unwrap());
            exit(misc::EXIT_CODE_012)
        }

        if atty::is(atty::Stream::Stdout) {
            let mut file = ::std::fs::File::create(
                &info.name.as_ref().unwrap_or(&"unknown_named_file_from_drive".to_owned())
            ).expect("Unable to open file");
            drive::download(
                &hub_tmp,
                &info,
                Some(&mut file)
            ).await;

        } else {
            let pipe = ::std::io::stdout();
            drive::download(
                &hub_tmp,
                &info,
                Some(&mut pipe.lock())
            ).await;
        }

        exit(0);
    }

    if let Some(upload) = matches.subcommand_matches("upload") {
        if atty::is(atty::Stream::Stdin) && !upload.is_present("testing") {
            error!("You need to pipe something to this program otherwise it has nothing to upload");
            exit(misc::EXIT_CODE_001);
        }

        let mut encryption_pub_key = None;
        if upload.is_present("encrypt") {
            encryption_pub_key = Some(crypto::load_public_key(None))
        }

        let upload_result: drive::UploadResult;
        if upload.is_present("testing") {
            upload_result = drive::upload::<TestBuffer>(
                &hub.await,
                TestBuffer::new(
                    parse_data_size(upload.value_of("testing_size").unwrap()).as_u64() as usize,
                ),
                misc::parse_data_size(upload.value_of("data_size").unwrap()).as_u64() as usize,
                upload.value_of("file_name"),
                upload.value_of("parent_folder_id"),
                upload.is_present("duplicate"),
                upload.is_present("replace"),
                encryption_pub_key,
            )
            .await;
        } else {
            upload_result = drive::upload::<StdinWrapperWithSendSupport>(
                &hub.await,
                StdinWrapperWithSendSupport::new(),
                misc::parse_data_size(upload.value_of("data_size").unwrap()).as_u64() as usize,
                upload.value_of("file_name"),
                upload.value_of("parent_folder_id"),
                upload.is_present("duplicate"),
                upload.is_present("replace"),
                encryption_pub_key,
            )
            .await;
        }

        misc::print_upload(upload_result, json_output);
        exit(0);
    }
}
