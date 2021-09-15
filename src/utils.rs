use crate::config::CONFIG;
use std::collections::HashSet;
use chrono::{DateTime, Utc};
use ansi_term::Colour;
use sha2::{Digest, Sha256};
use std::fs;

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

pub fn get_response_headers(contents: &std::vec::Vec<u8>, filename: &str) -> (String, String, String) {
    let content_length = format!("Content-Length: {}", contents.len());

    let hash = format!("{:x}", Sha256::digest(&contents));
    let etag = format!("ETag: \"{}\"", hash);
    log!("debug", format!("File hash: {}", hash));

    let modified: DateTime<Utc> = fs::metadata(&filename).unwrap().modified().unwrap().into();
    let modified_formatted = format!(
        "Last-Modified: {}",
        modified.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    );

    return (content_length, etag, modified_formatted);
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
