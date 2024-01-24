use crate::app_config::AppConfig;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{path::PathBuf, time::Duration};

use crate::consts;

#[derive(Deserialize, Serialize, Clone)]
struct OtpResponse {
    access_token: String,
    refresh_token: String,
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

    let pb = ProgressBar::new_spinner();

    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim.bold} shc: {wide_msg}")
            .unwrap()
            .tick_chars("/|\\- "),
    );
    pb.set_message("Sending OTP...");
    let _ = client
        .post(format!("{}/auth/otp", consts::SHC_BACKEND_API_BASE_URL))
        .json(&json!({
            "name": name,
            "email": email
        }))
        .send()
        .await?;

    pb.finish_and_clear();

    let otp = dialoguer::Input::<String>::new()
        .with_prompt("Check your mail for OTP, Enter")
        .interact_text()
        .unwrap();

    let pb = ProgressBar::new_spinner();

    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim.bold} shc: {wide_msg}")
            .unwrap()
            .tick_chars("/|\\- "),
    );
    pb.set_message("Verifying OTP...");

    let res = client
        .post(format!("{}/auth/login", consts::SHC_BACKEND_API_BASE_URL))
        .json(&json!(
            {
                "name": name,
                "otp": otp,
                "email": email
            }
        ))
        .send()
        .await?;

    pb.finish_and_clear();
    if res.status().is_success() {
        println!("Login Successfull");
        let res: OtpResponse = res.json().await?;
        config.email = Some(res.email);
        config.name = Some(res.name);
        config.user_id = Some(res.id);
        config.access_token = Some(res.access_token);
        config.refresh_token = Some(res.refresh_token);
        config.save(config_path);
    } else {
        println!("Login Failed");
    }
    Ok(())
}

pub async fn check_for_api_key(
    config: &mut AppConfig,
    config_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    match config.access_token.as_ref() {
        Some(_) => {}
        None => {
            println!("Please login first");
            login(config, config_path).await?;
        }
    }
    Ok(())
}
