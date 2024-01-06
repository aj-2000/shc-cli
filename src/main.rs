use clap::{arg, Command};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

fn cli() -> Command {
    Command::new("shc")
        .about("share code in minimum time")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("login").about("logging to use shc"))
        .subcommand(
            Command::new("share")
                .about("share file")
                .arg(arg!(<FILE> "file path to share"))
                .arg_required_else_help(true),
        )
}

#[derive(Deserialize, Serialize, Clone)]

struct Config {
    api_key: Option<String>,
    email: Option<String>,
}

impl Config {
    fn new(config_path: &PathBuf) -> Self {
        if !config_path.exists() {
            let config = Config {
                api_key: None,
                email: None,
            };
            config.save(config_path);
            return config;
        }

        let contents =
            fs::read_to_string(config_path).expect("Something went wrong reading the file");

        let config: Config = toml::from_str(&contents).expect("Could not parse TOML");

        config
    }

    fn save(&self, config_path: &PathBuf) {
        let toml = toml::to_string(self).unwrap();
        fs::write(config_path, toml).unwrap();
    }
}

fn check_for_api_key(config: &mut Config, config_path: &PathBuf) {
    match config.api_key.as_ref() {
        Some(_) => {}
        None => {
            println!("Please login first");
            let email = dialoguer::Input::<String>::new()
                .with_prompt("Email")
                .interact_text()
                .unwrap();

            let otp = dialoguer::Input::<String>::new()
                .with_prompt("Check your inbox for OTP, Enter OTP")
                .interact_text()
                .unwrap();

            println!("Logging in with email: {} {}", email, "");

            config.email = Some(email.to_string());
            config.api_key = Some(otp.to_string());
            config.save(&config_path);
        }
    }
}

fn main() {
    let config_path = dirs::home_dir()
        .unwrap()
        .join("Documents/DEV/shc-cli/config.toml");
    let mut config = Config::new(&config_path);

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("login", _)) => {
            let email = dialoguer::Input::<String>::new()
                .with_prompt("Email")
                .interact_text()
                .unwrap();

            let otp = dialoguer::Input::<String>::new()
                .with_prompt("Check you mail for OTP, Enter")
                .interact_text()
                .unwrap();

            println!("Logging in with email: {} {}", email, "");

            config.email = Some(email.to_string());
            config.api_key = Some(otp.to_string());
            config.save(&config_path);
        }

        None => println!("No subcommand was used"),

        _ => {
            check_for_api_key(&mut config, &config_path);
            match matches.subcommand() {
                Some(("share", sub_matches)) => {
                    let file = sub_matches.get_one::<String>("FILE").expect("required");
                    let file_path = PathBuf::from(file);
                    if !file_path.exists() {
                        println!("File not found");
                        return;
                    }
                    let file_name = file_path.file_name().unwrap().to_str().unwrap();
                    println!("Sharing file: {} ", file_name);
                }
                _ => println!("Command not found."),
            };
        }
    };
}

