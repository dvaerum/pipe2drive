extern crate clap;

use clap::{Parser, Subcommand, Args, ValueEnum, builder::PossibleValue};

#[derive(Parser, Debug)]
#[clap[name = env ! ("CARGO_PKG_NAME")]]
#[clap[author = env ! ("CARGO_PKG_AUTHORS")]]
#[clap[version = env ! ("CARGO_PKG_VERSION")]]
#[clap[about = "If you pipe data (doesn't matter what data) to this program and then select a name for that data and declare it size, it will be uploaded to Google Drive"]]
#[clap[long_about = None]]
pub struct Arguments {
    /// Select the FILE containing the client secret. If you don't have one go here\nhttps://console.developers.google.com/apis/credentials
    #[clap(long)]
    #[clap(value_name = "FILE")]
    pub secret: Option<String>,
    /// Select the FILE/there the file containing the client token is/should be saved
    #[clap(long)]
    #[clap(value_name = "FILE")]
    pub token: Option<String>,

    /// Select log LEVEL (can also configures with the environment variable: RUST_LOG)
    #[clap(value_enum)]
    #[clap(long)]
    #[clap(value_name = "LEVEL")]
    #[clap(default_value = option_env ! ("RUST_LOG").unwrap_or("info"))]
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
            "t" => ArgLogLevel::Trace,

            "debug" => ArgLogLevel::Debug,
            "d" => ArgLogLevel::Debug,

            "info" => ArgLogLevel::Info,
            "i" => ArgLogLevel::Info,

            "warning" => ArgLogLevel::Warn,
            "warn" => ArgLogLevel::Warn,
            "w" => ArgLogLevel::Warn,

            "error" => ArgLogLevel::Error,
            "err" => ArgLogLevel::Error,
            "e" => ArgLogLevel::Error,
            "quiet" => ArgLogLevel::Error,
            _ => {
                ArgLogLevel::Warn
            }
        }
    )
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArgLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl ValueEnum for ArgLogLevel {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            ArgLogLevel::Trace,
            ArgLogLevel::Debug,
            ArgLogLevel::Info,
            ArgLogLevel::Warn,
            ArgLogLevel::Error,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            ArgLogLevel::Trace => PossibleValue::new("trace").aliases(["t"]),
            ArgLogLevel::Debug => PossibleValue::new("debug").aliases(["d"]),
            ArgLogLevel::Info => PossibleValue::new("info").aliases(["i"]),
            ArgLogLevel::Warn => PossibleValue::new("warn").aliases(["warn", "w"]),
            ArgLogLevel::Error => PossibleValue::new("error").aliases(["err", "e", "quiet"]),
        })
    }
}

#[derive(Subcommand, Debug)]
#[clap[about]]
pub enum Commands {
    /// Get info about ID
    #[command(arg_required_else_help = true)]
    Info(Info),

    /// List files and there ID
    #[command(arg_required_else_help = false)]
    List(List),

    /// Download a file from Google Drive
    #[command(arg_required_else_help = true)]
    Download(Download),

    /// Upload a file to Google Drive
    #[command(arg_required_else_help = true)]
    Upload(Upload),
}

#[derive(Args, Debug)]
pub struct Info {
    /// Provided the ID of the content of that you want more info about
    #[clap(long)]
    #[clap(value_name = "ID")]
    pub id: String,
}

#[derive(Args, Debug)]
pub struct List {
    /// If a folder ID is provided the content of that folder will be listed, otherwise the content of 'My Drive' will be listed
    #[clap(long)]
    #[clap(value_name = "ID")]
    pub folder: Option<String>,
}

#[derive(Args, Debug)]
pub struct Download {
    /// Provided the ID of the file (or one of the split files) you want to download
    #[clap(long)]
    #[clap(value_name = "ID")]
    pub file: String,
}

#[derive(Args, Debug)]
pub struct Upload {
    /// The SIZE of the data you want to upload.
    /// Example: 100mib, 1gb or 1048576 (aka. 1mib)
    /// Supported Sizes: b, kb, kib, mb, mib, gb, gib, tb & tib
    #[clap(long)]
    #[clap(value_name = "SIZE")]
    #[clap(verbatim_doc_comment)]
    pub size: String,

    /// The NAME of the file uploaded to Google Drive
    #[clap(long)]
    #[clap(value_name = "NAME")]
    #[clap(default_value = "Untitled")]
    pub filename: String,

    /// The ID of the folder where you want the file to be uploaded to.
    /// If this is not defined, the file will be uploaded to 'My Drive'
    #[clap(long)]
    #[clap(value_name = "ID")]
    #[clap(verbatim_doc_comment)]
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

    /// Uploading a test file consisting of the repeated text sequence '0123456789'
    #[clap(long)]
    pub testing: bool,

    /// The SIZE of the uploaded text-file
    #[clap(long)]
    #[clap(value_name = "SIZE")]
    #[clap(default_value_t = String::from("100MiB"))]
    pub testing_size: String,
}
