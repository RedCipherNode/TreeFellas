use std::time::Instant;

use treefellas_core::StdFsScanner;
use treefellas_core::analysis::analyze;
use treefellas_core::formatter::format_size;
use treefellas_core::types::Entry;

fn main() -> std::io::Result<()> {
    let path = r"C:\";

    println!("Scanning: {}", path);

    let start = Instant::now();

    let tree = StdFsScanner::scan(path)?;

    let elapsed = start.elapsed();

    let analysis = analyze(&tree);

    println!();
    println!("===== Filesystem Tree =====");

    print_tree(&tree, 0);

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
