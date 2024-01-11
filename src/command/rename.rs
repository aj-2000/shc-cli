use chrono::{DateTime, Utc};
use dialoguer::{Confirm, Editor, Select};
use serde_json::json;

use crate::command::list::File;
use crate::consts;

pub async fn rename_file(
    search: &str,
    user_id: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let res = client
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

    let selection = Select::new()
        .with_prompt("Which file do you want to delete?")
        .items(&items)
        .interact()
        .unwrap();

    if let Some(new_filename) = Editor::new().edit("New filename").unwrap() {
        let confirm = Confirm::new()
            .with_prompt("Are you sure?")
            .default(false)
            .interact()
            .unwrap();

        if !confirm {
            println!("Aborted");
            return Ok(());
        } else {
            print!("renaming file...");
            let file_id = res[selection].id.clone();
            let body = json!({
                "name": new_filename,
            });
            let res = client
                .patch(format!(
                    "{}/api/file/rename/{}",
                    consts::SHC_BACKEND_API_BASE_URL,
                    file_id
                ))
                .header("user_id", user_id)
                .header("user_password", password)
                .json(&body)
                .send()
                .await?;

            if res.status().is_success() {
                println!("Done");
            } else {
                println!("Failed");
            }
        }
    } else {
        println!("Abort!");
    }

    Ok(())
}
