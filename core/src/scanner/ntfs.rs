use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

pub struct NtfsScanner;

#[allow(dead_code)]
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

    pub data: Option<MftDataAttribute>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MftFileReference {
    pub record_number: u64,
    pub sequence_number: u16,
}

#[derive(Debug)]
pub struct MftDataRun {
    pub start_lcn: i64,
    pub cluster_count: u64,
}

#[derive(Debug)]
pub struct MftDataAttribute {
    pub allocated_size: u64,
    pub real_size: u64,
    pub initialized_size: u64,
    pub runs: Vec<MftDataRun>,
}

pub struct MftReader {
    file: File,
    runs: Vec<MftDataRun>,

    bytes_per_sector: usize,
    cluster_size: u64,
    record_size: u64,
    real_size: u64,
}

#[derive(Debug)]
pub struct NtfsNode {
    pub file_reference: MftFileReference,
    pub parent_reference: MftFileReference,

    pub name: String,
    pub size: u64,
    pub is_directory: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

pub struct NtfsTree {
    nodes: Vec<NtfsNode>,
    children: Vec<Vec<NodeId>>,
    by_reference: HashMap<MftFileReference, Vec<NodeId>>,
    roots: Vec<NodeId>,
    unattached: Vec<NodeId>,
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

        Self::parse_mft_record(&record, true)
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

    fn parse_mft_record(record: &[u8], parse_data: bool) -> io::Result<MftRecord> {
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
        let mut data = None;
        for attribute in &attributes {
            match attribute.attribute_type {
                0x30 => {
                    file_names.push(Self::parse_file_name(record, attribute)?);
                }

                0x80 if parse_data => {
                    data = Some(Self::parse_data_attribute(record, attribute)?);
                }

                _ => {}
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
            data,
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

    fn parse_data_attribute(
        record: &[u8],
        attribute: &MftAttribute,
    ) -> io::Result<MftDataAttribute> {
        if !attribute.non_resident {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "$DATA attribute is resident",
            ));
        }

        let offset = attribute.offset;

        if offset + 64 > record.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid non-resident $DATA attribute",
            ));
        }

        let allocated_size =
            u64::from_le_bytes(record[offset + 40..offset + 48].try_into().unwrap());

        let real_size = u64::from_le_bytes(record[offset + 48..offset + 56].try_into().unwrap());

        let initialized_size =
            u64::from_le_bytes(record[offset + 56..offset + 64].try_into().unwrap());

        let mapping_pairs_offset =
            u16::from_le_bytes([record[offset + 32], record[offset + 33]]) as usize;

        let run_start = offset + mapping_pairs_offset;
        let attribute_end = offset + attribute.length as usize;

