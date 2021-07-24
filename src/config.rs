use clap::{App, Arg};
use yaml_rust::{YamlLoader};
use std::fs;
use ansi_term::Colour;
use chrono::{DateTime, Utc};
use crate::{LOG_LEVEL};


pub struct Config {
    pub host: String,
    pub port: String,
    pub root: String,
    pub resources: String,
}


pub fn get_config() -> Config {

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

    let config_path = String::from(matches.value_of("file").unwrap_or(""));

    if config_path != "" {
        crate::log!("info", format!("Config file: {}", config_path));

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

    crate::log!("info", format!("Host: {}", host));
    crate::log!("info", format!("Port: {}", port));
    crate::log!("info", format!("Document root: {}", root));
    crate::log!("info", format!("Resources root: {}", resources));

    return Config {
        host,
        port,
        root,
        resources
    };
}
