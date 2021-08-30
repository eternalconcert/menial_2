extern crate clap;
extern crate yaml_rust;

use menial_2::config::Config;
use ansi_term::Colour;
use chrono::{DateTime, Utc};
use menial_2::config::{CONFIG};
use menial_2::{log, ThreadPool, LOG_LEVEL};
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::collections::{HashSet};
use openssl::ssl::{SslMethod, SslAcceptor, SslStream, SslFiletype};
use std::sync::Arc;

fn main() {
    let menial_version: &'static str = option_env!("MENIAL_VERSION").unwrap_or("DEV");

    log!("info", format!("Starting menial/2 ({})", menial_version));
    let random_config = CONFIG.values().collect::<Vec<&Config>>()[0];
    log!("info", format!("Config file: {}", random_config.file));

    let mut ports = HashSet::new();

    for (host, value) in CONFIG.iter() {

        ports.insert(value.port.to_owned());
        log!("info", format!("{}: Document root: {}", host, value.root));
        log!("info", format!("{}: Resources root: {}", host, value.port));
    }

    let worker_count: usize = 20;  // What is a sane number for workers?
    log!("info", format!("Using {} workers", worker_count));
    let pool = ThreadPool::new(worker_count);
    for port in ports {
        log!("info", format!("Listening on port: {}", port));
        pool.execute(move || {
            run_server(port.parse::<usize>().unwrap());
        });
    }

    // pool.execute(move || {
    //     run_ssl_server(4433);
    // });
}

fn run_ssl_server(port: usize) {
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    acceptor.set_private_key_file("/tmp/key.pem", SslFiletype::PEM).unwrap();
    acceptor.set_certificate_chain_file("/tmp/cert.pem").unwrap();
    acceptor.check_private_key().unwrap();

    let acceptor = Arc::new(acceptor.build());

    let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", port)).unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let acceptor = acceptor.clone();

        pool.execute(move || {
            let stream = acceptor.accept(stream).unwrap();
            handle_ssl_connection(stream);
        });
    }

}

fn run_server(port: usize) {
    let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", port)).unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(move || {
            handle_connection(stream);
        });
    }
}

const SERVER_LINE: &str = "Server: menial/2";


fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let (response, contents) = handle_request(buffer);

    stream.write(response.as_bytes()).unwrap();
    match stream.write_all(&contents) {
        Ok(_) => {
            log!("debug", "Write content successfull");
        },
        Err(ref _e) => {
            log!("warning", "Could not write buffer!");
            // panic!("could not write buffer!")
        },
    };

    stream.flush().unwrap();
}


fn handle_ssl_connection(mut stream: SslStream<TcpStream>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let (response, contents) = handle_request(buffer);

    stream.write(response.as_bytes()).unwrap();
    match stream.write_all(&contents) {
        Ok(_) => {
            log!("debug", "Write content successfull");
        },
        Err(ref _e) => {
            log!("warning", "Could not write buffer!");
            // panic!("could not write buffer!")
        },
    };

    stream.flush().unwrap();
}



fn handle_request(buffer: [u8; 1024]) -> (String, Vec<u8>) {
    let request_content = String::from_utf8_lossy(&buffer);
    log!("debug", request_content);
    let mut document = String::from("");
    let mut host = String::from("");

    for line in request_content.split("\n") {
        if line.starts_with("GET") || line.starts_with("POST") {
            match line.find("HTTP") {
                Some(v) => document = String::from(&line[4..v - 1]),
                None => {}
            }
        }
        else if line.starts_with("Host:") {
            host = String::from(&line[6..line.len() - 1]);
        }
    }

    if host.find(":").unwrap_or(0) == 0 {
        host = format!("{}:{}", host, "80")

    }

    log!("debug", format!("Requested host: {}", host));

    let mut host_config = CONFIG.values().collect::<Vec<&Config>>()[0];
    match CONFIG.get(&host) {
        Some(conf) => host_config = conf,
        _ => {
            log!("debug", "Host not found");
        },
    }
    let document_root = host_config.root.to_owned();
    let resources_root = host_config.resources.to_owned();

    let mut status: u16 = 0;

    match request_content.find("..") {
        Some(_) => {
            log!(
                "warning",
                format!("Intrusion try detected: {}", request_content)
            );
            status = 400;
        }
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
        }
        400 => {
            status_line = String::from("HTTP/1.1 400 BAD REQUEST");
            filename = String::from(format!("{}/400.html", resources_root));
        }
        404 => {
            status_line = String::from("HTTP/1.1 404 NOT FOUND");
            filename = String::from(format!("{}/404.html", resources_root));
        }
        _ => {}
    }

    let contents = fs::read(filename).unwrap();

    let response = format!(
        "{}\n{}\nContent-Length: {}\r\n\r\n",
        status_line,
        SERVER_LINE,
        contents.len(),
    );
    return (response, contents);
}