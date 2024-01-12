use chrono::{DateTime, Utc};
use dialoguer::{Confirm, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::command::list::File;
use crate::consts;

pub async fn remove_file(
    search: &str,
    access_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let pb = ProgressBar::new_spinner();

    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim.bold} shc: {wide_msg}")
            .unwrap()
            .tick_chars("/|\\- "),
    );
    pb.set_message("Fetching files...");
    let res = client
        .get(format!(
            "{}/api/file/list?search={}",
            consts::SHC_BACKEND_API_BASE_URL,
            search
        ))
        .header("Authorization", access_token)
        .send()
        .await?
        .json::<Vec<File>>()
        .await?;
    pb.finish_and_clear();

    let items = res
        .iter()
        .map(|file| -> Result<String, Box<dyn std::error::Error>> {
            let updated_at = DateTime::<Utc>::from(DateTime::parse_from_rfc3339(&file.updated_at)?)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            let size = if file.size < 1024 {
                format!("{:.3} KB", file.size as f64 / 1024.0)
            } else {
                format!("{:.3} MB", file.size as f64 / 1024.0 / 1024.0)
            };
            Ok(format!("{}  {}  {}", file.name, size, updated_at,))
        })
        .collect::<Result<Vec<String>, Box<dyn std::error::Error>>>()?;

    let selection = if items.is_empty() {
        println!("No files found.");
        return Ok(());
    } else {
        Select::new()
            .with_prompt("Which file do you want to delete?")
            .items(&items)
            .interact()
            .unwrap()
    };

    let confirm = Confirm::new()
        .with_prompt("Are you sure?")
        .default(false)
        .interact()
        .unwrap();

    if !confirm {
        println!("Aborted");
        return Ok(());
    } else {
        let pb = ProgressBar::new_spinner();

        pb.enable_steady_tick(Duration::from_millis(200));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.dim.bold} shc: {wide_msg}")
                .unwrap()
                .tick_chars("/|\\- "),
        );
        pb.set_message("Deleting file...");
        let file_id = res[selection].id.clone();
        let res = client
            .delete(format!(
                "{}/api/file/remove/{}",
                consts::SHC_BACKEND_API_BASE_URL,
                file_id
            ))
            .header("Authorization", access_token)
            .send()
            .await?;
        pb.finish_and_clear();
        if res.status().is_success() {
            println!("Done");
        } else {
            println!("Failed");
        }
    }
    Ok(())
}
