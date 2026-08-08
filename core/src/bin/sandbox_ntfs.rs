use std::time::Instant;

use treefellas_core::NtfsScanner;

fn main() -> std::io::Result<()> {
    let drive = r"D:";

    println!("Scanning: {}", drive);

    let start = Instant::now();

    let record = NtfsScanner::scan(drive)?;

    println!("Scan finished in {:?}", start.elapsed());

    println!();
    println!("===== MFT Record #0 =====");
    println!("Size      : {} bytes", record.len());
    println!("Signature : {}", String::from_utf8_lossy(&record[0..4]));

    Ok(())
}
