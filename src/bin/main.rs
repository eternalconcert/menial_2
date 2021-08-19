extern crate clap;
extern crate yaml_rust;

use ansi_term::Colour;
use chrono::{DateTime, Utc};
use menial_2::config::get_config;
use menial_2::{log, ThreadPool, LOG_LEVEL};
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;

fn main() {
    log!("info", "Starting menial/2");
    let config = get_config();

    log!("info", format!("Config file: {}", config[0].file));

    let pool = ThreadPool::new(config.len());
    for i in 0..config.len() {
        log!("info", format!("Bind: {}", config[i].bind));
        log!("info", format!("Port: {}", config[i].port));
        log!("info", format!("Document root: {}", config[i].root));
        log!("info", format!("Resources root: {}", config[i].resources));
        pool.execute(move || {
            run_server(i);
        });
    }

}


fn run_server(i: usize) {
    let config = get_config();
    let listener = TcpListener::bind(format!("{}:{}", config[i].bind, config[i].port)).unwrap();

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
    log!("debug", format!("Requested host: {}", host));
    
    let mut config_index = 0;
    for (i, _) in get_config().iter().enumerate() {
        let combined_host = String::from(format!("{}:{}", &get_config()[i].host, &get_config()[i].port));
        if combined_host == host {
            println!("{}", combined_host == host);
            config_index = i;
            
        }
        
    }

    let host_config = &get_config()[config_index];
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

    stream.write(response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();

    stream.flush().unwrap();
}
