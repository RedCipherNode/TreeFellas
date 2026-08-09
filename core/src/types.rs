use std::path::PathBuf;
use std::time::SystemTime;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Metadata {
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub readonly: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,

    pub is_directory: bool,

    pub size: u64,
    pub allocated_size: u64,

    pub file_count: u64,
    pub directory_count: u64,

    pub metadata: Metadata,

    pub children: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntrySummary {
    pub path: String,
    pub size: u64,
    pub allocated_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionStatistic {
    pub extension: String,
    pub count: u64,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Analysis {
    pub total_size: u64,
    pub total_allocated_size: u64,

    pub total_files: u64,
    pub total_directories: u64,

    pub largest_files: Vec<EntrySummary>,
    pub largest_directories: Vec<EntrySummary>,

    pub extension_statistics: Vec<ExtensionStatistic>,

    pub empty_files: Vec<EntrySummary>,
    pub empty_directories: Vec<EntrySummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub tree: Entry,
    pub analysis: Analysis,
}
