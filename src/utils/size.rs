const SIZE_IN_MB: f64 = 1024.0 * 1024.0 * 1024.0; // 1Mb

pub fn bytes_to_gb_string(size: u64)-> String{
    format!("{:.2} GiB", size as f64 / SIZE_IN_MB)
}

pub fn bytes_to_percent_string(size: f64)-> String{
    format!("{:.2}%", size)
}
