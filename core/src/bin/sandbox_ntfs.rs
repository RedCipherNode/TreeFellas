use std::time::Instant;

use treefellas_core::NtfsScanner;

fn main() -> std::io::Result<()> {
    let drive = r"D:";

    println!("Scanning: {}", drive);

    let start = Instant::now();

    let record = NtfsScanner::scan(drive)?;

    println!("Scan finished in {:?}", start.elapsed());

    println!();
    println!("===== MFT Record =====");
    println!("Signature       : {:?}", record.signature);
    println!("Sequence        : {}", record.sequence_number);
    println!("Hard Links      : {}", record.hard_link_count);
    println!("Attribute Offset: {}", record.first_attribute_offset);
    println!("Flags           : 0x{:04X}", record.flags);
    println!("Used Size       : {}", record.used_size);
    println!("Allocated Size  : {}", record.allocated_size);
    println!("Base Reference  : {}", record.base_record_reference);
    println!("Next Attribute  : {}", record.next_attribute_id);
    println!("Record Number   : {}", record.record_number);

    Ok(())
}
