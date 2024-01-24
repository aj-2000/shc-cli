use chrono::{DateTime, Utc};
use dialoguer::{theme, Confirm, Editor, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;
use std::time::Duration;

use crate::consts;
use crate::models::ShcFileResponse;

pub async fn rename_file(
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

    let res = &client
        .get(format!(
            "{}/api/files?search={}&page=1&limit=100",
            consts::SHC_BACKEND_API_BASE_URL,
            search
        ))
        .header("Authorization", access_token)
        .send()
        .await?
        .json::<ShcFileResponse>()
        .await?;
    pb.finish_and_clear();

    print!("{}[2J", 27 as char);

    let items = res
        .results
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
        Select::with_theme(
            &theme::ColorfulTheme::default(

            )
        )
            .with_prompt(format!("Which file do you want to delete?\nLast 100 files (you can use filter to get more specific results)")).default(0)
            .items(&items)
            .interact()
            .unwrap()
    };

    if let Some(new_filename) = Editor::new().edit("new filename").unwrap() {
        let confirm = Confirm::new()
            .with_prompt("Are you sure?")
            .default(false)
            .interact()
            .unwrap();

        if !confirm {
            println!("Aborted");
            return Ok(());
        } else {
            let file_id = res.results[selection].id.clone();
            let pb = ProgressBar::new_spinner();

            pb.enable_steady_tick(Duration::from_millis(200));
            pb.set_style(
                ProgressStyle::with_template("{spinner:.dim.bold} shc: {wide_msg}")
                    .unwrap()
                    .tick_chars("/|\\- "),
            );
            pb.set_message("Renaming file...");
            let res = client
                .patch(format!(
                    "{}/api/files/rename/{}",
                    consts::SHC_BACKEND_API_BASE_URL,
                    file_id
                ))
                .header("Authorization", access_token)
                .json(&json!({
                    "name": new_filename,
                }))
                .send()
                .await?;

            pb.finish_and_clear();

            if res.status().is_success() {
                println!("Done");
            } else {
                println!("Failed");
            }
        }
    } else {
        // TODO: Handle empty filename correctly
        println!("ShcFile name cannot be empty");
    }

    Ok(())
}
