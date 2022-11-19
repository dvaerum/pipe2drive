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

use arguments::ArgLogLevel;
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
use clap::Parser;



#[tokio::main]
async fn main() {
    let args = arguments::Arguments::parse();

    let log_level = match args.logging {
        ArgLogLevel::Trace => Level::Trace,
        ArgLogLevel::Debug => Level::Debug,
        ArgLogLevel::Info => Level::Info,
        ArgLogLevel::Warn => Level::Warn,
        ArgLogLevel::Error => Level::Error,
    };

    logger::init_with_level(log_level).unwrap();

    let json_output: bool = args.json;

    let hub = auth::auth(
        args.secret,
        args.token,
    );

    match args.command {
        arguments::Commands::Info(info) => {
            let info = drive::info(&hub.await, info.id.as_str()).await;
            misc::print_info(&info, json_output);
            exit(0);
        },
        arguments::Commands::List(list) => {
            let files = drive::list(&hub.await, list.folder).await;
            misc::print_list(files, json_output);
            exit(0);
        },
        arguments::Commands::Download(download) => {
            let hub_tmp = hub.await;

            let info = drive::info(
                &hub_tmp,
                &download.file
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
        },
        arguments::Commands::Upload(upload) => {
            if atty::is(atty::Stream::Stdin) && !upload.testing {
                error!("You need to pipe something to this program otherwise it has nothing to upload");
                exit(misc::EXIT_CODE_001);
            }

            let mut encryption_pub_key = None;
            if upload.encrypt {
                encryption_pub_key = Some(crypto::load_public_key(None))
            }

            let upload_result: drive::UploadResult;
            if upload.testing {
                upload_result = drive::upload::<TestBuffer>(
                    &hub.await,
                    TestBuffer::new(
                        parse_data_size(upload.testing_size.as_str()).as_u64() as usize,
                    ),
                    misc::parse_data_size(upload.size.as_str()).as_u64() as usize,
                    upload.filename,
                    upload.parent_folder,
                    upload.duplicate,
                    upload.replace,
                    encryption_pub_key,
                )
                .await;
            } else {
                upload_result = drive::upload::<StdinWrapperWithSendSupport>(
                    &hub.await,
                    StdinWrapperWithSendSupport::new(),
                    misc::parse_data_size(upload.size.as_str()).as_u64() as usize,
                    upload.filename,
                    upload.parent_folder,
                    upload.duplicate,
                    upload.replace,
                    encryption_pub_key,
                )
                .await;
            }

            misc::print_upload(upload_result, json_output);
            exit(0);
        },
    }
}
