#[repr(C)]
#[derive(Clone, Copy)]
pub struct GPTPart {
    pub type_guid: GPTPartType,
    pub unique_guid: u128,
    pub start_lba: u64,
    pub end_lba: u64,
    pub attributes: u64,
    part_name: [u8; 72],
}

impl GPTPart {
    pub fn set_name(&mut self, name: &str) {
        for (index, character) in name.bytes().enumerate() {
            self.part_name[index] = character;
        }
    }

    pub fn crc32(&self) -> u32 {
        let array: [u8; 128] = unsafe {
            core::mem::transmute(*self)
        };

        super::crc32(&array)
    }
}

impl Default for GPTPart {
    fn default() -> Self {
        Self {
            type_guid: GPTPartType::Unused,
            unique_guid: 0,
            start_lba: 0,
            end_lba: 0,
            attributes: 0,
            part_name: [0; 72]
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GPTHeader {
    /// Signature identifying a GPT Header, should be an ascii string "EFI PART”
    sig: [u8; 8],
    /// Revision, should be `0x00010000`
    rev: u32,
    /// Size of the GPT Header, must be 112
    size: u32,
    /// CRC32 checksum, calculate using `crc32_compute_table`
    pub crc32: u32,
    /// Reserved, must be 0
    _res0: u32,
    /// LBA of the current GPT Header
    pub host_lba: u64,
    /// LBA of the alternate GPT Header
    pub alt_lba: u64,
    /// First partition usable LBA
    pub first_lba: u64,
    /// Last partition usable LBA
    pub last_lba: u64,
    /// Unique ID for the disk
    pub disk_guid: u128,
    /// Starting partition for partition entry array
    pub part_lba: u64,
    /// Number of partitions
    pub part_num: u32,
    /// Size of each partition entry, must be 128
    part_size: u32,
    /// Checksum of partition entry array
    pub part_checksum: u32,
    /// Padding
    _res1: u32,
}

impl GPTHeader {
    pub fn crc32(&self) -> u32 {
        let array: [u8; 112] = unsafe {
            core::mem::transmute(*self)
        };

        super::crc32(&array)
    }
}

impl Default for GPTHeader {
    fn default() -> Self {
        const GPT_SIG: &[u8; 8] = b"EFI PART";

        Self {
            sig: *GPT_SIG,
            rev: 0x00010000,
            size: 112,
            crc32: 0,
            _res0: 0,
            host_lba: 0,
            alt_lba: 0,
            first_lba: 0,
            last_lba: 0,
            disk_guid: 0,
            part_lba: 0,
            part_num: 0,
            part_size: 128,
            part_checksum: 0,
            _res1: 0
        }
    }
}

#[repr(u128)]
#[derive(Clone, Copy)]
pub enum GPTPartType {
    Unused = 0x00,
    EFISysPart = 0xC12A7328F81F11D2BA4B00A0C93EC93B,
    MBRPart = 0x024DEE4133E711D39D690008C781F39F,
    GeneralPart = 0x750f1f42f93f12ee1227e55c10bb3b96,
    SwapPart = 0x33766656ea736a11ddf8b05435cc0685,
}