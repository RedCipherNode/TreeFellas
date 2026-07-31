use std::fs::File;
use std::io;

pub struct NtfsScanner;

impl NtfsScanner {
    pub fn scan(volume: &str) -> io::Result<()> {
        let _file = File::open(volume)?;

        Ok(())
    }
}

//helper
fn volume_path(drive: &str) -> String {
    format!(r"\\.\{}", drive.trim_end_matches('\\'))
}
