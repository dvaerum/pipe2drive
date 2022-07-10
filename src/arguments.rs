extern crate clap;
use clap::{Parser, AppSettings, Subcommand, Args, ArgEnum};

#[derive(Parser, Debug)]
#[clap[name = env!("CARGO_PKG_NAME")]]
#[clap[author = env!("CARGO_PKG_AUTHORS")]]
#[clap[version = env!("CARGO_PKG_VERSION")]]
#[clap[about = "If you pipe data (doesn't matter what data) to this program and then select a name for that data and declare it size, it will be uploaded to Google Drive"]]
#[clap[long_about = None]]
#[clap[setting = AppSettings::SubcommandRequiredElseHelp]]
pub struct Arguments {
    /// Select the file containing the client secret. If you don't have one go here\nhttps://console.developers.google.com/apis/credentials
    #[clap(long)]
    #[clap(value_name = "FILE")]
    pub secret: Option<String>,
    /// Select the file/there the file containing the client token is/should be saved
    #[clap(long)]
    #[clap(value_name = "FILE")]
    pub token: Option<String>,

    /// Select log level (can also configures with the environment variable: RUST_LOG)
    #[clap(arg_enum)]
    #[clap(long)]
    #[clap(value_name = "LEVEL")]
    #[clap(parse(try_from_str = str2logging_level))]
    #[clap(default_value = option_env!("RUST_LOG").unwrap_or("info"))]
    pub logging: ArgLogLevel,

    /// Print json object to stdout
    #[clap(long)]
    pub json: bool,

    /// Upload a file to Google Drive
    #[clap(subcommand)]
    pub command: Commands,
}

#[allow(dead_code)]
fn str2logging_level(s: &str) -> Result<ArgLogLevel, &'static str> {
    Ok(
        match s.to_lowercase().as_str() {
            "trace" => ArgLogLevel::Trace,
            "debug" => ArgLogLevel::Debug,
            "info" => ArgLogLevel::Info,
            "warn" => ArgLogLevel::Warn,
            "warning" => ArgLogLevel::Warn,
            "error" => ArgLogLevel::Error,
            "quiet" => ArgLogLevel::Error,
            _ => {
                ArgLogLevel::Warn
            }
        }
    )
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
pub enum ArgLogLevel {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Subcommand, Debug)]
#[clap[about]]
pub enum Commands {
    /// Get info about ID
    Info(Info),

    /// List files and there ID
    List(List),

    /// Download a file from Google Drive
    Download(Download),

    /// Upload a file to Google Drive
    Upload(Upload),
}

#[derive(Args, Debug)]
pub struct Info {
    /// Provided the ID of the content of that you want more info about
    #[clap(long)]
    #[clap(value_name = "ID")]
    pub id: String
}

#[derive(Args, Debug)]
pub struct List {
    /// If a folder ID is provided the content of that folder will be listed,\notherwise the content of 'My Drive' will be listed
    #[clap(long)]
    #[clap(value_name = "ID")]
    pub folder: Option<String>,
}

#[derive(Args, Debug)]
pub struct Download {
    /// Provided the ID of the file (or one of the split files) you want to download
    #[clap(long)]
    #[clap(value_name = "ID")]
    pub file: String
}

#[derive(Args, Debug)]
pub struct Upload {
    /// The size of the data you want to upload.\nExample: 100mib, 1gb or 1048576 aka. 1mib)\nSupported Sizes: b, kb, kib, mb, mib, gb, gib, tb and tib
    #[clap(long)]
    #[clap(value_name = "size")]
    pub size: String,

    /// The name of the file uploaded to Google Drive
    #[clap(long)]
    #[clap(value_name = "name")]
    #[clap(default_value = "Untitled")]
    pub filename: String,

    /// The ID of the folder where you want the file to be uploaded to.\nIf this is not defined, the file will be uploaded to 'My Drive'
    #[clap(long, value_name = "ID")]
    pub parent_folder: Option<String>,

    /// If a file exists with the same name it will be replaced
    #[clap(long)]
    pub replace: bool,

    /// Allow multiple files to have the same name
    #[clap(long)]
    pub duplicate: bool,

    /// Encrypt the context before uploading it to Google Drive
    #[clap(long)]
    pub encrypt: bool,

    /// Uploading a 100MiB file consisting of the repeated text sequence '0123456789'
    #[clap(long)]
    pub testing: bool,

    /// Uploading a 100MiB file consisting of the repeated text sequence '0123456789'
    #[clap(long)]
    #[clap(value_name = "size")]
    #[clap(default_value_t = String::from("100MiB"))]
    pub testing_size: String,
}
