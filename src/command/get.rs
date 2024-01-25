use chrono::{DateTime, Utc};
use dialoguer::{Confirm, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tokio_stream::StreamExt;
use crate::api_client;

pub async fn download_file(
    search: &str,
    api_client: &mut api_client::ApiClient,
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
   let res = api_client
        .list_files(search)
        .await?;

    pb.finish_and_clear();

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
        let file_id = res.results[selection].id.clone();
        let pb = ProgressBar::new_spinner();

        pb.enable_steady_tick(Duration::from_millis(200));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.dim.bold} shc: {wide_msg}")
                .unwrap()
                .tick_chars("/|\\- "),
        );
        pb.set_message("Preparing for download...");
        let res = api_client
            .get_file_download_url(&file_id)
            .await;
        
        pb.finish_and_clear();
        
        let shc_file = match res {
            Ok(shc_file) => shc_file,
            Err(e) => {
                println!("Error: {}", e);
                std::process::exit(1);
            }
        };

        let download_url = shc_file.download_url;
        let file_name = shc_file.name;

        let mut downloaded: u64 = 0;

        let res = client.get(download_url.unwrap()).send().await?;
        let total_size = downloaded + res.content_length().unwrap_or(0);
        let bar = ProgressBar::new(total_size);
        let file = File::create(&file_name)
            .map_err(|_| format!("Failed to create file '{file_name}'"))
            .unwrap();

        let mut out: Box<dyn Write + Send> = Box::new(std::io::BufWriter::new(file));

        bar.set_style(
        ProgressStyle::with_template(
            "{msg}\n{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {bytes_per_sec} \n",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
        bar.reset_eta();
        bar.set_message(format!("Downloading... {}", file_name));

        let mut stream = res.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|_| "Error while downloading.").unwrap();
            out.write_all(&chunk)
                .map_err(|_| "Error while writing to output.")
                .unwrap();
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            bar.set_position(new);
        }
        bar.finish_and_clear();
        println!("Downloaded {}", file_name);
    }

    Ok(())
}
