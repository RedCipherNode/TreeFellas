use std::fs::{self, DirEntry};
use std::path::Path;

use crate::types::{Entry, Metadata};

pub struct Scanner;

impl Scanner {
    pub fn scan<P: AsRef<Path>>(path: P) -> std::io::Result<Entry> {
        Self::scan_path(path.as_ref())
    }

    fn scan_path(path: &Path) -> std::io::Result<Entry> {
        let metadata = fs::metadata(path)?;

        let mut entry = Entry {
            name: path
                .file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy()
                .into_owned(),

            path: path.to_path_buf(),

            is_directory: metadata.is_dir(),

            size: 0,
            file_count: 0,
            directory_count: 0,

            // Metadata detail diambil nanti saat dibutuhkan
            metadata: Metadata {
                created: None,
                modified: None,
                readonly: false,

                #[cfg(windows)]
                hidden: false,

                #[cfg(not(windows))]
                hidden: false,
            },

            children: Vec::new(),
        };

        if metadata.is_file() {
            entry.size = metadata.len();
            entry.file_count = 1;

            return Ok(entry);
        }

        entry.directory_count = 1;

        for child in fs::read_dir(path)? {
            let child = child?;

            if let Ok(child_entry) = Self::scan_dir_entry(child) {
                entry.size += child_entry.size;
                entry.file_count += child_entry.file_count;
                entry.directory_count += child_entry.directory_count;

                entry.children.push(child_entry);
            }
        }

        Ok(entry)
    }

    fn scan_dir_entry(child: DirEntry) -> std::io::Result<Entry> {
        Self::scan_path(&child.path())
    }
}
