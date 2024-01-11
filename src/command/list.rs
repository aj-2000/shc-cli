use chrono::{DateTime, Utc};
use prettytable::{row, Cell, Row, Table};
use serde::{Deserialize, Serialize};

use crate::consts;

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub name: String,
    pub id: String,
    pub extension: String,
    pub mime_type: String,
    pub size: u64,
    pub updated_at: String,
    pub user_id: String,
    pub r2_path: String,
}

pub async fn list_files(
    search: &str,
    user_id: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let res = &client
        .get(format!(
            "{}/api/file/list?search={}",
            consts::SHC_BACKEND_API_BASE_URL,
            search
        ))
        .header("user_id", user_id)
        .header("user_password", password)
        .send()
        .await?
        .json::<Vec<File>>()
        .await?;

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
        let shareable_link = format!("https://sharecode.com/file/{}", file.id);
        let size = if file.size < 1024 {
            format!("{:.3} KB", file.size as f64 / 1024.0)
        } else {
            format!("{:.3} MB", file.size as f64 / 1024.0 / 1024.0)
        };

        table.add_row(Row::new(vec![
            Cell::new(&format!("{:02}", file_index)),
            Cell::new(&file.name),
            Cell::new(&size),
            Cell::new("Public".to_string().as_str()),
            Cell::new(&updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            Cell::new(&shareable_link.as_str()),
        ]));
    }
    console::Term::stdout().write_line(format!("Files Count: {}", res.len()).as_str())?;
    table.printstd();

    Ok(())
}
