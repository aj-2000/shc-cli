use chrono::{DateTime, Utc};
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{row, Cell, Row, Table};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::consts;

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub name: String,
    pub id: String,
    pub extension: String,
    pub mime_type: String,
    pub size: u64,
    pub is_public: bool,
    pub updated_at: String,
    pub user_id: String,
    pub r2_path: String,
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
            "{}/api/files/list?search={}",
            consts::SHC_BACKEND_API_BASE_URL,
            search
        ))
        .header("Authorization", access_token)
        .send()
        .await?
        .json::<Vec<File>>()
        .await?;
    pb.finish_and_clear();

    let mut table = Table::new();
    table.add_row(row![
        "S/N",
        "Name",
        "Size",
        "Visibility",
        "Updated At",
        "Shareable Link"
    ]);

    let mut file_index = 0;
    for file in res {
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
            Cell::new(&updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            Cell::new(&shareable_link.as_str()),
        ]));
    }
    console::Term::stdout().write_line(format!("\nFiles Count: {}\n", res.len()).as_str())?;
    table.printstd();

    Ok(())
}