        if run_start > attribute_end || attribute_end > record.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid mapping pairs offset",
            ));
        }

        let runs = Self::parse_mapping_pairs(&record[run_start..attribute_end])?;

        Ok(MftDataAttribute {
            allocated_size,
            real_size,
            initialized_size,
            runs,
        })
    }

    fn parse_mapping_pairs(data: &[u8]) -> io::Result<Vec<MftDataRun>> {
        let mut runs = Vec::new();
        let mut offset = 0usize;
        let mut current_lcn = 0i64;

        while offset < data.len() {
            let header = data[offset];
            offset += 1;

            if header == 0 {
                break;
            }

            let length_size = (header & 0x0F) as usize;
            let offset_size = ((header >> 4) & 0x0F) as usize;

            if length_size == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid mapping pair length",
                ));
            }

            if offset + length_size + offset_size > data.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Mapping pair exceeds attribute",
                ));
            }

            let mut cluster_count = 0u64;

            for i in 0..length_size {
                cluster_count |= (data[offset + i] as u64) << (i * 8);
            }

            offset += length_size;

            let mut lcn_delta = 0i64;

            if offset_size > 0 {
                let mut value = 0i64;

                for i in 0..offset_size {
                    value |= (data[offset + i] as i64) << (i * 8);
                }

                // Sign extend the relative LCN.
                if data[offset + offset_size - 1] & 0x80 != 0 {
                    value |= !0i64 << (offset_size * 8);
                }

                lcn_delta = value;
            }

            offset += offset_size;

            current_lcn += lcn_delta;

            runs.push(MftDataRun {
                start_lcn: current_lcn,
                cluster_count,
            });
        }

        Ok(runs)
    }

    //  Helpers temporary
    pub fn open_mft_reader(drive: &str) -> io::Result<MftReader> {
        let volume = volume_path(drive);

        let mut file = File::open(&volume)?;

        let boot_sector = Self::read_boot_sector(&mut file)?;

        let cluster_size = boot_sector.cluster_size();

        let mft_offset = boot_sector.mft_offset();

        let record_size = boot_sector.file_record_size();

        file.seek(SeekFrom::Start(mft_offset))?;

        let mut record = vec![0u8; record_size as usize];

        file.read_exact(&mut record)?;

        Self::apply_fixup(&mut record, boot_sector.bytes_per_sector as usize)?;

        let mft_record = Self::parse_mft_record(&record, true)?;

        let data = mft_record.data.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "$MFT has no $DATA attribute")
        })?;

        let reader = MftReader::new(
            file,
            data.runs,
            boot_sector.bytes_per_sector as usize,
            cluster_size,
            record_size,
            data.real_size,
        );

        Ok(reader)
    }

    pub fn file_name_to_node(record: &MftRecord, file_name: &FileNameAttribute) -> NtfsNode {
        let file_reference = MftFileReference {
            record_number: record.record_number as u64,
            sequence_number: record.sequence_number,
        };

        NtfsNode {
            file_reference,
            parent_reference: file_name.parent_reference,
            name: file_name.name.clone(),
            size: file_name.real_size,
            is_directory: file_name.flags & 0x10000000 != 0,
        }
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

impl MftReader {
    fn new(
        file: File,
        runs: Vec<MftDataRun>,
        bytes_per_sector: usize,
        cluster_size: u64,
        record_size: u64,
        real_size: u64,
    ) -> Self {
        Self {
            file,
            runs,
            bytes_per_sector,
            cluster_size,
            record_size,
            real_size,
        }
    }

    fn resolve_offset(&self, logical_offset: u64) -> io::Result<u64> {
        if logical_offset >= self.real_size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "MFT offset exceeds real size",
            ));
        }

        let mut logical_start = 0u64;

        for run in &self.runs {
            let run_size = run.cluster_count * self.cluster_size;

            let logical_end = logical_start + run_size;

            if logical_offset < logical_end {
                let within_run = logical_offset - logical_start;

                if run.start_lcn < 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Negative LCN is not supported",
                    ));
                }

                return Ok(run.start_lcn as u64 * self.cluster_size + within_run);
            }

            logical_start = logical_end;
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unable to resolve MFT logical offset",
        ))
    }

    // Reads one specific MFT record
    // Useful for random access and debugging
    pub fn read_record(&mut self, record_number: u64) -> io::Result<Option<MftRecord>> {
        let logical_offset = record_number.checked_mul(self.record_size).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "MFT record offset overflow")
        })?;

        let physical_offset = self.resolve_offset(logical_offset)?;

        self.file.seek(SeekFrom::Start(physical_offset))?;

        let mut record = vec![0u8; self.record_size as usize];

        self.file.read_exact(&mut record)?;

        if record.len() < 4 || &record[0..4] != b"FILE" {
            return Ok(None);
        }

        NtfsScanner::apply_fixup(&mut record, self.bytes_per_sector)?;

        Ok(Some(NtfsScanner::parse_mft_record(&record, false)?))
    }

    // Reads the MFT in large chunks instead of reading one record at a time
    // This is the main path used for full-volume scanning
    pub fn enumerate_records<F>(&mut self, mut callback: F) -> io::Result<()>
    where
        F: FnMut(u64, Option<MftRecord>),
    {
        const CHUNK_SIZE: u64 = 8 * 1024 * 1024;

        let record_count = self.record_count();
        let mut record_number = 0u64;

        for run in &self.runs {
            if run.start_lcn < 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Negative LCN is not supported",
                ));
            }

            let run_size = run.cluster_count * self.cluster_size;

            let physical_offset = run.start_lcn as u64 * self.cluster_size;

            let mut run_position = 0u64;

            while run_position < run_size && record_number < record_count {
                let remaining = run_size - run_position;

                let mut chunk_size = remaining.min(CHUNK_SIZE);

                chunk_size -= chunk_size % self.record_size;

                if chunk_size == 0 {
                    break;
                }

                self.file
                    .seek(SeekFrom::Start(physical_offset + run_position))?;

                let mut buffer = vec![0u8; chunk_size as usize];

                self.file.read_exact(&mut buffer)?;

                let records_in_chunk = chunk_size / self.record_size;

                for index in 0..records_in_chunk {
                    if record_number >= record_count {
                        break;
                    }

                    let start = (index * self.record_size) as usize;

                    let end = start + self.record_size as usize;

                    let record = &mut buffer[start..end];

                    let parsed = if &record[0..4] == b"FILE" {
                        NtfsScanner::apply_fixup(record, self.bytes_per_sector)?;

                        Some(NtfsScanner::parse_mft_record(record, false)?)
                    } else {
                        None
                    };

                    callback(record_number, parsed);

                    record_number += 1;
                }

                run_position += chunk_size;
            }
        }

        Ok(())
    }

    pub fn record_count(&self) -> u64 {
        self.real_size / self.record_size
    }
}

