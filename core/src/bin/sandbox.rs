use std::time::Instant;

use treefellas_core::analysis::analyze;
use treefellas_core::Scanner;

fn main() -> std::io::Result<()> {
    // Ganti sesuai kebutuhan
    let path = r"D:\";

    println!("Scanning: {}", path);

    let start = Instant::now();

    let tree = Scanner::scan(path)?;

    println!("Scan finished in {:?}", start.elapsed());

    let analysis = analyze(&tree);

    println!();
    println!("===== Analysis =====");
    println!("Size        : {}", analysis.total_size);
    println!("Files       : {}", analysis.total_files);
    println!("Directories : {}", analysis.total_directories);

    Ok(())
}