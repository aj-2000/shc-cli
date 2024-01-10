use serde::{Deserialize, Serialize};
use console::Term;
use prettytable::{Table, Row, Cell, row};
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize, Clone, Debug)]
struct File {
    name: String,
    id: String,
    extension: String,
    mime_type: String,
    size: u64,
    updated_at: String,
    user_id: String,
    r2_path: String,
}
    

pub async fn list_files(
    search: &String,
    user_id: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let term = Term::stdout();

    let res = client
        .get("http://localhost:6969/api/file/list")
        .header("user_id", user_id)
        .header("user_password", password)
        .send()
        .await?
        .json::<Vec<File>>() 
        .await?;

    let mut table = Table::new();
    table.add_row(row!["Name", "Size", "Updated At", "R2 Path"]);

    for file in res {
        let updated_at = DateTime::<Utc>::from(DateTime::parse_from_rfc3339(&file.updated_at)?);
        let r2_path = ".../".to_string() + file.r2_path.split('/').last().unwrap_or("");

        let size = if file.size < 1024 {
            format!("{:.3} KB", file.size as f64 / 1024.0)
        } else {
            format!("{:.3} MB", file.size as f64 / 1024.0 / 1024.0)
        };

        table.add_row(Row::new(vec![
            Cell::new(&file.name),
            Cell::new(&size),
            Cell::new(&updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            Cell::new(&r2_path.as_str()),
        ]));
    }

    console::Term::stdout().hide_cursor()?;
    console::Term::stdout().write_line("Files:")?;
    table.printstd();

    Ok(())
}