impl NtfsTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            children: Vec::new(),
            by_reference: HashMap::new(),
            roots: Vec::new(),
            unattached: Vec::new(),
        }
    }

    pub fn insert(&mut self, node: NtfsNode) -> NodeId {
        let id = NodeId(self.nodes.len());

        self.by_reference
            .entry(node.file_reference)
            .or_default()
            .push(id);

        self.nodes.push(node);
        self.children.push(Vec::new());

        id
    }

    pub fn node(&self, id: NodeId) -> Option<&NtfsNode> {
        self.nodes.get(id.0)
    }

    pub fn children(&self, id: NodeId) -> Option<&[NodeId]> {
        self.children.get(id.0).map(Vec::as_slice)
    }

    pub fn nodes(&self) -> &[NtfsNode] {
        &self.nodes
    }

    pub fn find_by_reference(&self, reference: MftFileReference) -> Option<&[NodeId]> {
        self.by_reference.get(&reference).map(Vec::as_slice)
    }

    pub fn roots(&self) -> &[NodeId] {
        &self.roots
    }

    pub fn unattached(&self) -> &[NodeId] {
        &self.unattached
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn link_relationships(&mut self) {
        self.roots.clear();
        self.unattached.clear();

        for children in &mut self.children {
            children.clear();
        }

        for index in 0..self.nodes.len() {
            let node_id = NodeId(index);
            let node = &self.nodes[index];

            if node.file_reference.record_number == 5
                && node.parent_reference.record_number == 5
                && node.is_directory
            {
                self.roots.push(node_id);
                continue;
            }

            let Some(parent_ids) = self.by_reference.get(&node.parent_reference) else {
                self.unattached.push(node_id);
                continue;
            };

            let Some(parent_id) = parent_ids
                .iter()
                .copied()
                .find(|id| self.nodes[id.0].is_directory)
            else {
                self.unattached.push(node_id);
                continue;
            };

            self.children[parent_id.0].push(node_id);
        }
    }

    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    pub fn child_count(&self, id: NodeId) -> usize {
        self.children.get(id.0).map(Vec::len).unwrap_or(0)
    }

    pub fn linked_count(&self) -> usize {
        self.children.iter().map(Vec::len).sum()
    }

    pub fn unattached_count(&self) -> usize {
        self.unattached.len()
    }
}
