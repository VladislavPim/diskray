use crate::scanner::{ScanResult, FileEntry};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use humansize::{format_size, DECIMAL};

/// Categories for file classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileCategory {
    Documents,
    Images,
    Videos,
    Audio,
    Archives,
    Executables,
    Code,
    Data,
    System,
    Hidden,
    Temporary,
    Other,
}

/// File type information
#[derive(Debug, Clone)]
pub struct FileTypeInfo {
    pub category: FileCategory,
    pub extensions: Vec<String>,
    pub description: String,
}

/// Analysis filters
#[derive(Debug, Clone, Default)]
pub struct AnalysisFilters {
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub categories: HashSet<FileCategory>,
    pub extensions: HashSet<String>,
    pub show_hidden: bool,
    pub show_system: bool,
}

/// Statistics for a single extension
#[derive(Debug, Clone)]
pub struct ExtensionStats {
    pub extension: String,
    pub total_size: u64,
    pub file_count: usize,
}

/// Main disk analyzer
pub struct DiskAnalyzer {
    file_types: Vec<FileTypeInfo>,
    duplicate_cache: HashMap<u64, Vec<PathBuf>>,
    large_files: Vec<FileEntry>,
    old_files: Vec<FileEntry>,
    analysis_time: std::time::Instant,
    extension_stats_cache: Option<Vec<ExtensionStats>>,
}

impl DiskAnalyzer {
    pub fn new() -> Self {
        Self {
            file_types: Self::build_file_types(),
            duplicate_cache: HashMap::new(),
            large_files: Vec::new(),
            old_files: Vec::new(),
            analysis_time: std::time::Instant::now(),
            extension_stats_cache: None,
        }
    }

    /// Analyze scan results
    pub fn analyze(&mut self, scan_result: &ScanResult) {
        self.analysis_time = std::time::Instant::now();

        // Find large files (top 100 by size)
        let mut all_files: Vec<&FileEntry> = scan_result
            .entries
            .iter()
            .filter(|e| !e.is_directory)
            .collect();

        all_files.sort_by(|a, b| b.size.cmp(&a.size));
        self.large_files = all_files
            .iter()
            .take(100)
            .map(|&e| e.clone())
            .collect();

        // Find old files (modified more than 1 year ago)
        let one_year_ago = chrono::Utc::now() - chrono::Duration::days(365);
        self.old_files = scan_result
            .entries
            .iter()
            .filter(|e| !e.is_directory && e.modified < one_year_ago)
            .cloned()
            .collect();

        self.old_files.sort_by(|a, b| a.modified.cmp(&b.modified));

        // Build duplicate cache (group by size as first pass)
        self.duplicate_cache.clear();
        for entry in &scan_result.entries {
            if !entry.is_directory && entry.size > 0 {
                self.duplicate_cache
                    .entry(entry.size)
                    .or_insert_with(Vec::new)
                    .push(entry.path.clone());
            }
        }

        // Remove sizes with only one file
        self.duplicate_cache.retain(|_, paths| paths.len() > 1);

        // Update extension statistics cache
        self.update_extension_stats(scan_result);
    }

    /// Update the cached extension statistics
    fn update_extension_stats(&mut self, scan_result: &ScanResult) {
        let mut map = HashMap::new();
        for entry in &scan_result.entries {
            if entry.is_directory {
                continue;
            }
            let ext = entry.extension.clone().unwrap_or_else(|| "".to_string());
            let entry_stats = map.entry(ext).or_insert((0, 0));
            entry_stats.0 += entry.size;
            entry_stats.1 += 1;
        }
        let mut list: Vec<_> = map
            .into_iter()
            .map(|(ext, (size, count))| ExtensionStats {
                extension: if ext.is_empty() {
                    "(no extension)".to_string()
                } else {
                    ext
                },
                total_size: size,
                file_count: count,
            })
            .collect();
        list.sort_by(|a, b| b.total_size.cmp(&a.total_size));
        self.extension_stats_cache = Some(list);
    }

    /// Get cached extension statistics
    pub fn get_extension_stats(&self, _scan_result: &ScanResult) -> &[ExtensionStats] {
        self.extension_stats_cache.as_deref().unwrap_or(&[])
    }

    /// Get statistics by file category
    pub fn get_category_stats(&self, scan_result: &ScanResult) -> HashMap<FileCategory, CategoryStats> {
        let mut stats = HashMap::new();

        for entry in &scan_result.entries {
            if !entry.is_directory {
                let category = self.categorize_file(entry);
                let stat = stats.entry(category).or_insert_with(CategoryStats::default);
                stat.total_size += entry.size;
                stat.file_count += 1;
                stat.files.push(entry.clone());
            }
        }

        // Sort files within each category by size
        for stat in stats.values_mut() {
            stat.files.sort_by(|a, b| b.size.cmp(&a.size));
        }

        stats
    }

