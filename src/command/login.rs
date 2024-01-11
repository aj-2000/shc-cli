use crate::app_config::AppConfig;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use crate::consts;

#[derive(Deserialize, Serialize, Clone)]
struct OtpResponse {
    password: String,
    email: String,
    name: String,
    id: String,
}

pub async fn login(
    config: &mut AppConfig,
    config_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
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
        .post(format!("{}/auth/login", consts::SHC_BACKEND_API_BASE_URL))
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
        .post("http://localhost:6969/auth/otp")
        .json(&map)
        .send()
        .await?;

    println!("Status: {} {}", res.status(), "rqeuesting for api key");

    if res.status().is_success() {
        println!("Login Successfull");
        let res: OtpResponse = res.json().await?;
        config.email = Some(res.email);
        config.name = Some(res.name);
        config.user_id = Some(res.id);
        config.password = Some(res.password);
        config.save(&config_path);
    } else {
        println!("Login Failed");
    }
    Ok(())
}

pub async fn check_for_api_key(
    config: &mut AppConfig,
    config_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    match config.password.as_ref() {
        Some(_) => {}
        None => {
            println!("Please login first");
            login(config, config_path).await?;
        }
    }
    Ok(())
}
