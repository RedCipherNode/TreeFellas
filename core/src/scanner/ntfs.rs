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

    pub attributes: Vec<MftAttribute>,
    pub file_names: Vec<FileNameAttribute>,
}

#[derive(Debug)]
pub struct MftAttribute {
    pub attribute_type: u32,
    pub length: u32,
    pub non_resident: bool,
    pub offset: usize,
}

#[derive(Debug)]
pub struct FileNameAttribute {
    pub parent_reference: MftFileReference,

    pub created_time: u64,
    pub modified_time: u64,
    pub changed_time: u64,
    pub accessed_time: u64,

    pub allocated_size: u64,
    pub real_size: u64,

    pub flags: u32,
    pub reparse_value: u32,

    pub name_namespace: u8,
    pub name: String,
}

#[derive(Debug)]
pub struct MftFileReference {
    pub record_number: u64,
    pub sequence_number: u16,
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
        let attributes = Self::parse_attributes(record, first_attribute_offset)?;

        let mut file_names = Vec::new();

        for attribute in &attributes {
            if attribute.attribute_type == 0x30 {
                file_names.push(Self::parse_file_name(record, attribute)?);
            }
        }

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
            attributes,
            file_names,
        })
    }

    fn parse_attributes(
        record: &[u8],
        first_attribute_offset: u16,
    ) -> io::Result<Vec<MftAttribute>> {
        let mut attributes = Vec::new();
        let mut offset = first_attribute_offset as usize;

        while offset + 8 <= record.len() {
            let attribute_type = u32::from_le_bytes(record[offset..offset + 4].try_into().unwrap());

            if attribute_type == 0xFFFF_FFFF {
                break;
            }

            let length = u32::from_le_bytes(record[offset + 4..offset + 8].try_into().unwrap());

            if length < 16 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid MFT attribute length",
                ));
            }

            let end = offset.checked_add(length as usize).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "MFT attribute offset overflow")
            })?;

            if end > record.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "MFT attribute exceeds record boundary",
                ));
            }

            let non_resident = record[offset + 8] != 0;

            attributes.push(MftAttribute {
                attribute_type,
                length,
                non_resident,
                offset,
            });

            offset = end;
        }

        Ok(attributes)
    }

    fn parse_file_name(record: &[u8], attribute: &MftAttribute) -> io::Result<FileNameAttribute> {
        if attribute.non_resident {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "$FILE_NAME attribute is unexpectedly non-resident",
            ));
        }

        let attribute_offset = attribute.offset;

        if attribute_offset + 16 > record.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid $FILE_NAME attribute",
            ));
        }

        let value_length = u32::from_le_bytes(
            record[attribute_offset + 16..attribute_offset + 20]
                .try_into()
                .unwrap(),
        ) as usize;

        let value_offset =
            u16::from_le_bytes([record[attribute_offset + 20], record[attribute_offset + 21]])
                as usize;

        let value_start = attribute_offset + value_offset;

        let value_end = value_start.checked_add(value_length).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "$FILE_NAME value overflow")
        })?;

        if value_end > record.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "$FILE_NAME value exceeds record",
            ));
        }

        let value = &record[value_start..value_end];

        if value.len() < 66 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "$FILE_NAME value is too small",
            ));
        }

        let parent_reference = u64::from_le_bytes(value[0..8].try_into().unwrap());

        let parent_record_number = parent_reference & 0x0000_FFFF_FFFF_FFFF;

        let parent_sequence_number = (parent_reference >> 48) as u16;

        let parent_reference = MftFileReference {
            record_number: parent_record_number,
            sequence_number: parent_sequence_number,
        };

        let created_time = u64::from_le_bytes(value[8..16].try_into().unwrap());

        let modified_time = u64::from_le_bytes(value[16..24].try_into().unwrap());

        let changed_time = u64::from_le_bytes(value[24..32].try_into().unwrap());

        let accessed_time = u64::from_le_bytes(value[32..40].try_into().unwrap());

        let allocated_size = u64::from_le_bytes(value[40..48].try_into().unwrap());

        let real_size = u64::from_le_bytes(value[48..56].try_into().unwrap());

        let flags = u32::from_le_bytes(value[56..60].try_into().unwrap());

        let reparse_value = u32::from_le_bytes(value[60..64].try_into().unwrap());

        let name_length = value[64] as usize;
        let name_namespace = value[65];

        let name_bytes = name_length.checked_mul(2).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "$FILE_NAME length overflow")
        })?;

        let name_end = 66 + name_bytes;

        if name_end > value.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "$FILE_NAME exceeds attribute value",
            ));
        }

        let name = String::from_utf16_lossy(
            &value[66..name_end]
                .chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect::<Vec<_>>(),
        );

        Ok(FileNameAttribute {
            parent_reference,
            created_time,
            modified_time,
            changed_time,
            accessed_time,
            allocated_size,
            real_size,
            flags,
            reparse_value,
            name_namespace,
            name,
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
