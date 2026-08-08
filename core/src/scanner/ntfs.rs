use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

pub struct NtfsScanner;

#[derive(Debug)]
pub struct NtfsBootSector {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub total_sectors: u64,

    pub mft_cluster: i64,
    pub mft_mirror_cluster: i64,

    pub clusters_per_file_record: i8,
    pub clusters_per_index_buffer: i8,

    pub volume_serial: u64,
}

impl NtfsScanner {
    pub fn scan(drive: &str) -> io::Result<Vec<u8>> {
        let volume = volume_path(drive);

        let mut file = File::open(&volume)?;

        let boot_sector = Self::read_boot_sector(&mut file)?;

        let mft_offset = boot_sector.mft_offset();
        let record_size = boot_sector.file_record_size();

        let record = Self::read_mft_record(&mut file, mft_offset, record_size)?;

        Ok(record)
    }

    fn read_boot_sector(file: &mut File) -> io::Result<NtfsBootSector> {
        let mut sector = [0u8; 512];

        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut sector)?;

        if &sector[3..11] != b"NTFS    " {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Volume is not NTFS",
            ));
        }

        let bytes_per_sector = u16::from_le_bytes([sector[11], sector[12]]);

        let sectors_per_cluster = sector[13];

        let total_sectors = u64::from_le_bytes([
            sector[40], sector[41], sector[42], sector[43], sector[44], sector[45], sector[46],
            sector[47],
        ]);

        let mft_cluster = i64::from_le_bytes([
            sector[48], sector[49], sector[50], sector[51], sector[52], sector[53], sector[54],
            sector[55],
        ]);

        let mft_mirror_cluster = i64::from_le_bytes([
            sector[56], sector[57], sector[58], sector[59], sector[60], sector[61], sector[62],
            sector[63],
        ]);

        let clusters_per_file_record = sector[64] as i8;
        let clusters_per_index_buffer = sector[68] as i8;

        let volume_serial = u64::from_le_bytes([
            sector[72], sector[73], sector[74], sector[75], sector[76], sector[77], sector[78],
            sector[79],
        ]);

        Ok(NtfsBootSector {
            bytes_per_sector,
            sectors_per_cluster,
            total_sectors,
            mft_cluster,
            mft_mirror_cluster,
            clusters_per_file_record,
            clusters_per_index_buffer,
            volume_serial,
        })
    }

    fn read_mft_record(file: &mut File, offset: u64, size: u64) -> io::Result<Vec<u8>> {
        file.seek(SeekFrom::Start(offset))?;

        let mut record = vec![0u8; size as usize];

        file.read_exact(&mut record)?;

        if record.len() < 4 || &record[0..4] != b"FILE" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid MFT record signature",
            ));
        }

        Ok(record)
    }
}

fn volume_path(drive: &str) -> String {
    format!(r"\\.\{}", drive.trim_end_matches('\\'))
}

impl NtfsBootSector {
    pub fn cluster_size(&self) -> u64 {
        self.bytes_per_sector as u64 * self.sectors_per_cluster as u64
    }

    pub fn mft_offset(&self) -> u64 {
        self.mft_cluster as u64 * self.cluster_size()
    }

    pub fn file_record_size(&self) -> u64 {
        if self.clusters_per_file_record > 0 {
            self.clusters_per_file_record as u64 * self.cluster_size()
        } else {
            1u64 << (-self.clusters_per_file_record as i32)
        }
    }
}
