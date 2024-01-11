use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

use crate::consts;

#[derive(Deserialize, Serialize, Clone)]
struct GetUploadUrlResponse {
    upload_url: String,
    r2_path: String,
}
pub async fn upload_file(
    file_path: &PathBuf,
    user_id: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !file_path.exists() {
        println!("File does not exist");
        return Ok(());
    }
    let file_name = file_path.file_name().unwrap().to_str().unwrap();
    let file = fs::read(file_path).unwrap();
    let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();

    let client = reqwest::Client::new();
    let mut map = HashMap::new();

    map.insert("file_name", file_name);
    map.insert("mime_type", mime_type.as_ref());

    let res = client
        .post(format!(
            "{}/api/file/upload-url",
            consts::SHC_BACKEND_API_BASE_URL
        ))
        .json(&map)
        .header("user_id", user_id)
        .header("user_password", password)
        .send()
        .await?;
    println!("Status-geturl: {}", res.status());

    let res1: GetUploadUrlResponse = res.json().await?;

    let res = client
        .put(&res1.upload_url)
        .body(file.to_owned())
        .header("Content-Type", mime_type.as_ref())
        .send()
        .await?;

    if res.status().is_success() {
        // Successful upload
        println!("File uploaded successfully");
    } else if res.status() == reqwest::StatusCode::FORBIDDEN {
        // Handle 403 Forbidden error
        println!("Upload failed: 403 Forbidden");
    } else {
        // Handle other errors
        println!("Upload failed with status: {}", res.status());
    }
    println!("Status-R2: {}", res.status());

    let mut map = HashMap::new();

    map.insert("name", file_name);
    map.insert("extension", file_name.split(".").last().unwrap());
    map.insert("r2_path", res1.r2_path.as_str());
    map.insert("mime_type", mime_type.as_ref());

    let size = file.len().to_string();
    map.insert("size", &size);

    let res = client
        .post(format!("{}/api/file/add", {
            consts::SHC_BACKEND_API_BASE_URL
        }))
        .json(&map)
        .header("user_id", user_id)
        .header("user_password", password)
        .send()
        .await?;

    println!("Status: {}", res.status());

    Ok(())
}
