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

#[derive(Debug)]
pub struct MftRecord {
    pub signature: [u8; 4],
    pub update_sequence_offset: u16,
    pub update_sequence_size: u16,

    pub log_file_sequence_number: u64,

    pub sequence_number: u16,
    pub hard_link_count: u16,

    pub first_attribute_offset: u16,
    pub flags: u16,

    pub used_size: u32,
    pub allocated_size: u32,

    pub base_record_reference: u64,
    pub next_attribute_id: u16,

    pub record_number: u32,
}
impl NtfsScanner {
    pub fn scan(drive: &str) -> io::Result<MftRecord> {
        let volume = volume_path(drive);

        let mut file = File::open(&volume)?;

        let boot_sector = Self::read_boot_sector(&mut file)?;

        let mft_offset = boot_sector.mft_offset();

        let record_size = boot_sector.file_record_size();

        Self::read_mft_record(
            &mut file,
            mft_offset,
            record_size,
            boot_sector.bytes_per_sector as usize,
        )
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

    fn read_mft_record(
        file: &mut File,
        offset: u64,
        size: u64,
        bytes_per_sector: usize,
    ) -> io::Result<MftRecord> {
        file.seek(SeekFrom::Start(offset))?;

        let mut record = vec![0u8; size as usize];

        file.read_exact(&mut record)?;

        if record.len() < 4 || &record[0..4] != b"FILE" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid MFT record signature",
            ));
        }

        Self::apply_fixup(&mut record, bytes_per_sector)?;

        Self::parse_mft_record(&record)
    }

    fn apply_fixup(record: &mut [u8], bytes_per_sector: usize) -> io::Result<()> {
        if record.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MFT record is too small",
            ));
        }

        let usa_offset = u16::from_le_bytes([record[4], record[5]]) as usize;

        let usa_count = u16::from_le_bytes([record[6], record[7]]) as usize;

        if usa_offset + (usa_count * 2) > record.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Update Sequence Array",
            ));
        }

        if usa_count < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Update Sequence Array count",
            ));
        }

        let update_sequence = u16::from_le_bytes([record[usa_offset], record[usa_offset + 1]]);

        for i in 1..usa_count {
            let sector_end = i * bytes_per_sector;

            if sector_end < 2 || sector_end > record.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid MFT sector boundary",
                ));
            }

            let offset = sector_end - 2;

            let existing = u16::from_le_bytes([record[offset], record[offset + 1]]);

            if existing != update_sequence {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "MFT Update Sequence mismatch",
                ));
            }

            let replacement_offset = usa_offset + (i * 2);

            let replacement = [record[replacement_offset], record[replacement_offset + 1]];

            record[offset..offset + 2].copy_from_slice(&replacement);
        }

        Ok(())
    }

    fn parse_mft_record(record: &[u8]) -> io::Result<MftRecord> {
        if record.len() < 48 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MFT record header is too small",
            ));
        }

        if &record[0..4] != b"FILE" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid MFT record signature",
            ));
        }

        let signature = [record[0], record[1], record[2], record[3]];

        let update_sequence_offset = u16::from_le_bytes([record[4], record[5]]);

        let update_sequence_size = u16::from_le_bytes([record[6], record[7]]);

        let log_file_sequence_number = u64::from_le_bytes(record[8..16].try_into().unwrap());

        let sequence_number = u16::from_le_bytes([record[16], record[17]]);

        let hard_link_count = u16::from_le_bytes([record[18], record[19]]);

        let first_attribute_offset = u16::from_le_bytes([record[20], record[21]]);

        let flags = u16::from_le_bytes([record[22], record[23]]);

        let used_size = u32::from_le_bytes(record[24..28].try_into().unwrap());

        let allocated_size = u32::from_le_bytes(record[28..32].try_into().unwrap());

        let base_record_reference = u64::from_le_bytes(record[32..40].try_into().unwrap());

        let next_attribute_id = u16::from_le_bytes([record[40], record[41]]);

        let record_number = u32::from_le_bytes(record[44..48].try_into().unwrap());

        Ok(MftRecord {
            signature,
            update_sequence_offset,
            update_sequence_size,
            log_file_sequence_number,
            sequence_number,
            hard_link_count,
            first_attribute_offset,
            flags,
            used_size,
            allocated_size,
            base_record_reference,
            next_attribute_id,
            record_number,
        })
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
