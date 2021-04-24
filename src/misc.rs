use regex::Regex;

use bytesize::ByteSize;
use std::path::PathBuf;
use std::process::exit;
use google_drive3::api::File;
use prettytable::Table;

pub const EXIT_CODE_001: i32 = 01;
pub const EXIT_CODE_002: i32 = 02;
pub const EXIT_CODE_003: i32 = 03;
pub const EXIT_CODE_004: i32 = 04;
pub const EXIT_CODE_005: i32 = 05;
pub const EXIT_CODE_006: i32 = 06;
pub const EXIT_CODE_007: i32 = 07;
pub const EXIT_CODE_008: i32 = 08;
pub const EXIT_CODE_009: i32 = 09;
pub const EXIT_CODE_010: i32 = 10;
pub const EXIT_CODE_011: i32 = 11;
pub const EXIT_CODE_012: i32 = 12;
pub const EXIT_CODE_013: i32 = 13;
pub const EXIT_CODE_014: i32 = 14;


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

pub fn print_info(file: &File) {
    let mut table = Table::new();
    table.set_titles(row!["Variable", "Value"]);
    table.set_format(*prettytable::format::consts::FORMAT_CLEAN);

    if let Some(ref v) = file.has_thumbnail { table.add_row(row!["has_thumbnail".to_owned(), v]); }
    if let Some(ref v) = file.mime_type { table.add_row(row!["mime_type".to_owned(), v]); }
    if let Some(ref v) = file.modified_by_me_time { table.add_row(row!["modified_by_me_time".to_owned(), v]); }
    if let Some(ref v) = file.thumbnail_link { table.add_row(row!["thumbnail_link".to_owned(), v]); }
    if let Some(ref v) = file.thumbnail_version { table.add_row(row!["thumbnail_version".to_owned(), v]); }
    if let Some(ref v) = file.explicitly_trashed { table.add_row(row!["explicitly_trashed".to_owned(), v]); }
    if let Some(ref v) = file.is_app_authorized { table.add_row(row!["is_app_authorized".to_owned(), v]); }
    if let Some(ref v) = file.writers_can_share { table.add_row(row!["writers_can_share".to_owned(), v]); }
    if let Some(ref v) = file.owned_by_me { table.add_row(row!["owned_by_me".to_owned(), v]); }
    if let Some(ref v) = file.viewed_by_me_time { table.add_row(row!["viewed_by_me_time".to_owned(), v]); }
    if let Some(ref v) = file.id { table.add_row(row!["id".to_owned(), v]); }
//    if let Some(ref v) = file.sharing_user { table.add_row(row!["sharing_user".to_owned(), v]); }
    if let Some(ref v) = file.size { table.add_row(row!["size".to_owned(), v]); }
//    if let Some(ref v) = file.video_media_metadata { table.add_row(row!["video_media_metadata".to_owned(), v]); }
//    if let Some(ref v) = file.last_modifying_user { table.add_row(row!["last_modifying_user".to_owned(), v]); }
    if let Some(ref v) = file.folder_color_rgb { table.add_row(row!["folder_color_rgb".to_owned(), v]); }
//    if let Some(ref v) = file.app_properties { table.add_row(row!["app_properties".to_owned(), v]); }
//    if let Some(ref v) = file.capabilities { table.add_row(row!["capabilities".to_owned(), v]); }
//    if let Some(ref v) = file.properties { table.add_row(row!["properties".to_owned(), v]); }
    if let Some(ref v) = file.web_view_link { table.add_row(row!["web_view_link".to_owned(), v]); }
    if let Some(ref v) = file.version { table.add_row(row!["version".to_owned(), v]); }
    if let Some(ref v) = file.parents { table.add_row(row!["parents".to_owned(), v.join(",")]); }
//    if let Some(ref v) = file.md { table.add_row(row!["md".to_owned(), v]); }
//    if let Some(ref v) = file.export_links { table.add_row(row!["export_links".to_owned(), v]); }
    if let Some(ref v) = file.shared { table.add_row(row!["shared".to_owned(), v]); }
    if let Some(ref v) = file.copy_requires_writer_permission { table.add_row(row!["copy_requires_writer_permission".to_owned(), v]); }
    if let Some(ref v) = file.full_file_extension { table.add_row(row!["full_file_extension".to_owned(), v]); }
    if let Some(ref v) = file.original_filename { table.add_row(row!["original_filename".to_owned(), v]); }
//    if let Some(ref v) = file.image_media_metadata { table.add_row(row!["image_media_metadata".to_owned(), v]); }
    if let Some(ref v) = file.description { table.add_row(row!["description".to_owned(), v]); }
    if let Some(ref v) = file.modified_time { table.add_row(row!["modified_time".to_owned(), v]); }
    if let Some(ref v) = file.viewed_by_me { table.add_row(row!["viewed_by_me".to_owned(), v]); }
    if let Some(ref v) = file.modified_by_me { table.add_row(row!["modified_by_me".to_owned(), v]); }
    if let Some(ref v) = file.kind { table.add_row(row!["kind".to_owned(), v]); }
    if let Some(ref v) = file.created_time { table.add_row(row!["created_time".to_owned(), v]); }
    if let Some(ref v) = file.quota_bytes_used { table.add_row(row!["quota_bytes_used".to_owned(), v]); }
    if let Some(ref v) = file.team_drive_id { table.add_row(row!["team_drive_id".to_owned(), v]); }
    if let Some(ref v) = file.trashed_time { table.add_row(row!["trashed_time".to_owned(), v]); }
    if let Some(ref v) = file.shared_with_me_time { table.add_row(row!["shared_with_me_time".to_owned(), v]); }
    if let Some(ref v) = file.icon_link { table.add_row(row!["icon_link".to_owned(), v]); }
    if let Some(ref v) = file.viewers_can_copy_content { table.add_row(row!["viewers_can_copy_content".to_owned(), v]); }
//    if let Some(ref v) = file.owners { table.add_row(row!["owners".to_owned(), v]); }
    if let Some(ref v) = file.name { table.add_row(row!["name".to_owned(), v]); }
    if let Some(ref v) = file.web_content_link { table.add_row(row!["web_content_link".to_owned(), v]); }
//    if let Some(ref v) = file.trashing_user { table.add_row(row!["trashing_user".to_owned(), v]); }
    if let Some(ref v) = file.drive_id { table.add_row(row!["drive_id".to_owned(), v]); }
//    if let Some(ref v) = file.spaces { table.add_row(row!["spaces".to_owned(), v]); }
//    if let Some(ref v) = file.permission_ids { table.add_row(row!["permission_ids".to_owned(), v]); }
    if let Some(ref v) = file.trashed { table.add_row(row!["trashed".to_owned(), v]); }
//    if let Some(ref v) = file.content_hints { table.add_row(row!["content_hints".to_owned(), v]); }
    if let Some(ref v) = file.file_extension { table.add_row(row!["file_extension".to_owned(), v]); }
    if let Some(ref v) = file.has_augmented_permissions { table.add_row(row!["has_augmented_permissions".to_owned(), v]); }
    if let Some(ref v) = file.starred { table.add_row(row!["starred".to_owned(), v]); }
    if let Some(ref v) = file.head_revision_id { table.add_row(row!["head_revision_id".to_owned(), v]); }
//    if let Some(ref v) = file.permissions { table.add_row(row!["permissions".to_owned(), v]); }
    if let Some(ref v) = file.md5_checksum { table.add_row(row!["md5Checksum".to_owned(), v]); }

    table.printstd();
}