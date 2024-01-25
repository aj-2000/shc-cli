use chrono::{DateTime, Utc};
use dialoguer::{theme, Select};

use crate::models::ShcFile;
use crate::utils::format_bytes;

pub fn shc_file_input(files: &Vec<ShcFile>, prompt: &str) -> usize {
    let files = files
        .iter()
        .map(|file| -> Result<String, Box<dyn std::error::Error>> {
            let updated_at = DateTime::<Utc>::from(DateTime::parse_from_rfc3339(&file.updated_at)?)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            let size = format_bytes(file.size);
            let visibility = if file.is_public {
                "Public".to_string()
            } else {
                "Private".to_string()
            };
            Ok(format!(
                "{}  {}  {} {}",
                file.name, size, updated_at, visibility
            ))
        })
        .collect::<Result<Vec<String>, Box<dyn std::error::Error>>>();

    let files = match files {
        Ok(items) => items,
        Err(_) => vec![],
    };

    let selection = Select::with_theme(&theme::ColorfulTheme::default())
        .max_length(20)
        .with_prompt(prompt)
        .default(0)
        .items(&files)
        .interact()
        .unwrap();

    selection
}
