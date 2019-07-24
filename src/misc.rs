use regex::Regex;

use bytesize::ByteSize;
use std::path::PathBuf;
use std::process::exit;
use google_drive3::File;

pub const EXIT_CODE_002: i32 = 2;
pub const EXIT_CODE_003: i32 = 3;
pub const EXIT_CODE_001: i32 = 1;
pub const EXIT_CODE_004: i32 = 4;
pub const EXIT_CODE_005: i32 = 5;
pub const EXIT_CODE_006: i32 = 6;
pub const EXIT_CODE_007: i32 = 7;
pub const EXIT_CODE_008: i32 = 8;
pub const EXIT_CODE_009: i32 = 9;


lazy_static! {
static ref RE_BYTE: Regex = Regex::new(r"^(?x)
(?P<size>\d+)  # the year
\s*
(?P<bytetype>[[:alpha:]]*) # the month
$").expect("Something is wrong with your regular expression");
}


pub fn config_file(file: Option<&str>, default: &str) -> PathBuf {
    use std::str::FromStr;

    match file {
        Some(f) => match PathBuf::from_str(f) {
            Ok(p) => p,
            Err(e) => {
                error!("{}", e);
                exit(EXIT_CODE_003)
            }
        },
        None => {
            match dirs::config_dir() {
                Some(mut c) => {
                    c.push("pipe2drive");
                    ::std::fs::create_dir_all(c.as_path()).unwrap_or_else(|e| {
                        error!("{} - {}", e, c.to_string_lossy().to_string());
                        exit(EXIT_CODE_006)
                    });
                    c.push(default);
                    c
                }
                None => {
                    error!("Wasn't able to find the variable XDG_CONFIG_HOME or the folder ~/.config, please use '--client-secret' to select the file");
                    exit(EXIT_CODE_002)
                }
            }
        }
    }
}



pub fn file_filter(regex: &str, files: &Vec<File>) -> Vec<File> {
    let mut filtered_files = Vec::new();

    let re = Regex::new(regex)
        .expect("Something is wrong with your regular expression");

    for file in files {
        if re.is_match(file.name.as_ref().unwrap()) {
            filtered_files.push(file.clone());
        }
    }

    filtered_files
}


pub fn parse_data_size(size: &str) -> ByteSize {
    let caps = match RE_BYTE.captures(size) {
        Some(x) => x,
        None => {
            error!("Unknown/Invalid format for: data size");
            exit(1);
        }
    };

    let bytesize = if &caps["bytetype"] != "" { &caps["bytetype"] } else { "b" };
    let bytesize = bytesize.to_lowercase();

    let size = match bytesize.as_str() {
        "b" => ByteSize::b(caps["size"].parse::<u64>().unwrap()),
        "kb" => ByteSize::kb(caps["size"].parse::<u64>().unwrap()),
        "kib" => ByteSize::kib(caps["size"].parse::<u64>().unwrap()),
        "mb" => ByteSize::mb(caps["size"].parse::<u64>().unwrap()),
        "mib" => ByteSize::mib(caps["size"].parse::<u64>().unwrap()),
        "gb" => ByteSize::gb(caps["size"].parse::<u64>().unwrap()),
        "gib" => ByteSize::gib(caps["size"].parse::<u64>().unwrap()),
        "tb" => ByteSize::tb(caps["size"].parse::<u64>().unwrap()),
        "tib" => ByteSize::tib(caps["size"].parse::<u64>().unwrap()),
        _ => {
            error!("Unknown/Invalid format for: data size");
            exit(1);
        }
    };

    info!("The Size of the uploaded data: {} Bytes", size.as_u64());

    size
}