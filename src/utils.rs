use std::fs::{self, File};
use std::io::{self};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{CompressionMethod::Bzip2, ZipWriter};

pub fn format_bytes(bytes: u64) -> String {
    let mut bytes = bytes as f64;
    let mut unit = 0;

    while bytes >= 1000.0 {
        bytes /= 1000.0;
        unit += 1;
    }

    let unit = match unit {
        0 => "B",
        1 => "KB",
        2 => "MB",
        3 => "GB",
        4 => "TB",
        5 => "PB",
        6 => "EB",
        7 => "ZB",
        _ => "YB",
    };

    format!("{:.2} {}", bytes, unit)
}

pub fn zip_directory_recursive(src_dir: &Path, size_limit: u64) -> io::Result<PathBuf> {
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
