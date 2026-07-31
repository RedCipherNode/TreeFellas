use std::fs::{self, DirEntry};
use std::path::Path;

use crate::types::{Entry, Metadata};

pub struct Scanner;

impl Scanner {
    pub fn scan<P: AsRef<Path>>(path: P) -> std::io::Result<Entry> {
        Self::scan_path(path.as_ref())
    }

    fn scan_path(path: &Path) -> std::io::Result<Entry> {
        // Jangan follow symlink/junction
        let metadata = fs::symlink_metadata(path)?;

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

            metadata: Metadata {
                created: None,
                modified: None,
                readonly: metadata.permissions().readonly(),

                #[cfg(windows)]
                hidden: false,

                #[cfg(not(windows))]
                hidden: false,
            },

            children: Vec::new(),
        };

        // Skip symlink / junction
        if metadata.file_type().is_symlink() {
            return Ok(entry);
        }

        if metadata.is_file() {
            entry.size = metadata.len();
            entry.file_count = 1;

            return Ok(entry);
        }

        entry.directory_count = 1;

        // Jangan gagal kalau folder tidak bisa dibuka
        let children = match fs::read_dir(path) {
            Ok(children) => children,
            Err(_) => return Ok(entry),
        };

        for child in children {
            let child = match child {
                Ok(child) => child,
                Err(_) => continue,
            };

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
        let path = child.path();

        Self::scan_path(path.as_path())
    }
}
