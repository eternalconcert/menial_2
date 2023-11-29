use crate::config::CONFIG;
use std::collections::HashSet;
use chrono::{DateTime, NaiveDateTime, Utc};
use ansi_term::Colour;
use std::fs;
use sha2::{Digest, Sha256};

use crate::{log, LOG_LEVEL};

const SERVER_LINE: &str = "Server: menial/2";

pub fn get_base_headers() -> String {
    let now: DateTime<Utc> = Utc::now();
    let date = format!(
        "Date: {}",
        now.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    );

    return format!("{}\n{}", SERVER_LINE, date);
}


fn get_file_name_extension(filename: &String) -> String {
    return filename.split('.').last().unwrap_or("").to_string();
}


fn get_content_type(filename_extension: String) -> String {
    let res;

    match filename_extension.as_str() {
        "html" => res = "text/html",
        "htm" => res = "text/html",
        "css" => res = "text/css",
        "gif" => res = "image/gif",
        "mp3" => res = "audio/mpeg",
        "mp4" => res = "audio/mp4",
        "jpg" => res = "image/jpeg",
        "jpeg" => res = "image/jpeg",
        "js" => res = "text/javascript",
        "json" => res = "application/json",
        "pdf" => res = "application/pdf",
        "png" => res = "image/png",
        "svg" => "image/svg+xml",
        "ttf" => res = "font/ttf",
        "txt" => res = "text/txt",
        "woff" => res = "font/woff",
        "woff2" => res = "font/woff2",
        "xml" => res = "application/xml",
        "zip" => res = "application/zip",
        _ => res = "text/plain"
    };

    return res.to_string();
}

pub fn get_response_headers(contents: &std::vec::Vec<u8>, modified: DateTime<Utc>, hash: &String, filename: &String) -> (String, String, String, String) {
    let content_length = format!("Content-Length: {}", contents.len());

    let etag = format!("ETag: \"{}\"", hash);
    log!("debug", format!("File hash: {}", hash));

    let modified_formatted = format!(
        "Last-Modified: {}",
        modified.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    );

    let extension = get_file_name_extension(filename);

    let content_type = format!("Content-Type: {}", get_content_type(extension));

    return (content_length, etag, modified_formatted, content_type);
}

pub fn get_ports() -> HashSet<String> {
    let mut ports = HashSet::new();
    for (host, value) in CONFIG.host_configs.iter() {
        ports.insert(value.port.to_owned());
        log!("info", format!("{}: Document root: {}", host, value.root));
        log!(
            "info",
            format!("{}: Resources root: {}", host, value.resources)
        );
    }
    return ports;
}

pub fn get_ssl_ports() -> HashSet<String> {
    let mut ssl_port = HashSet::new();
    for (port, _) in CONFIG.ssl_config.iter() {
        ssl_port.insert(port.to_owned());
    }
    return ssl_port;
}

pub fn make304(modified: String, etag: String) -> String {
    return format!(
        "{}\n{}\n{}\n{}\r\n\r\n",
        "HTTP/1.1 304 Not Modified", get_base_headers(), modified, etag
    );
}

pub fn intrusion_try_detected(request_content: String) -> bool {
    let double_dots: Vec<_> = request_content.match_indices("..").collect();
    let double_slashes: Vec<_> = request_content.match_indices("//").collect();
    return (double_slashes.len() > 1) || (double_dots.len() > 0);
}

pub fn get_file_data(filename: &String) -> (Vec<u8>, chrono::DateTime<chrono::Utc>, chrono::NaiveDateTime, String) {
    let contents = fs::read(&filename).unwrap();

    let file_modified: DateTime<Utc> = fs::metadata(&filename).unwrap().modified().unwrap().into();
    let modified_short = NaiveDateTime::parse_from_str(
        &file_modified.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
        "%a, %d %b %Y %H:%M:%S GMT",
    )
    .unwrap();

    let hash = format!("{:x}", Sha256::digest(&contents));

    return (contents, file_modified, modified_short, hash);
}

pub fn get_redirect_response(permanent: bool, to: String, base_headers: String) -> (String, Vec<u8>) {
    let status_line;
    if permanent {
        status_line = "HTTP/1.1 301 Moved Permanently";
    } else {
        status_line = "HTTP/1.1 302 Found";
    }
    let headers = format!(
        "{}\nLocation: {}\n{}\r\n\r\n",
        status_line, to, base_headers,
    );
    return (headers, Vec::new());
}

pub fn get_default_status_page(status: u16) -> (Vec<u8>, chrono::DateTime<chrono::Utc>, chrono::NaiveDateTime, String) {

    let file_modified = chrono::offset::Utc::now();

    let modified_short = NaiveDateTime::parse_from_str(
        &file_modified.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
        "%a, %d %b %Y %H:%M:%S GMT",
    ).unwrap();

    return (String::from(format!("{} - Menial 2", get_status_part(status))).as_bytes().to_vec(), file_modified, modified_short, String::from(""))
}

pub fn get_status_line(status: u16) -> String {
    return String::from(format!("HTTP/1.1 {}", get_status_part(status)));
}

fn get_status_part(status: u16) -> String {
    let status_part;
    match status {
        200 => {
            status_part = String::from("200 OK");
        }
        400 => {
            status_part = String::from("400 Bad Request");
        }
        401 => {
            status_part = String::from("401 Unauthorized");
        }
        404 => {
            status_part = String::from("404 Not Found");
        }
        _ => {
            panic!("Unknown Status: {}", status);
        }
    }
    return status_part;
}
