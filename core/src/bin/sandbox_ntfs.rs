use std::path::Path;
use std::time::Instant;

use treefellas_core::analysis::analyze;
use treefellas_core::formatter::format_size;
use treefellas_core::types::Entry;
use treefellas_core::{NtfsScanner, NtfsTree};

fn main() -> std::io::Result<()> {
    let path = r"C:\";

    println!("Scanning: {}", path);

    let start = Instant::now();

    let mut reader = NtfsScanner::open_mft_reader(path)?;
    let mut tree = NtfsTree::new();

    reader.enumerate_records(|_, record| match record {
        Some(record) => {
            for file_name in &record.file_names {
                let node = NtfsScanner::file_name_to_node(&record, file_name);

                tree.insert(node);
            }
        }

        None => {}
    })?;

    tree.link_relationships();

    let roots = tree.to_entries(Path::new(path));

    let root = match roots.into_iter().next() {
        Some(root) => root,
        None => {
            println!("No NTFS root found.");
            return Ok(());
        }
    };

    let elapsed = start.elapsed();

    let analysis = analyze(&root);

    println!();
    println!("===== Filesystem Tree =====");

    print_tree(&root, 0);

    println!();
    println!("===== Analysis =====");
    println!("Total Files       : {}", analysis.total_files);
    println!("Total Directories : {}", analysis.total_directories);
    println!("Total Logical Size: {}", format_size(analysis.total_size));
    println!("Scan Time         : {:?}", elapsed);

    Ok(())
}

fn print_tree(entry: &Entry, depth: usize) {
    let indent = "  ".repeat(depth);

    if entry.is_directory {
        println!("{}📁 {} [{}]", indent, entry.name, format_size(entry.size));
    } else {
        println!("{}📄 {} ({})", indent, entry.name, format_size(entry.size));
    }

    for child in &entry.children {
        print_tree(child, depth + 1);
    }
}
