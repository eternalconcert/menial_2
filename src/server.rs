use crate::utils::{get_base_headers, get_response_headers};
use crate::config::{HostConfig, CONFIG};
use crate::{log, ThreadPool, LOG_LEVEL};

use ansi_term::Colour;
use chrono::{DateTime, NaiveDateTime, Utc};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslStream};
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;


pub fn run_ssl_server(port: usize) {
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    let key = &CONFIG.ssl_config[&port.to_string()].key;
    let cert = &CONFIG.ssl_config[&port.to_string()].cert;
    acceptor
        .set_private_key_file(key, SslFiletype::PEM)
        .unwrap();
    acceptor.set_certificate_chain_file(cert).unwrap();
    acceptor.check_private_key().unwrap();

    let acceptor = Arc::new(acceptor.build());

    let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", port)).unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let acceptor = acceptor.clone();

        pool.execute(move || match acceptor.accept(stream) {
            Ok(stream) => {
                handle_ssl_connection(stream);
            }
            Err(_) => {
                log!("debug", "Not an HTTPS request");
            }
        });
    }
}

pub fn run_server(port: usize) {
    let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", port)).unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(move || {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let (headers, contents) = handle_request(buffer, "80");

    stream.write(headers.as_bytes()).unwrap();
    match stream.write_all(&contents) {
        Ok(_) => {
            log!("debug", "Write content successfull");
        }
        Err(ref _e) => {
            log!("warning", "Could not write buffer!");
            // panic!("could not write buffer!")
        }
    };

    stream.flush().unwrap();
}

fn handle_ssl_connection(mut stream: SslStream<TcpStream>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let (response, contents) = handle_request(buffer, "443");

    stream.write(response.as_bytes()).unwrap();
    match stream.write_all(&contents) {
        Ok(_) => {
            log!("debug", "Write content successfull");
        }
        Err(ref _e) => {
            log!("warning", "Could not write buffer!");
            // panic!("could not write buffer!")
        }
    };

    stream.flush().unwrap();
}

fn handle_request(buffer: [u8; 1024], default_port: &str) -> (String, Vec<u8>) {
    let request_content = String::from_utf8_lossy(&buffer);
    log!("debug", request_content);
    let mut document = String::from("");
    let mut host = String::from("");
    let mut modified_since = String::from("");
    // let mut none_match = String::from("");
    for line in request_content.split("\n") {
        if line.starts_with("GET") || line.starts_with("POST") {
            match line.find("HTTP") {
                Some(v) => document = String::from(&line[4..v - 1]),
                None => {}
            }
        } else if line.starts_with("If-Modified-Since:") {
            modified_since = String::from(&line[19..line.len() - 1]);
        // } else if line.starts_with("If-None-Match:") {
        //     none_match = String::from(&line[14..line.len() - 1]);
        } else if line.starts_with("Host:") {
            host = String::from(&line[6..line.len() - 1]);
        }
    }

    // log!("warning", none_match);

    if host.find(":").unwrap_or(0) == 0 {
        host = format!("{}:{}", host, default_port)
    }

    log!("debug", format!("Requested host: {}", host));

    let mut host_config = CONFIG.host_configs.values().collect::<Vec<&HostConfig>>()[0];

    match CONFIG.host_configs.get(&host) {
        Some(conf) => host_config = conf,
        _ => {
            log!("debug", "Host not found");
        }
    }

    let base_headers = get_base_headers();

    if host_config.redirect_to != "" {
        let status_line;
        if host_config.redirect_permanent {
            status_line = "HTTP/1.1 301 Moved Permanently";
        } else {
            status_line = "HTTP/1.1 302 Found";
        }
        let headers = format!(
            "{}\nLocation: {}\n{}\r\n\r\n",
            status_line, host_config.redirect_to, base_headers,
        );
        return (headers, Vec::new());
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

    let contents = fs::read(&filename).unwrap();

    let modified: DateTime<Utc> = fs::metadata(&filename).unwrap().modified().unwrap().into();
    let modified_short = NaiveDateTime::parse_from_str(
        &modified.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
        "%a, %d %b %Y %H:%M:%S GMT",
    )
    .unwrap();

    let (content_length, etag, modified) = get_response_headers(&contents, &filename);

    let headers = format!(
        "{}\n{}\n{}\n{}\n{}\r\n\r\n",
        status_line, base_headers, content_length, etag, modified,
    );

    if !modified_since.is_empty() {
        match NaiveDateTime::parse_from_str(&modified_since, "%a, %d %b %Y %H:%M:%S GMT") {
            Ok(value) => {
                if value >= modified_short {
                    let headers = format!(
                        "{}\n{}\n{}\n{}\r\n\r\n",
                        "HTTP/1.1 304 Not Modified", base_headers, modified, etag
                    );
                    return (headers, Vec::new());
                }
            }
            Err(_) => {}
        }
    }
    return (headers, contents);
}
