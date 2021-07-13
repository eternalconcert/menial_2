extern crate clap;

use ::menial_2::ThreadPool;
use clap::{App, Arg};
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use chrono::{DateTime, Utc};
use ansi_term::Colour;

macro_rules! log {
    ($level: expr, $text: expr) => {
        assert!($level == "debug" || $level == "info" || $level == "warning" || $level == "error");
        let now: DateTime<Utc> = Utc::now();
        let formatted = String::from(format!("{} [{}:{}]: {}", now.format("%Y-%m-%d %H:%M:%S"), file!(), line!(), $text));

        match $level {
            "debug" => println!("{}", Colour::Green.paint(formatted)),
            "info" => println!("{}", formatted),
            "warning" => println!("{}", Colour::Yellow.paint(formatted)),
            "error" => println!("{}", Colour::Red.paint(formatted)),
            _ => println!("{}", formatted),
        }
    }
}

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
        .get_matches();

    let host = matches.value_of("host").unwrap_or("127.0.0.1");
    log!("info", format!("Host: {}", host));

    let port = matches.value_of("port").unwrap_or("8080");
    log!("info", format!("Port: {}", port));

    let root = String::from(matches.value_of("root").unwrap_or("."));
    log!("info", format!("Document root: {}", root));

    let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let r = root.to_owned();
        pool.execute(move || {
            handle_connection(stream, &r);
        });
    }
}

const SERVER_LINE: &str = "Server: menial 2";

fn handle_connection(mut stream: TcpStream, root: &str) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let content = String::from_utf8_lossy(&buffer);

    let get = b"GET / HTTP/1.1\r\n";

    match content.find("HTTP") {
        Some(v) => println!("---- {}", v),
        None => {}
    }

    let doc = String::from(format!("{}/index.html", root));

    let (status_line, filename) = if Path::new(&doc).exists() && buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", doc)
    } else {
        // let doc = String::from(format!("{}/404.html", root));
        (
            "HTTP/1.1 404 NOT FOUND",
            String::from("deployment/404.html"),
        )
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\n{}\nContent-Length: {}\r\n\r\n{}",
        status_line,
        SERVER_LINE,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
