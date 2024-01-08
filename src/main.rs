use clap::{arg, Command};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

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
    password: Option<String>,
    email: Option<String>,
    name: Option<String>,
}

impl Config {
    fn new(config_path: &PathBuf) -> Self {
        if !config_path.exists() {
            let config = Config {
                password: None,
                email: None,
                name: None,
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
    match config.password.as_ref() {
        Some(_) => {}
        None => {
            println!("Please login first");
            let name = dialoguer::Input::<String>::new()
                .with_prompt("Email")
                .interact_text()
                .unwrap();
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
            config.password = Some(otp.to_string());
            config.save(&config_path);
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
struct OtpResponse {
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = dirs::home_dir()
        .unwrap()
        .join("Documents/DEV/shc-cli/config.toml");
    let mut config = Config::new(&config_path);

    let matches = cli().get_matches();

    let client = reqwest::Client::new();

    match matches.subcommand() {
        Some(("login", _)) => {
            let name = dialoguer::Input::<String>::new()
                .with_prompt("Name")
                .interact_text()
                .unwrap();

            let email = dialoguer::Input::<String>::new()
                .with_prompt("Email")
                .interact_text()
                .unwrap();

            let mut map = HashMap::new();

            map.insert("name", name.clone());
            map.insert("email", email.clone());

            let res = client
                .post("http://localhost:6969/api/auth/login")
                .json(&map)
                .send()
                .await?;

            println!("Status: {} {}", res.status(), "OTP sent to your email");

            let otp = dialoguer::Input::<String>::new()
                .with_prompt("Check you mail for OTP, Enter")
                .interact_text()
                .unwrap();

            let mut map = HashMap::new();

            map.insert("name", name.clone());
            map.insert("otp", otp.clone());
            map.insert("email", email.clone());

            let res = client
                .post("http://localhost:6969/api/auth/otp")
                .json(&map)
                .send()
                .await?;

             println!("Status: {} {}", res.status(), "rqeuesting for api key");


            if res.status().is_success() {
                println!("Login Successfull");
                config.email = Some(email.to_string());
                config.name = Some(name.to_string());
                let res: OtpResponse = res.json().await?;
                config.password = Some(res.password);
                config.save(&config_path);
            } else {
                println!("Login Failed");
            }
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
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "File not found",
                        )
                        .into());
                    }
                    let file_name = file_path.file_name().unwrap().to_str().unwrap();
                    println!("Sharing file: {} ", file_name);
                }
                _ => println!("Command not found."),
            };
        }
    };
    Ok(())
}
