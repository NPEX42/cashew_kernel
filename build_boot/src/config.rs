use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub runner: String,
    pub run_args: Option<Vec<String>>,
    pub debug_args: Option<Vec<String>>,
    pub disks: Option<HashMap<String, String>>,
    machine: Option<Machine>,
}

#[derive(Debug, Deserialize)]
struct Machine {
    pub memory: Option<String>,
    pub tty: Option<String>,
    pub wait_for_gdb: bool,
    pub nic: Option<String>,
}

impl Config {
    pub fn from(cfg: &str) -> Result<Config, toml::de::Error> {
        toml::from_str(cfg)
    }

    /// Get The Debug Args If Present. If There Are No Debug Arguments, The Run Args Are Returned If Present.
    #[allow(unused)]
    pub fn debug_args(&self) -> Option<&Vec<String>> {
        if self.debug_args.is_none() {
            self.run_args.as_ref()
        } else {
            self.debug_args.as_ref()
        }
    }

    pub fn get_disk(&self, disk: &str) -> Option<&String> {
        if let Some(disks) = &self.disks {
            return disks.get(disk);
        } else {
            return None;
        }
    }

    pub fn get_disks(&self) -> Option<Vec<(&String, &String)>> {
        if let Some(disks) = &self.disks {
            return Some(disks.into_iter().collect());
        } else {
            return None;
        }
    }

    pub fn to_args(&self) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        args.push("-drive".into());
        args.push(format!(
            "format=raw,file={}",
            self.get_disk("boot").unwrap()
        ));


        if let Some(machine) = &self.machine {
            args.push("-m".into());
            if let Some(mem) = &machine.memory {
                args.push(mem.clone());
            } else {
                args.push("128M".into());
            }

            args.push("-serial".into());
            if let Some(tty) = &machine.tty {
                args.push(tty.clone());
            } else {
                args.push("stdio".into());
            }

            if machine.wait_for_gdb {
                args.push("-s".into());
                args.push("-S".into());
            }

            if let Some(nic) = &machine.nic {
                args.push("-device".into());
                args.push(nic.into());
            }

            for (name, disk) in self.get_disks().unwrap().iter() {
                // Boot has been dealt With
                if name.eq_ignore_ascii_case("boot") {
                    continue;
                }

                args.push(format!("-{}", name));
                args.push(disk.to_string());
            }
        }

        println!("Args: {:?}.", args);
        args
    }
}
