use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::min;
use std::fs::{self, File};
use std::io::{self};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;
use zip::write::FileOptions;
use zip::{CompressionMethod::Bzip2, ZipWriter};

use crate::consts;

#[derive(Deserialize, Serialize, Clone)]
struct AddFileResponse {
    upload_url: String,
    file_id: String,
    file_name: String,
    is_public: bool,
}

fn zip_directory_recursive(src_dir: &Path, size_limit: u64) -> io::Result<PathBuf> {
    let dest_file_path = src_dir
        .file_name()
        .map(|name| PathBuf::from(name.to_string_lossy().into_owned() + ".zip"))
        .unwrap_or_else(|| PathBuf::from("archive.zip"));

    let dest_file = File::create(&dest_file_path)?;

    let mut zip = ZipWriter::new(dest_file);

    fn zip_inner(
        path: &Path,
        zip: &mut ZipWriter<File>,
        base_path: &Path,
        size_limit: u64,
        current_size: &mut u64,
    ) -> io::Result<u64> {
        let mut total_size = 0;

        if path.is_file() {
            let relative_path = path.strip_prefix(base_path).unwrap();
            let zip_path = relative_path.to_string_lossy();
            let options = FileOptions::default()
                .compression_method(Bzip2)
                .unix_permissions(0o755);

            zip.start_file(zip_path, options)?;
            let mut file = File::open(path)?;

            let file_size = io::copy(&mut file, zip)?;
            total_size += file_size;
            *current_size += file_size;

            if *current_size > size_limit {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Exceeded size limit for zip file",
                ));
            }
        } else if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                total_size += zip_inner(&entry_path, zip, base_path, size_limit, current_size)?;
            }
        }

        Ok(total_size)
    }

    let src_dir = fs::canonicalize(src_dir)?;

    let mut current_size = 0;

    let _total_size = zip_inner(&src_dir, &mut zip, &src_dir, size_limit, &mut current_size)?;
    Ok(dest_file_path)
}

pub async fn upload_file(
    file_path: &PathBuf,
    access_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !file_path.exists() {
        println!("ShcFile or Folder does not exist");
        return Ok(());
    }

    let is_dir = file_path.is_dir();
    let file_path = if is_dir {
        let pb = ProgressBar::new_spinner();

        pb.enable_steady_tick(Duration::from_millis(200));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.dim.bold} shc: {wide_msg}")
                .unwrap()
                .tick_chars("/|\\- "),
        );

        pb.set_message("Compressing folder...");
        let zip_file_path = zip_directory_recursive(&file_path, 30 * 1024 * 1024)?;
        pb.finish_and_clear();
        zip_file_path
    } else {
        file_path.clone()
    };

    let file_name = file_path.file_name().unwrap().to_str().unwrap();
    let mime_type = mime_guess::from_path(&file_path).first_or_octet_stream();
    let file = tokio::fs::File::open(&file_path)
        .await
        .expect("Cannot open input file for HTTPS read");
    let total_size = file
        .metadata()
        .await
        .expect("Cannot determine input file size for HTTPS read")
        .len();
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
            "{}/api/files/add",
            consts::SHC_BACKEND_API_BASE_URL
        ))
        .json(&json!(
            {
                "file_name": file_name,
                "mime_type": mime_type.as_ref(),
                "file_size": total_size,
            }
        ))
        .header("Authorization", access_token)
        .send()
        .await?;
    pb.finish_and_clear();

    let res: AddFileResponse = res.json().await?;
    let file_id = res.file_id;
    let file_name = res.file_name;
    let upload_url = res.upload_url;
    // let user_id = res.user_id;

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
    let res = client
        .patch(format!(
            "{}/api/files/update-upload-status/{}",
            consts::SHC_BACKEND_API_BASE_URL,
            file_id
        ))
        .json(&json!(
            {
                "upload_status": "uploading",
            }
        ))
        .header("Authorization", access_token)
        .send()
        .await?;

    if !res.status().is_success() {
        println!("Failed to add file");
        return Ok(());
    }

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
        .put(upload_url)
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
        .patch(format!(
            "{}/api/files/update-upload-status/{}",
            consts::SHC_BACKEND_API_BASE_URL,
            file_id
        ))
        .json(&json!(
            {
                "upload_status": "uploaded",
            }
        ))
        .header("Authorization", access_token)
        .send()
        .await?;

    if !res.status().is_success() {
        println!("Failed to add file");
        return Ok(());
    }
    pb.finish_and_clear();

    if res.status().is_success() {
        print!(
            "\n{} added successfully\nShcFile Link: https://shc.ajaysharma.dev/files/{}\n",
            file_name, file_id
        );
    } else {
        println!("Failed to add file");
    }

    // Delete the zip file if it was created by the app
    if is_dir {
        std::fs::remove_file(&file_path)?;
    }

    Ok(())
}
