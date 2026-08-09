use std::time::Instant;

use treefellas_core::StdFsScanner;
use treefellas_core::analysis::analyze;
use treefellas_core::formatter::format_size;
use treefellas_core::types::Entry;

fn main() -> std::io::Result<()> {
    let path = r"D:";

    println!("Scanning: {}", path);

    let start = Instant::now();

    let tree = StdFsScanner::scan(path)?;

    println!("Scan finished in {:?}", start.elapsed());

    let analysis = analyze(&tree);

    println!();
    println!("===== Analysis =====");
    println!("Size        : {}", format_size(analysis.total_size));
    println!("Files       : {}", analysis.total_files);
    println!("Directories : {}", analysis.total_directories);

    println!();
    println!("===== Filesystem Tree =====");

    print_tree(&tree, 0, 2);

    Ok(())
}

fn print_tree(entry: &Entry, depth: usize, max_depth: usize) {
    let indent = "  ".repeat(depth);

    println!(
        "{}{} | {} | {}",
        indent,
        entry.name,
        if entry.is_directory {
            "Directory"
        } else {
            "File"
        },
        format_size(entry.size),
    );

    if depth >= max_depth {
        return;
    }

    for child in &entry.children {
        print_tree(child, depth + 1, max_depth);
    }
}
