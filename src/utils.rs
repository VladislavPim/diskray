// Utility functions for DiskRay
use std::path::Path;

/// Format file size for display
pub fn format_size(size: u64) -> String {
    humansize::format_size(size, humansize::DECIMAL)
}

/// Get file extension from path
pub fn get_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}