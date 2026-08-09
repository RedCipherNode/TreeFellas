use std::time::Instant;

use treefellas_core::{NtfsScanner, NtfsTree};

fn main() -> std::io::Result<()> {
    let drive = r"D:";

    println!("Scanning: {}", drive);

    let start = Instant::now();

    let mut reader = NtfsScanner::open_mft_reader(drive)?;

    let mut records = 0u64;
    let mut unused = 0u64;
    let mut file_name_count = 0u64;
    let mut tree = NtfsTree::new();

    reader.enumerate_records(|_, record| match record {
        Some(record) => {
            records += 1;
            file_name_count += record.file_names.len() as u64;

            for file_name in &record.file_names {
                let node = NtfsScanner::file_name_to_node(&record, file_name);
                tree.insert(node);
            }
        }

        None => {
            unused += 1;
        }
    })?;

    tree.link_relationships();

    println!();
    println!("===== NTFS Scan =====");
    println!("Records      : {}", records);
    println!("Unused       : {}", unused);
    println!("File Names   : {}", file_name_count);
    println!("Nodes        : {}", tree.nodes().len());
    println!("Roots        : {}", tree.root_count());
    println!("Linked       : {}", tree.linked_count());
    println!("Elapsed      : {:?}", start.elapsed());

    Ok(())
}
