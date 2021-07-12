use ::menial_2::ThreadPool;

use std::thread;
use std::time::Duration;

use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

const SERVER_LINE: &str = "Server: menial 2";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "deployment/index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 404 NOT FOUND", "deployment/404.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "deployment/404.html")
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
