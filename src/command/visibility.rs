use dialoguer::{Confirm, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::api_client;

pub async fn toggle_file_visibility(
    search: &str,
    api_client: &mut api_client::ApiClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();

    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim.bold} shc: {wide_msg}")
            .unwrap()
            .tick_chars("/|\\- "),
    );
    pb.set_message("Fetching files...");
    let res = api_client.list_files(search).await?;
    pb.finish_and_clear();

    let items = &res
        .results
        .iter()
        .map(|file| -> Result<String, Box<dyn std::error::Error>> {
            let size = if file.size < 1024 {
                format!("{:.3} KB", file.size as f64 / 1024.0)
            } else {
                format!("{:.3} MB", file.size as f64 / 1024.0 / 1024.0)
            };
            let visibility = if file.is_public { "Public" } else { "Private" };
            Ok(format!("{}  {}  {}", file.name, size, visibility))
        })
        .collect::<Result<Vec<String>, Box<dyn std::error::Error>>>()?;

    let selection = if items.is_empty() {
        println!("No files found.");
        return Ok(());
    } else {
        Select::new()
            .with_prompt("Which file you want to change visibility?")
            .items(items)
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
        pb.set_message("Toggling visibility...");
        let file_id = res.results[selection].id.clone();
        let res = api_client.toggle_file_visibility(file_id.as_str()).await;
        pb.finish_and_clear();
        match res {
            Ok(res) => {
                let visiblity = if res.is_public { "Public" } else { "Private" };
                println!("Visibility of \"{}\" changed to {}", res.name, visiblity);
            }
            Err(e) => {
                println!("Error: {}", e)
            }
        }
    }
    Ok(())
}
