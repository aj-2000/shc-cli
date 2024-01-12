use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::min;
use std::path::PathBuf;
use std::time::Duration;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

use crate::consts;
use indicatif::{ProgressBar, ProgressStyle};

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
    let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();

    let client = reqwest::Client::new();

    let pb = ProgressBar::new_spinner();

    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim.bold} shc: {wide_msg}")
            .unwrap()
            .tick_chars("/|\\- "),
    );

    pb.set_message("Preparing for upload...");

    let res = client
        .post(format!(
            "{}/api/file/upload-url",
            consts::SHC_BACKEND_API_BASE_URL
        ))
        .json(&json!(
            {
                "file_name": file_name,
                "mime_type": mime_type.as_ref()
            }
        ))
        .header("user_id", user_id)
        .header("user_password", password)
        .send()
        .await?;
    pb.finish_and_clear();

    let res: GetUploadUrlResponse = res.json().await?;
    let r2_path = res.r2_path;

    let file = tokio::fs::File::open(&file_path)
        .await
        .expect("Cannot open input file for HTTPS read");
    let total_size = file
        .metadata()
        .await
        .expect("Cannot determine input file size for HTTPS read")
        .len();

    let mut uploaded = 0;

    let mut reader_stream = ReaderStream::new(file);
    let bar = ProgressBar::new(total_size);
    bar.set_style(
        ProgressStyle::with_template(
            "{msg}\n{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {bytes_per_sec} \n",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    bar.reset_eta();
    bar.set_message(format!("Uploading {}", file_name));
    let async_stream = async_stream::stream! {
        while let Some(chunk) = reader_stream.next().await {
            if let Ok(chunk) = &chunk {
                let new = min(uploaded + (chunk.len() as u64), total_size);
                uploaded = new;
                bar.set_position(new);
                if uploaded >= total_size {
                    //TODO: fix this
                        bar.finish_and_clear();
                }
            }
            yield chunk;
        }
    };
    let _ = client
        .put(&res.upload_url)
        .body(reqwest::Body::wrap_stream(async_stream))
        .header("Content-Type", mime_type.as_ref())
        .header("Content-Length", total_size.to_string())
        .send()
        .await?;

    let pb = ProgressBar::new_spinner();

    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim.bold} shc: {wide_msg}")
            .unwrap()
            .tick_chars("/|\\- "),
    );

    pb.set_message("Adding file...");

    let res = client
        .post(format!("{}/api/file/add", {
            consts::SHC_BACKEND_API_BASE_URL
        }))
        .json(&json!({
            "name": file_name,
            "extension": file_name.split(".").last().unwrap(),
            "r2_path": r2_path,
            "mime_type": mime_type.as_ref(),
            "size": total_size,
        }))
        .header("user_id", user_id)
        .header("user_password", password)
        .send()
        .await?;

    pb.finish_and_clear();

    if res.status().is_success() {
        println!("File added successfully");
    } else {
        println!("Failed to add file");
    }

    Ok(())
}
