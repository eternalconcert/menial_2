use crate::utils::{
    get_base_headers,
    get_response_headers,
    make304,
    intrusion_try_detected,
    get_file_data,
    get_redirect_response,
    get_status_line,
    get_default_status_page
};
use crate::config::{HostConfig, CONFIG};
use crate::{log, ThreadPool, LOG_LEVEL};
use base64::{Engine as _, engine::{general_purpose}};

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
    let mut none_match = String::from("");
    let mut auth = String::from("");

    for line in request_content.split("\n") {
        if line.starts_with("GET") || line.starts_with("POST") {
            match line.find("HTTP") {
                Some(v) => document = String::from(&line[4..v - 1]),
                None => {}
            }
        } else if line.starts_with("If-Modified-Since:") {
            modified_since = String::from(&line[19..line.len() - 1]);
        } else if line.starts_with("If-None-Match:") {
            none_match = String::from(&line[15..line.len() - 1]);
        } else if line.starts_with("Host:") {
            host = String::from(&line[6..line.len() - 1]);
        } else if line.starts_with("Authorization:") {
            auth = String::from(&line[15..line.len() - 1]);
        }
    }

    if host.find(":").unwrap_or(0) == 0 {
        host = format!("{}:{}", host, default_port)
    }

    log!("debug", format!("Requested host: {}", host));


    let split_for_search = document.split("?");
    let parts: Vec<&str> = split_for_search.collect();
    if parts.len() > 1 {
        let search_params = parts[1].to_string();
        log!("debug", format!("Search params (NOT used at the moment!): {}", search_params));
        log!("debug", "Removed search params from document.");
        document = parts[0].to_string();
    }

    let mut host_config = CONFIG.host_configs.values().collect::<Vec<&HostConfig>>()[0];

    match CONFIG.host_configs.get(&host) {
        Some(conf) => host_config = conf,
        _ => {
            log!("debug", "Host not found");
        }
    }

    let base_headers = get_base_headers();

    if host_config.authfile != "" {
        let unauthorized = (String::from("HTTP/1.1 401 Unauthorized\n"), Vec::from("WWW-Authenticate: Basic realm=User Visible Realm\n\n"));

        if auth.len() > 5 {
            let basic_auth = String::from_utf8(general_purpose::STANDARD.decode(String::from(&auth[6..auth.len() - 0])).unwrap()).unwrap();
            let split_for_auth: Vec<&str> = basic_auth.split(":").collect();
            let username = split_for_auth[0].to_string();
            let password = general_purpose::STANDARD.encode(split_for_auth[1].to_string());

            let raw_authfile_content = fs::read_to_string(&host_config.authfile).unwrap();

            let mut authenticated = false;
            for line in raw_authfile_content.split("\n") {
                let line_splitted: Vec<&str> = line.split(":").collect();
                if line_splitted.len() != 2 {
                    continue;
                }
                let line_username = line_splitted[0];
                let line_password = line_splitted[1];
                if line_username.to_string() == username && line_password.to_string() == password {
                    authenticated = true;
                    log!("debug", String::from("User authorized"));
                    break;
                };
            }
            if !authenticated {
                log!("debug", String::from("User not authorized: Wrong password/username"));
                return unauthorized;
            }
        } else {
            log!("debug", String::from("User not authorized: First attempt, not asked for password/username"));
            return unauthorized;
        }
    }

    if host_config.redirect_to != "" {
        return get_redirect_response(host_config.redirect_permanent, host_config.redirect_to.to_owned(), base_headers);
    }

    let document_root = host_config.root.to_owned();
    let resources_root = host_config.resources.to_owned();

    let mut status: u16 = 0;

    if intrusion_try_detected(request_content.to_string()) {
        log!(
            "warning",
            format!("Intrusion try detected: {}", request_content)
        );
        status = 400;
   }

    if document == "/" {
        document = String::from("/index.html");
    }

    log!("debug", format!("GET-Request: {}", document));

    let doc = String::from(format!("{}{}", document_root, document));
    log!("debug", format!("Requested document path: {}", doc));

    let filename: String;
    if status == 0 {
        if Path::new(&doc).exists() && request_content.starts_with("GET") {
            status = 200;
        } else {
            log!("warning", format!("Requested document not found: {}", doc));
            status = 404;
        };
    }

    let status_line = get_status_line(status);

    let (contents, file_modified, modified_short, hash);
    match status {
        200 => {
            (contents, file_modified, modified_short, hash) = get_file_data(&doc);
            filename = doc;
        }
        400 | 404 => {
            if host_config.resources != "" {
                filename = String::from(format!("{}/{}.html", resources_root, status));
                (contents, file_modified, modified_short, hash) = get_file_data(&filename);
            } else {
                filename = String::from(format!("{}.html", status));
                (contents, file_modified, modified_short, hash) = get_default_status_page(status);
            }
        }
        _ => {
            panic!("Unknown Status: {}", status);
        }
    }

    let (content_length, etag, modified, content_type) = get_response_headers(&contents, file_modified, &hash, &filename);

    if !none_match.is_empty() && none_match == format!("\"{}\"", hash) {
        return (make304(modified, etag), Vec::new());
    }

    if !modified_since.is_empty() {
        match NaiveDateTime::parse_from_str(&modified_since, "%a, %d %b %Y %H:%M:%S GMT") {
            Ok(value) => {
                if value >= modified_short {
                    return (make304(modified, etag), Vec::new());
                }
            }
            Err(_) => {}
        }
    }

    let headers = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\r\n\r\n",
        status_line, base_headers, content_length, etag, modified, content_type,
    );

    return (headers, contents);
}
