use crate::{api_client, consts};
use chrono::{DateTime, Utc};
use console::style;
use dialoguer::{theme, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::utils::format_bytes;

pub async fn list_files(
    search: &str,
    access_token: &str,
    refresh_token: &str,
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
    let mut api_client = crate::api_client::ApiClient::new(access_token, refresh_token);
    let res = api_client.list_files(search).await?;
    pb.finish_and_clear();

    let items = res
        .results
        .iter()
        .map(|file| -> Result<String, Box<dyn std::error::Error>> {
            let updated_at = DateTime::<Utc>::from(DateTime::parse_from_rfc3339(&file.updated_at)?)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            let size = format_bytes(file.size);
            Ok(format!("{}  {}  {}", file.name, size, updated_at,))
        })
        .collect::<Result<Vec<String>, Box<dyn std::error::Error>>>()?;

    let selection = if items.is_empty() {
        return Ok(());
    } else {
        let file_count = items.len();
        let prompt = if file_count > 100 {
            format!("Select a file to see more info. (Last 100 files, use filter to get more specific results)")
        } else {
            format!("Select a file to see more info.  ({} files)", file_count)
        };

        Select::with_theme(&theme::ColorfulTheme::default())
            .max_length(20)
            .with_prompt(prompt)
            .default(0)
            .items(&items)
            .interact()
            .unwrap()
    };

    let file = &res.results[selection];
    let file_name = &file.name;
    let upload_status = &file.upload_status;
    let updated_at = DateTime::<Utc>::from(DateTime::parse_from_rfc3339(&file.updated_at)?)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let size = if file.size < 1024 {
        format!("{:.3} KB", file.size as f64 / 1024.0)
    } else {
        format!("{:.3} MB", file.size as f64 / 1024.0 / 1024.0)
    };
    let visibility = if file.is_public { "Public" } else { "Private" };
    let shareable_link = format!("https://shc.ajaysharma.dev/files/{}", file.id);

    console::Term::stdout()
        .write_line( format!(
        "\nFile Name: {}\nUpload Status: {}\nUpdated At: {}\nSize: {}\nVisibility: {}\nShareable Link: {}",
        style(file_name).cyan(),
        style(upload_status).yellow(),
        style(updated_at).green(),
        style(size).magenta(),
        style(visibility).blue(),
        style(shareable_link).underlined().bright().blue()
    ).as_ref())?;

    Ok(())
}
