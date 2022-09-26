use crate::config::CONFIG;
use std::collections::HashSet;
use chrono::{DateTime, Utc};
use ansi_term::Colour;

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
        "ttf" => res = "font/ttf",
        "txt" => res = "text/txt",
        "woff" => res = "font/woff",
        "woff2" => res = "font/woff2",
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
