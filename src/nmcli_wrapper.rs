use std::{io::BufRead, process::Command};

#[derive(Debug)]
pub struct NetworkInfo {
    pub device: String,
    pub connection: String,
    pub ip4_address: String,
    pub ip4_gateway: String,
    pub ip4_dns: String,
}

#[derive(Debug)]
pub struct Network {
    pub in_use: String,
    pub bssid: String,
    pub ssid: String,
    pub mode: String,
    pub chan: String,
    pub rate: String,
    pub signal: String,
    pub bars: String,
    pub security: String,
}

impl Network {
    pub const fn ref_array(&self) -> [&String; 9] {
        [
            &self.in_use,
            &self.bssid,
            &self.ssid,
            &self.mode,
            &self.chan,
            &self.rate,
            &self.signal,
            &self.bars,
            &self.security,
        ]
    }

    pub fn bssid(&self) -> &str {
        &self.bssid
    }

    pub fn ssid(&self) -> &str {
        &self.ssid
    }

    pub fn mode(&self) -> &str {
        &self.mode
    }

    pub fn chan(&self) -> &str {
        &self.chan
    }

    pub fn rate(&self) -> &str {
        &self.rate
    }

    pub fn signal(&self) -> &str {
        &self.signal
    }

    pub fn bars(&self) -> &str {
        &self.bars
    }

    pub fn security(&self) -> &str {
        &self.security
    }
}

#[derive(Debug)]
pub struct NmcliWrapper {}

impl NmcliWrapper {
    pub fn connect_to_network(network_name: &str, password: &str) {
        let output = Command::new("nmcli")
            .args([
                "device",
                "wifi",
                "connect",
                network_name,
                "password",
                password,
            ])
            .output()
            .expect("failed to connect");

        println!("Some debug!!!");

        output.stdout.lines().for_each(|l| println!("{:?}", l));
    }

    pub fn rescan_networks() {
        Command::new("nmcli")
            .args(["device", "wifi", "rescan"])
            .output()
            .expect("failed to rescan networks");
    }

    pub fn get_saved_networks() {
        let output = Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "NAME,TYPE",
            "connection",
            "show"
        ])
        .output()
        .expect("failed to get saved networks");

        if !output.status.success() {
            eprintln!(
                "Error gathering device name:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            // return String::new();
        }

        output.stdout.lines().for_each(|line| {
            let part = line.unwrap();
            if part.ends_with("wireless") && !part.starts_with("Hotspot") {
                let parts: Vec<&str> = part.split(":").collect();
                println!("{}", parts[0]);
            }
        });

    }

    pub fn delete_network() {
        
    }

    pub fn get_device_name() -> String {
        let output = Command::new("nmcli")
            .args(["-t", "-f", "DEVICE", "connection", "show", "--active"])
            .output()
            .expect("failed");

        if !output.status.success() {
            eprintln!(
                "Error gathering device name:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return String::new();
        }

        let device_name = output.stdout.lines().next().unwrap();
        device_name.unwrap()
    }

    pub fn get_device_info(device: String) -> NetworkInfo {
        let output = Command::new("nmcli")
            .args(["-t", "device", "show", device.as_str()])
            .output()
            .expect("failed");

        let mut device_info = NetworkInfo {
            device: String::new(),
            connection: String::new(),
            ip4_address: String::new(),
            ip4_gateway: String::new(),
            ip4_dns: String::new(),
        };

        output.stdout.lines().for_each(|line| {
            let info = line.unwrap();
            let parts: Vec<&str> = info.split(":").collect();
            if parts.len() >= 2 {
                match parts[0] {
                    "GENERAL.DEVICE" => {
                        device_info.device = parts[1].to_string();
                    }
                    "GENERAL.CONNECTION" => {
                        device_info.connection = parts[1].to_string();
                    }
                    "IP4.ADDRESS[1]" => {
                        device_info.ip4_address = parts[1].to_string();
                    }
                    "IP4.GATEWAY" => {
                        device_info.ip4_gateway = parts[1].to_string();
                    }
                    "IP4.DNS[1]" => {
                        device_info.ip4_dns = parts[1].to_string();
                    }
                    _ => {}
                }
            }
        });
        device_info
    }

    pub fn get_networks_list() -> Vec<Network> {
        let output = Command::new("nmcli")
            .args([
                "-t",
                "-f",
                "IN-USE,BSSID,SSID,MODE,CHAN,RATE,SIGNAL,BARS,SECURITY",
                "device",
                "wifi",
                "list",
            ])
            .output()
            .expect("failed");

        if !output.status.success() {
            eprintln!(
                "Error gathering network list:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Vec::new();
        }

        let networks: Vec<Network> = output
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: String = line.unwrap().replace("\\", "");

                let new_parts: Vec<&str> = parts.split(":").collect();

                if new_parts.len() >= 14 {
                    let bssid = format!(
                        "{}:{}:{}:{}:{}:{}",
                        new_parts[1],
                        new_parts[2],
                        new_parts[3],
                        new_parts[4],
                        new_parts[5],
                        new_parts[6]
                    )
                    .to_string();

                    Some(Network {
                        in_use: new_parts[0].to_string(),
                        bssid: bssid,
                        ssid: new_parts[7].to_string(),
                        mode: new_parts[8].to_string(),
                        chan: new_parts[9].to_string(),
                        rate: new_parts[10].to_string(),
                        signal: new_parts[11].to_string(),
                        bars: new_parts[12].to_string(),
                        security: new_parts[13].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        networks
    }
}
