//! Utility helpers — size formatting, directory traversal.

use std::path::Path;

use crate::cli_handlers::KIB_F64;

#[tracing::instrument]
pub fn human_readable_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= KIB_F64 && unit_idx < UNITS.len() - 1 {
        size /= KIB_F64;
        unit_idx += 1;
    }
    format!("{:.2} {}", size, UNITS[unit_idx])
}

pub fn dir_size(path: &Path) -> std::io::Result<usize> {
    let mut total = 0usize;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            if ft.is_dir() {
                total += dir_size(&entry.path())?;
            } else {
                total += entry.metadata()?.len() as usize;
            }
        }
    }
    Ok(total)
}
