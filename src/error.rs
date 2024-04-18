pub fn file_read_error(file_path: &std::path::Path) -> String {
    format!("Failed to read {}", file_path.display())
}

pub fn file_write_error(file_path: &std::path::Path) -> String {
    format!("Failed to write {}", file_path.display())
}
