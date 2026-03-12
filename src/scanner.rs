use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use jwalk::{WalkDir, Parallelism};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use parking_lot::Mutex;

const IGNORED_DIRS: &[&str] = &["$Recycle.Bin", "System Volume Information"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_directory: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub modified: DateTime<Utc>,
    pub extension: Option<String>,
    pub parent: Option<PathBuf>,
    pub children: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub root_path: PathBuf,
    pub total_size: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub entries: Vec<FileEntry>,
    pub scan_duration: std::time::Duration,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub scan_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub current_path: PathBuf,
    pub is_complete: bool,
}

pub struct FileSystemScanner {
    root_path: PathBuf,
    should_stop: Arc<AtomicBool>,
    files_scanned: Arc<AtomicU64>,
    bytes_scanned: Arc<AtomicU64>,
    current_path: Arc<Mutex<PathBuf>>,
    is_complete: Arc<AtomicBool>,
    result: Arc<Mutex<Option<ScanResult>>>,
}

impl FileSystemScanner {
    pub fn new(path: PathBuf) -> Self {
        Self {
            root_path: path,
            should_stop: Arc::new(AtomicBool::new(false)),
            files_scanned: Arc::new(AtomicU64::new(0)),
            bytes_scanned: Arc::new(AtomicU64::new(0)),
            current_path: Arc::new(Mutex::new(PathBuf::new())),
            is_complete: Arc::new(AtomicBool::new(false)),
            result: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&mut self) {
        let root_path = self.root_path.clone();
        let should_stop = self.should_stop.clone();
        let files_scanned = self.files_scanned.clone();
        let bytes_scanned = self.bytes_scanned.clone();
        let current_path = self.current_path.clone();
        let is_complete = self.is_complete.clone();
        let result = self.result.clone();

        std::thread::spawn(move || {
            let scan_result = Self::scan_directory(
                &root_path,
                &should_stop,
                &files_scanned,
                &bytes_scanned,
                &current_path,
            );
            match scan_result {
                Ok(res) => {
                    *result.lock() = Some(res);
                }
                Err(e) => {
                    eprintln!("Scan error: {}", e);
                }
            }
            is_complete.store(true, Ordering::SeqCst);
        });
    }

    fn scan_directory(
        root: &Path,
        should_stop: &AtomicBool,
        files_scanned: &AtomicU64,
        bytes_scanned: &AtomicU64,
        current_path: &Mutex<PathBuf>,
    ) -> Result<ScanResult> {
        let scan_start = std::time::Instant::now();

        let walker = WalkDir::new(root)
            .follow_links(false)
            .parallelism(Parallelism::Serial);

        let mut entries = Vec::new();
        let mut path_to_index = HashMap::new();

        for dir_entry_result in walker {
            if should_stop.load(Ordering::Relaxed) {
                break;
            }

            let dir_entry = match dir_entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = dir_entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if IGNORED_DIRS.contains(&file_name) {
                continue;
            }

            let metadata = match dir_entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let is_dir = metadata.is_dir();
            let size = if is_dir { 0 } else { metadata.len() };
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_lowercase());
            let modified = metadata
                .modified()
                .map(|t| DateTime::<Utc>::from(t))
                .unwrap_or_else(|_| Utc::now());
            let name = file_name.to_string();
            let parent = path.parent().map(|p| p.to_path_buf());

            files_scanned.fetch_add(1, Ordering::Relaxed);
            if !is_dir {
                bytes_scanned.fetch_add(size, Ordering::Relaxed);
            }
            *current_path.lock() = path.to_path_buf();

            let entry = FileEntry {
                path: path.to_path_buf(),
                name,
                size,
                is_directory: is_dir,
                modified,
                extension,
                parent,
                children: Vec::new(),
            };
            let idx = entries.len();
            path_to_index.insert(entry.path.clone(), idx);
            entries.push(entry);
        }

        for i in 0..entries.len() {
            if let Some(parent) = &entries[i].parent {
                if let Some(&parent_idx) = path_to_index.get(parent) {
                    let child_path = entries[i].path.clone();
                    entries[parent_idx].children.push(child_path);
                }
            }
        }

        Self::calculate_directory_sizes(&mut entries, &path_to_index);

        let file_count = entries.iter().filter(|e| !e.is_directory).count() as u64;
        let dir_count = entries.iter().filter(|e| e.is_directory).count() as u64;
        let total_size = entries.iter().filter(|e| !e.is_directory).map(|e| e.size).sum();

        let scan_duration = scan_start.elapsed();

        Ok(ScanResult {
            root_path: root.to_path_buf(),
            total_size,
            file_count,
            dir_count,
            entries,
            scan_duration,
            scan_time: Utc::now(),
        })
    }

    fn calculate_directory_sizes(entries: &mut [FileEntry], path_to_index: &HashMap<PathBuf, usize>) {
        let mut dir_indices: Vec<usize> = entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_directory)
            .map(|(idx, _)| idx)
            .collect();

        dir_indices.sort_by(|&a, &b| {
            let depth_a = entries[a].path.components().count();
            let depth_b = entries[b].path.components().count();
            depth_b.cmp(&depth_a)
        });

        for idx in dir_indices {
            let mut dir_size = 0;
            for child_path in &entries[idx].children {
                if let Some(&child_idx) = path_to_index.get(child_path) {
                    dir_size += entries[child_idx].size;
                }
            }
            entries[idx].size = dir_size;
        }
    }

    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::SeqCst);
    }

    pub fn is_finished(&self) -> bool {
        self.is_complete.load(Ordering::SeqCst)
    }

    pub fn take_result(&mut self) -> Option<ScanResult> {
        self.result.lock().take()
    }

    pub fn get_progress(&self) -> ScanProgress {
        ScanProgress {
            files_scanned: self.files_scanned.load(Ordering::Relaxed),
            bytes_scanned: self.bytes_scanned.load(Ordering::Relaxed),
            current_path: self.current_path.lock().clone(),
            is_complete: self.is_complete.load(Ordering::SeqCst),
        }
    }
}