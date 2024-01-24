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
