use std::collections::HashMap;
use std::env::args;
use std::process::exit;
use std::process::Command;
use std::str;

use serde_json::Value;
use std::cmp::max;

#[macro_use]
extern crate serde_derive;

const CONFIG_FILE: &str = "docker-compose.yml";

#[derive(Deserialize, Serialize)]
struct Network {
    #[serde(alias = "IPAddress")]
    ip_address: String,
}

#[derive(Deserialize, Serialize)]
struct NetworkSettings {
    #[serde(alias = "Ports")]
    ports: HashMap<String, Value>,
    #[serde(alias = "Networks")]
    networks: HashMap<String, Network>,
}

#[derive(Deserialize, Serialize)]
struct Inspect {
    #[serde(alias = "Name")]
    name: String,
    #[serde(alias = "NetworkSettings")]
    network_settings: NetworkSettings,
}

#[derive(Debug)]
struct Host {
    name: String,
    protocol: String,
    ip: String,
    port: String,
}

impl Host {
    fn from_inspect(inspect: &Inspect) -> Self {
        let mut port_protocol = inspect
            .network_settings
            .ports
            .keys()
            .next()
            .unwrap()
            .split("/");

        Self {
            name: inspect.name.trim_start_matches("/").to_owned(),
            port: port_protocol.next().unwrap().to_owned(),
            protocol: port_protocol.next().unwrap().to_owned(),
            ip: inspect
                .network_settings
                .networks
                .iter()
                .next()
                .unwrap()
                .1
                .ip_address
                .to_string(),
        }
    }

    fn url(&self) -> String {
        format!("http://{}:{}", self.ip, self.port)
    }
}

fn center(val: &String, width: usize) -> String {
    if val.len() >= width {
        return val.to_owned();
    }
    let diff = width - val.len();
    let end = diff / 2;
    let start = diff - end;
    [" ".repeat(start), val.to_owned(), " ".repeat(end)]
        .concat()
        .to_owned()
}

fn ljust(val: &String, width: usize) -> String {
    if val.len() >= width {
        return val.to_owned();
    }
    [val.to_owned(), " ".repeat(width - val.len())]
        .concat()
        .to_owned()
}

fn show_help() {
    println!(
        "{}",
        [
            format!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION")).as_str(),
            env!("CARGO_PKG_DESCRIPTION"),
            "",
            "OPTIONS:",
            "    -f, --file File",
            format!(
                "        Specify an alternate compose file. Default: {}",
                CONFIG_FILE
            )
            .as_str(),
            "    --help",
            "        Prints help information. Use --help for more details.",
            "    --version",
            "        Prints version information.",
            "",
        ]
        .join("\n")
    );
}

fn show_version() {
    println!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));
}

macro_rules! show_help {
    ($arg: expr) => {
        if $arg == "--help" {
            show_help();
            exit(0);
        } else if $arg == "--version" {
            show_version();
            exit(0);
        }
    };
}

fn main() {
    let mut config_file = CONFIG_FILE.to_owned();
    let mut args = args();
    // skip arg[0]
    args.next();
    loop {
        match args.next() {
            Some(arg) => {
                show_help!(arg);
                if arg == "-f" || arg == "--file" {
                    if let Some(arg) = args.next() {
                        show_help!(arg);
                        config_file = arg.to_owned()
                    }
                }
            }
            None => break,
        }
    }

    let ret = Command::new("docker-compose")
        .args(&["-f", config_file.as_str(), "ps", "-q"])
        .output()
        .expect("failed to execute process");
    let container_ids: Vec<&str> = str::from_utf8(&ret.stdout)
        .unwrap()
        .strip_suffix("\n")
        .unwrap()
        .split("\n")
        .collect();

    let ret = Command::new("docker")
        .args(&["container", "inspect"])
        .args(&container_ids)
        .output()
        .unwrap();

    let data = str::from_utf8(&ret.stdout).unwrap();

    let json: Vec<Inspect> = serde_json::from_str(data).unwrap();
    let hosts: Vec<Host> = json.iter().map(Host::from_inspect).collect();

    let max_len_name = hosts.iter().fold(4, |acc, host| max(acc, host.name.len()));
    let max_len_protocol = hosts
        .iter()
        .fold(8, |acc, host| max(acc, host.protocol.len()));
    let max_len_ip = hosts.iter().fold(2, |acc, host| max(acc, host.ip.len()));
    let max_len_port = hosts.iter().fold(4, |acc, host| max(acc, host.port.len()));
    let max_len_url = hosts.iter().fold(3, |acc, host| max(acc, host.url().len()));

    println!(
        "{}  {}  {}  {}  {}",
        center(&"Name".to_string(), max_len_name),
        center(&"Protocol".to_string(), max_len_protocol),
        center(&"Ip".to_string(), max_len_ip),
        center(&"Port".to_string(), max_len_port),
        center(&"Url".to_string(), max_len_url),
    );
    println!(
        "{}",
        "-".repeat(max_len_name + max_len_protocol + max_len_ip + max_len_port + max_len_url + 10)
    );
    for host in hosts {
        println!(
            "{}  {}  {}  {}  {}",
            ljust(&host.name, max_len_name),
            ljust(&host.protocol, max_len_protocol),
            ljust(&host.ip, max_len_ip),
            ljust(&host.port, max_len_port),
            ljust(&host.url(), max_len_url),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::center;

    #[test]
    fn centered() {
        let s = "ABC".to_owned();
        let r = center(&s, 2);
        assert_eq!("ABC", r.as_str());
        let r = center(&s, 5);
        assert_eq!(" ABC ", r.as_str());
        let r = center(&s, 4);
        assert_eq!(" ABC", r.as_str());
    }
}
