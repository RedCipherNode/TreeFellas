use std::time::Instant;

use treefellas_core::StdFsScanner;
use treefellas_core::analysis::analyze;

fn main() -> std::io::Result<()> {
    // Ganti sesuai kebutuhan
    let path = r"D:\";

    println!("Scanning: {}", path);

    let start = Instant::now();

    let tree = StdFsScanner::scan(path)?;

    println!("Scan finished in {:?}", start.elapsed());

    let analysis = analyze(&tree);

    println!();
    println!("===== Analysis =====");
    println!("Size        : {}", analysis.total_size);
    println!("Files       : {}", analysis.total_files);
    println!("Directories : {}", analysis.total_directories);

    Ok(())
}
