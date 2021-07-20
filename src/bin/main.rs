extern crate clap;
extern crate yaml_rust;

use ::menial_2::{log, LOG_LEVEL};
use ::menial_2::ThreadPool;
use clap::{App, Arg};
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use chrono::{DateTime, Utc};
use ansi_term::Colour;
use yaml_rust::{YamlLoader};


fn main() {

    let matches = App::new("Menial 2")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("host")
                .help("The host to run")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("port")
                .help("The port to run")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("root")
                .short("r")
                .long("root")
                .value_name("root")
                .help("The document root")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("resources")
                .short("s")
                .long("resources")
                .value_name("resources")
                .help("The resources directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("file")
                .help("The document conf")
                .takes_value(true),
        )
        .get_matches();


    let host: String;
    let port: String;
    let root: String;
    let resources: String;

    let config_path = matches.value_of("file").unwrap_or("");
    if config_path != "" {
        log!("info", format!("Config file: {}", config_path));

        let yaml_content = fs::read_to_string(config_path).unwrap();

        let docs = YamlLoader::load_from_str(&yaml_content).unwrap();
        let doc = &docs[0];

        host = doc["host"].as_str().unwrap_or("127.0.0.1").to_owned();
        port = doc["port"].as_str().unwrap_or("8080").to_owned();
        root = String::from(doc["root"].as_str().unwrap_or("."));
        resources = String::from(doc["resources"].as_str().unwrap_or("."));

    } else {
        host = matches.value_of("host").unwrap_or("127.0.0.1").to_owned();
        port = matches.value_of("port").unwrap_or("8080").to_owned();
        root = String::from(matches.value_of("root").unwrap_or("default")).to_owned();
        resources = String::from(matches.value_of("resources").unwrap_or("default/pages")).to_owned();
    }

    log!("info", format!("Host: {}", host));
    log!("info", format!("Port: {}", port));
    log!("info", format!("Document root: {}", root));
    log!("info", format!("Resources root: {}", resources));

    let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let dr = root.to_owned();
        let rr = resources.to_owned();
        pool.execute(move || {
            handle_connection(stream, &dr, &rr);
        });
    }
}

const SERVER_LINE: &str = "Server: menial 2";

fn handle_connection(mut stream: TcpStream, document_root: &str, resources_root: &str) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let content = String::from_utf8_lossy(&buffer);

    let mut document = String::from("");

    match content.find("HTTP") {
        Some(v) => document = String::from(&content[4..v-1]),
        None => {}
    }

    match content.find("..") {
        Some(_) => {
            log!("warning", format!("Intrustion try detected: {}", content));
            document = String::from("/");
        },
        None => {}
    }

    if document == "/" {
        document = String::from("/index.html");
    }

    log!("debug", format!("GET-Request: {}", document));

    let doc = String::from(format!("{}{}", document_root, document));
    log!("debug", format!("Requested document path: {}", doc));

    let status_line: String;
    let filename: String;
    if Path::new(&doc).exists() && buffer.starts_with(b"GET") {
        status_line = String::from("HTTP/1.1 200 OK");
        filename = doc;
    } else {
        log!("warning", format!("Requested document not found: {}", doc));
        status_line = String::from("HTTP/1.1 404 NOT FOUND");
        filename = String::from(format!("{}/404.html", resources_root));
    };

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
