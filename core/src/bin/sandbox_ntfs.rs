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

    println!();
    println!("===== Attributes =====");

    for attribute in &record.attributes {
        println!(
            "Type: 0x{:08X} | Length: {} | Non-resident: {}",
            attribute.attribute_type, attribute.length, attribute.non_resident
        );
    }

    println!();
    println!("===== File Names =====");

    for file_name in &record.file_names {
        println!("Name   : {}", file_name.name);
        println!(
            "Parent Record : {}",
            file_name.parent_reference.record_number
        );

        println!(
            "Parent Seq    : {}",
            file_name.parent_reference.sequence_number
        );
        println!("Size   : {}", file_name.real_size);
    }

    if let Some(data) = &record.data {
        println!();
        println!("===== $DATA =====");
        println!("Allocated : {}", data.allocated_size);
        println!("Real Size : {}", data.real_size);
        println!("Initialized: {}", data.initialized_size);

        println!();
        println!("===== Data Runs =====");

        for run in &data.runs {
            println!("LCN: {} | Clusters: {}", run.start_lcn, run.cluster_count);
        }
    }

    let mut reader = NtfsScanner::open_mft_reader(drive)?;

    println!();
    println!("===== MFT Enumeration =====");

    let start = Instant::now();

    let record_count = reader.record_count();

    let mut used = 0u64;
    let mut unused = 0u64;
    let mut files = 0u64;
    let mut directories = 0u64;

    let start = Instant::now();

    reader.enumerate_records(|_, record| match record {
        Some(record) => {
            used += 1;

            if record.flags & 0x0002 != 0 {
                directories += 1;
            } else {
                files += 1;
            }
        }

        None => {
            unused += 1;
        }
    })?;

    println!("Records     : {}", reader.record_count());
    println!("Used        : {}", used);
    println!("Unused      : {}", unused);
    println!("Files       : {}", files);
    println!("Directories : {}", directories);
    println!("Elapsed     : {:?}", start.elapsed());

    Ok(())
}
