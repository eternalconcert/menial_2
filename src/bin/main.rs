extern crate clap;
extern crate yaml_rust;
extern crate num_cpus;

use menial_2::config::HostConfig;
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

fn get_ports() -> HashSet<String> {
    let mut ports = HashSet::new();
    for (host, value) in CONFIG.host_configs.iter() {

        ports.insert(value.port.to_owned());
        log!("info", format!("{}: Document root: {}", host, value.root));
        log!("info", format!("{}: Resources root: {}", host, value.port));
    }
    return ports;
}

fn get_ssl_ports() -> HashSet<String> {
    let mut ssl_port = HashSet::new();
    for (port, _) in CONFIG.ssl_config.iter() {

        ssl_port.insert(port.to_owned());
        // log!("info", format!("{}: Document root: {}", host, value.root));
        // log!("info", format!("{}: Resources root: {}", host, value.port));
    }
    return ssl_port;
}

fn main() {
    let menial_version: &'static str = option_env!("MENIAL_VERSION").unwrap_or("DEV");

    log!("info", format!("Starting menial/2 ({})", menial_version));
    log!("info", format!("Config file: {}", CONFIG.file));

    let ports = get_ports();
    let ssl_ports = get_ssl_ports();

    let worker_count: usize = num_cpus::get();
    log!("info", format!("Using {} workers", worker_count));
    let pool = ThreadPool::new(worker_count);
    for port in ports {
        if ssl_ports.contains(&port) {
            pool.execute(move || {
                log!("info", format!("Listening on ssl port: {}", port));
                run_ssl_server(port.parse::<usize>().unwrap());
            });
        } else {
            pool.execute(move || {
                log!("info", format!("Listening on normal port: {}", port));
                run_server(port.parse::<usize>().unwrap());
            });            
        }
        
    }
}

fn run_ssl_server(port: usize) {
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    let key = &CONFIG.ssl_config[&port.to_string()].key;
    let cert = &CONFIG.ssl_config[&port.to_string()].cert;
    acceptor.set_private_key_file(key, SslFiletype::PEM).unwrap();
    acceptor.set_certificate_chain_file(cert).unwrap();
    acceptor.check_private_key().unwrap();

    let acceptor = Arc::new(acceptor.build());

    let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", port)).unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let acceptor = acceptor.clone();

        pool.execute(move || {
            match acceptor.accept(stream) {
                Ok(stream) => {
                    handle_ssl_connection(stream);
                }
                Err(_) => {
                    log!("debug", "Not an HTTPS request");
                }
            }
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

    let mut host_config = CONFIG.host_configs.values().collect::<Vec<&HostConfig>>()[0];
    match CONFIG.host_configs.get(&host) {
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