use chrono::{DateTime, Utc};
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{row, Cell, Row, Table};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::consts;

#[derive(Serialize, Deserialize, Debug)]
pub struct ShcFile {
    pub name: String,
    pub id: String,
    pub extension: String,
    pub mime_type: String,
    pub size: u64,
    pub is_public: bool,
    pub updated_at: String,
    pub user_id: String,
    pub download_url: Option<String>,
    pub upload_status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShcFileResponse {
    pub results: Vec<ShcFile>,
    pub total_results: u64,
    pub total_pages: u64,
    pub current_page: u64,
    pub previous_page: Option<u64>,
    pub next_page: Option<u64>,
    pub per_page: u64,
}

pub async fn list_files(
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

    let mut table = Table::new();
    table.add_row(row![
        "S/N",
        "Name",
        "Size",
        "Visibility",
        "Status",
        "Updated At",
        "Shareable Link"
    ]);

    let mut file_index = 0;
    for file in &res.results {
        file_index += 1;
        let updated_at = DateTime::<Utc>::from(DateTime::parse_from_rfc3339(&file.updated_at)?);
        let shareable_link = format!("https://shc.ajaysharma.dev/files/{}", file.id);
        let size = if file.size < 1024 {
            format!("{:.3} KB", file.size as f64 / 1024.0)
        } else {
            format!("{:.3} MB", file.size as f64 / 1024.0 / 1024.0)
        };

        let visibility = if file.is_public { "Public" } else { "Private" };

        table.add_row(Row::new(vec![
            Cell::new(&format!("{:02}", file_index)),
            Cell::new(&file.name),
            Cell::new(&size),
            Cell::new(visibility),
            Cell::new(&file.upload_status),
            Cell::new(&updated_at.format("%Y-%m-%d %H:%M").to_string()),
            Cell::new(&shareable_link.as_str()),
        ]));
    }
    console::Term::stdout().write_line(format!("\nFiles Count: {}\n", res.total_results).as_str())?;
    table.printstd();

    Ok(())
}
