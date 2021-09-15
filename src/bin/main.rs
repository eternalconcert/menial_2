extern crate clap;
extern crate num_cpus;
extern crate yaml_rust;

use menial_2::utils::{get_ports, get_ssl_ports};
use ansi_term::Colour;
use chrono::{DateTime, Utc};
use menial_2::config::CONFIG;
use menial_2::server::{run_server, run_ssl_server};
use menial_2::{log, ThreadPool, LOG_LEVEL};

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
