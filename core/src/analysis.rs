use std::collections::HashMap;

use crate::types::{Analysis, Entry, EntrySummary, ExtensionStatistic};

pub fn analyze(root: &Entry) -> Analysis {
    let mut analysis = Analysis {
        total_size: root.size,
        total_files: root.file_count,
        total_directories: root.directory_count,
        total_allocated_size: root.allocated_size,

        largest_files: Vec::new(),
        largest_directories: Vec::new(),

        extension_statistics: Vec::new(),

        empty_files: Vec::new(),
        empty_directories: Vec::new(),
    };

    let mut extensions: HashMap<String, (u64, u64)> = HashMap::new();

    visit(root, &mut analysis, &mut extensions);

    analysis
        .largest_files
        .sort_unstable_by(|a, b| b.size.cmp(&a.size));

    analysis
        .largest_directories
        .sort_unstable_by(|a, b| b.size.cmp(&a.size));

    analysis.largest_files.truncate(100);
    analysis.largest_directories.truncate(100);

    analysis.extension_statistics = extensions
        .into_iter()
        .map(|(extension, (count, size))| ExtensionStatistic {
            extension,
            count,
            size,
        })
        .collect();

    analysis
}

fn visit(entry: &Entry, analysis: &mut Analysis, extensions: &mut HashMap<String, (u64, u64)>) {
    if entry.is_directory {
        analysis.largest_directories.push(EntrySummary {
            allocated_size: entry.allocated_size,
            path: entry.path.display().to_string(),
            size: entry.size,
        });

        if entry.children.is_empty() {
            analysis.empty_directories.push(EntrySummary {
                allocated_size: entry.allocated_size,
                path: entry.path.display().to_string(),
                size: 0,
            });
        }

        for child in &entry.children {
            visit(child, analysis, extensions);
        }

        return;
    }

    analysis.largest_files.push(EntrySummary {
        allocated_size: entry.allocated_size,
        path: entry.path.display().to_string(),
        size: entry.size,
    });

    if entry.size == 0 {
        analysis.empty_files.push(EntrySummary {
            allocated_size: entry.allocated_size,
            path: entry.path.display().to_string(),
            size: 0,
        });
    }

    let extension = entry
        .path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let stat = extensions.entry(extension).or_insert((0, 0));

    stat.0 += 1;
    stat.1 += entry.size;
}
