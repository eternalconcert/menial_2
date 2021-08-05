use clap::{App, Arg};
use std::fs;
use yaml_rust::YamlLoader;

pub struct Config {
    pub file: String,
    pub host: String,
    pub port: String,
    pub root: String,
    pub resources: String,
}

pub fn get_config() -> Vec<Config> {
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

    let config_path = String::from(matches.value_of("file").unwrap_or(""));
    if config_path != "" {

        let conf = config_path.to_owned();
        let yaml_content = fs::read_to_string(config_path).unwrap();

        let docs = YamlLoader::load_from_str(&yaml_content).unwrap();
        let doc = &docs[0];

        let hosts = &doc["hosts"];
        let mut res = Vec::with_capacity(hosts.as_hash().unwrap().len());

        for item in hosts.as_hash().unwrap() {
            let host = item.0.as_str().unwrap();
            let port = item.1["port"].as_str().unwrap();
            let root = item.1["root"].as_str().unwrap();
            let resources = item.1["resources"].as_str().unwrap();
            res.push(Config {
                file: String::from(conf.to_owned()),
                host: host.to_owned(),
                port: port.to_owned(),
                root: root.to_owned(),
                resources: resources.to_owned(),
            });
        };
        return res;

    } else {
        let mut res = Vec::with_capacity(1);
        let host = matches.value_of("host").unwrap_or("127.0.0.1").to_owned();
        let port = matches.value_of("port").unwrap_or("8080").to_owned();
        let root = String::from(matches.value_of("root").unwrap_or("default")).to_owned();
        let resources =
            String::from(matches.value_of("resources").unwrap_or("default/pages")).to_owned();
        res.push(Config {
            file: String::from("None"),
            host: host,
            port: port,
            root: root,
            resources: resources,
        });
        return res;
    }
}