    /// Categorize a file based on its extension
    pub fn categorize_file(&self, entry: &FileEntry) -> FileCategory {
        if let Some(ext) = &entry.extension {
            for file_type in &self.file_types {
                if file_type.extensions.contains(&ext.to_lowercase()) {
                    return file_type.category;
                }
            }
        }

        // Check for hidden/system files
        if entry.name.starts_with('.') {
            return FileCategory::Hidden;
        }

        if entry.path.to_string_lossy().contains("node_modules")
            || entry.path.to_string_lossy().contains("target/")
            || entry.path.to_string_lossy().contains(".git/")
        {
            return FileCategory::System;
        }

        FileCategory::Other
    }

    /// Get file type information
    pub fn get_file_type_info(&self, extension: &str) -> Option<&FileTypeInfo> {
        let ext_lower = extension.to_lowercase();
        self.file_types
            .iter()
            .find(|ft| ft.extensions.contains(&ext_lower))
    }

    /// Find potential duplicates (files with same size)
    pub fn find_potential_duplicates(&self) -> Vec<DuplicateGroup> {
        let mut duplicates = Vec::new();

        for (size, paths) in &self.duplicate_cache {
            if paths.len() > 1 {
                duplicates.push(DuplicateGroup {
                    size: *size,
                    paths: paths.clone(),
                });
            }
        }

        duplicates.sort_by(|a, b| b.paths.len().cmp(&a.paths.len()));
        duplicates
    }

    /// Get largest files
    pub fn get_largest_files(&self, count: usize) -> &[FileEntry] {
        &self.large_files[..count.min(self.large_files.len())]
    }

    /// Get oldest files
    pub fn get_oldest_files(&self, count: usize) -> &[FileEntry] {
        &self.old_files[..count.min(self.old_files.len())]
    }

    /// Build file type database
    fn build_file_types() -> Vec<FileTypeInfo> {
        vec![
            FileTypeInfo {
                category: FileCategory::Documents,
                extensions: vec![
                    "pdf", "doc", "docx", "txt", "rtf", "odt", "pages",
                    "xls", "xlsx", "csv", "ods", "numbers", "ppt", "pptx",
                    "key", "md", "tex", "epub", "mobi"
                ].iter().map(|s| s.to_string()).collect(),
                description: "Documents and Office files".to_string(),
            },
            FileTypeInfo {
                category: FileCategory::Images,
                extensions: vec![
                    "jpg", "jpeg", "png", "gif", "bmp", "tiff", "svg",
                    "webp", "raw", "ico", "psd", "ai", "eps"
                ].iter().map(|s| s.to_string()).collect(),
                description: "Image files".to_string(),
            },
            FileTypeInfo {
                category: FileCategory::Videos,
                extensions: vec![
                    "mp4", "avi", "mkv", "mov", "wmv", "flv", "m4v",
                    "mpg", "mpeg", "webm", "3gp", "m2ts"
                ].iter().map(|s| s.to_string()).collect(),
                description: "Video files".to_string(),
            },
            FileTypeInfo {
                category: FileCategory::Audio,
                extensions: vec![
                    "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a",
                    "opus", "aiff", "alac"
                ].iter().map(|s| s.to_string()).collect(),
                description: "Audio files".to_string(),
            },
            FileTypeInfo {
                category: FileCategory::Archives,
                extensions: vec![
                    "zip", "rar", "7z", "tar", "gz", "bz2", "xz",
                    "iso", "dmg", "pkg", "deb", "rpm"
                ].iter().map(|s| s.to_string()).collect(),
                description: "Archive files".to_string(),
            },
            FileTypeInfo {
                category: FileCategory::Executables,
                extensions: vec![
                    "exe", "dll", "so", "dylib", "app", "msi", "bat",
                    "sh", "bin", "jar", "apk", "ipa"
                ].iter().map(|s| s.to_string()).collect(),
                description: "Executable files".to_string(),
            },
            FileTypeInfo {
                category: FileCategory::Code,
                extensions: vec![
                    "rs", "py", "js", "ts", "java", "cpp", "c", "h",
                    "cs", "go", "php", "rb", "swift", "kt", "scala",
                    "html", "css", "json", "xml", "yml", "yaml", "toml"
                ].iter().map(|s| s.to_string()).collect(),
                description: "Source code files".to_string(),
            },
            FileTypeInfo {
                category: FileCategory::Data,
                extensions: vec![
                    "db", "sqlite", "sql", "mdb", "accdb", "json",
                    "xml", "csv", "tsv", "parquet", "feather", "hdf5"
                ].iter().map(|s| s.to_string()).collect(),
                description: "Database and data files".to_string(),
            },
        ]
    }
}

/// Statistics for a file category
#[derive(Debug, Clone, Default)]
pub struct CategoryStats {
    pub total_size: u64,
    pub file_count: u64,
    pub files: Vec<FileEntry>,
}

/// Group of potential duplicate files
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub size: u64,
    pub paths: Vec<PathBuf>,
}

impl CategoryStats {
    /// Format size for display
    pub fn formatted_size(&self) -> String {
        format_size(self.total_size, DECIMAL)
    }

    /// Percentage of total
    pub fn percentage_of(&self, total: u64) -> f32 {
        if total == 0 {
            0.0
        } else {
            (self.total_size as f64 / total as f64 * 100.0) as f32
        }
    }
}