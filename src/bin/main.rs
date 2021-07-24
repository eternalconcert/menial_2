extern crate clap;
extern crate yaml_rust;

use menial_2::{log, LOG_LEVEL, ThreadPool};
use menial_2::config::{get_config};
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use chrono::{DateTime, Utc};
use ansi_term::Colour;

fn main() {
    let config = get_config();

    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port)).unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let root = config.root.to_owned();
        let resources = config.resources.to_owned();
        pool.execute(move || {
            handle_connection(stream, &root, &resources);
        });
    }
}

const SERVER_LINE: &str = "Server: menial 2";

fn handle_connection(mut stream: TcpStream, document_root: &str, resources_root: &str) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request_content = String::from_utf8_lossy(&buffer);

    let mut document = String::from("");

    match request_content.find("HTTP") {
        Some(v) => document = String::from(&request_content[4..v-1]),
        None => {}
    }

    let mut status: u16 = 0;

    match request_content.find("..") {
        Some(_) => {
            log!("warning", format!("Intrusion try detected: {}", request_content));
            status = 400;
        },
        None => {}
    }

    if document == "/" {
        document = String::from("/index.html");
    }

    log!("debug", format!("GET-Request: {}", document));

    let doc = String::from(format!("{}{}", document_root, document));
    log!("debug", format!("Requested document path: {}", doc));

    let mut status_line: String = String::from("");
    let mut filename: String = String::from("");
    if Path::new(&doc).exists() && request_content.starts_with("GET") {
        status = 200;
    } else if status == 0 {
        log!("warning", format!("Requested document not found: {}", doc));
        status = 404;
    };

    match status {
        200 => {
            status_line = String::from("HTTP/1.1 200 OK");
            filename = doc;
        },
        400 => {
            status_line = String::from("HTTP/1.1 400 BAD REQUEST");
            filename = String::from(format!("{}/400.html", resources_root));
        },
        404 => {
            status_line = String::from("HTTP/1.1 404 NOT FOUND");
            filename = String::from(format!("{}/404.html", resources_root));
        },
        _ => {}

    }

    let contents = fs::read(filename).unwrap();

    let response = format!(
        "{}\n{}\nContent-Length: {}\r\n\r\n",
        status_line,
        SERVER_LINE,
        contents.len(),
    );

    stream.write(response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();

    stream.flush().unwrap();
}
