use ::std::collections::HashMap;
use clap::{App, Arg};
use lazy_static::lazy_static;
use std::fs;
use yaml_rust::YamlLoader;

pub struct HostConfig {
    pub host: String,
    pub port: String,
    pub root: String,
    pub resources: String,
    pub redirect_to: String,
    pub redirect_permanent: bool,
}

pub struct SslConfig {
    pub port: String,
    pub key: String,
    pub cert: String,
}

pub struct Config {
    pub file: String,
    pub loglevel: String,
    pub ssl_config: HashMap<String, SslConfig>,
    pub host_configs: HashMap<String, HostConfig>,
}

pub fn _get_config() -> Config {
    let mut host_config = HashMap::new();
    let mut ssl_config = HashMap::new();

    let matches = App::new("Menial 2")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("host")
                .help("The host to run"),
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
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .value_name("key")
                .help("The ssl key")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cert")
                .short("c")
                .long("cert")
                .value_name("cert")
                .help("The ssl cert")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("loglevel")
                .short("l")
                .long("loglevel")
                .value_name("loglevel")
                .help("The loglevel")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("redirect_to")
                .short("t")
                .long("redirect_to")
                .value_name("redirect_to")
                .help("The redirect target")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("redirect_permanent")
                .short("u")
                .long("redirect_permanent")
                .value_name("redirect_permanent")
                .help("Should redirect permanent"),
        )
        .get_matches();

    let config_path = String::from(matches.value_of("file").unwrap_or(""));
    if config_path != "" {
        let path = config_path.to_owned();
        let yaml_content = fs::read_to_string(config_path).unwrap();

        let docs = YamlLoader::load_from_str(&yaml_content).unwrap();
        let doc = &docs[0];

        let hosts = &doc["hosts"];

        for item in hosts.as_hash().unwrap() {
            let combined_host = &item.0.as_str().unwrap().to_owned();

            let port;
            if combined_host.split(":").count() == 2 {
                port = combined_host.split(":").collect::<Vec<&str>>()[1];
            } else {
                port = "80";
            }

            let root = item.1["root"].as_str().unwrap_or("");
            let resources = item.1["resources"].as_str().unwrap_or("");
            let redirect_to = item.1["redirect_to"].as_str().unwrap_or("");
            let redirect_permanent = item.1["redirect_permanent"].as_bool().unwrap_or(false);

            host_config.insert(
                combined_host.to_owned(),
                HostConfig {
                    host: combined_host.to_owned(),
                    port: port.to_owned(),
                    root: root.to_owned(),
                    resources: resources.to_owned(),
                    redirect_to: redirect_to.to_owned(),
                    redirect_permanent: redirect_permanent,
                },
            );
        }

        let ssl = &doc["ssl"];
        for item in ssl.as_hash().unwrap() {
            let ssl_port = &item.0.as_i64().unwrap();
            ssl_config.insert(
                ssl_port.to_string(),
                SslConfig {
                    port: ssl_port.to_string(),
                    key: item.1["key"].as_str().unwrap().to_string(),
                    cert: item.1["cert"].as_str().unwrap().to_string(),
                },
            );
        }

        let loglevel = &doc["loglevel"].as_str().unwrap_or("info");

        return Config {
            file: String::from(path.to_owned()),
            loglevel: loglevel.to_string(),
            ssl_config: ssl_config,
            host_configs: host_config,
        };
    } else {
        let host = matches.value_of("host").unwrap_or("127.0.0.1").to_owned();
        let port = matches.value_of("port").unwrap_or("8080").to_owned();
        let root =
            String::from(matches.value_of("root").unwrap_or("default/welcomepage")).to_owned();
        let loglevel = matches.value_of("loglevel").unwrap_or("info").to_owned();
        let key = String::from(matches.value_of("key").unwrap_or("")).to_owned();
        let cert = String::from(matches.value_of("cert").unwrap_or("")).to_owned();

        let resources =
            String::from(matches.value_of("resources").unwrap_or("default/pages")).to_owned();
        let combined_host = format!("{}:{}", host.to_owned(), port);

        let redirect_to = matches.value_of("redirect_to").unwrap_or("").to_owned();
        let redirect_permanent = matches.is_present("redirect_permanent");

        host_config.insert(
            combined_host,
            HostConfig {
                host: host.to_owned(),
                port: port.to_owned(),
                root: root,
                resources: resources,
                redirect_to: redirect_to,
                redirect_permanent: redirect_permanent,
            },
        );

        if key != "" && cert != "" {
            ssl_config.insert(
                port.to_owned(),
                SslConfig {
                    port: port.to_owned(),
                    key: key,
                    cert: cert,
                },
            );
        }

        return Config {
            file: String::from("None"),
            loglevel: loglevel,
            ssl_config: ssl_config,
            host_configs: host_config,
        };
    }
}

lazy_static! {
    pub static ref CONFIG: Config = _get_config();
}
