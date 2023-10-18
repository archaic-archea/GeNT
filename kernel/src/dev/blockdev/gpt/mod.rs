mod structs;

pub fn init_disk_gpt(disk: &mut dyn super::Disk) {
    let mut block = [0; 512];

    disk.read(&mut block, 0);

    // Set MBR
    if !((block[510] == 0x55) && (block[511] == 0xAA)) {
        for byte in block[0..512].iter_mut() {
            *byte = 0;
        }

        let log_blocks = disk.blocks();

        let log_end_blk = if log_blocks >= 0xffffffff {
            [0xff_u8; 4]
        } else {
            let le_bytes = log_blocks.to_le_bytes();

            [
                le_bytes[0],
                le_bytes[1],
                le_bytes[2],
                le_bytes[3],
            ]
        };

        let chs_blocks = disk.chs_end();

        let chs_end_blk = if chs_blocks == 0xffffffff {
            [0xff_u8; 3]
        } else {
            let le_bytes = chs_blocks.to_le_bytes();

            [
                le_bytes[0],
                le_bytes[1],
                le_bytes[2],
            ]
        };

        // Boot indicator is already 0

        // Start CHS
        //block[447] = 0x0;
        block[448] = 0x02;
        //block[449] = 0x0;

        // OS type
        block[450] = 0xEE;

        // End CHS
        block[451] = chs_end_blk[0];
        block[452] = chs_end_blk[1];
        block[453] = chs_end_blk[2];

        // Start LBA
        block[454] = 0x01;
        //block[455] = 0x00;
        //block[456] = 0x00;
        //block[457] = 0x00;

        // End LBA
        block[458] = log_end_blk[0];
        block[459] = log_end_blk[1];
        block[460] = log_end_blk[2];
        block[461] = log_end_blk[3];

        // MBR Magic number
        block[510] = 0x55;
        block[511] = 0xAA;

        disk.write(&block, 0);
    }

    let mut part = structs::GPTPart::default();
    part.start_lba = 4;
    part.end_lba = (disk.blocks() - 1) as u64;
    part.type_guid = structs::GPTPartType::SwapPart;
    part.unique_guid = 0xc31345c1bbe82ca639519f54bed57569;
    part.set_name("GENT SWAP PART");

    let mut header = structs::GPTHeader::default();
    header.host_lba = 1;
    header.alt_lba = 2;
    header.first_lba = 4;
    header.last_lba = (disk.blocks() - 1) as u64;
    header.disk_guid = 0x052428fae046c21b4c7fcff4d186f5d4;
    header.part_lba = 3;
    header.part_num = 1;
    header.part_checksum = part.crc32();
    header.crc32 = header.crc32();

    gpt_header(
        disk, 
        header
    );

    gpt_part(
        disk, 
        part,
        3
    );
}

#[inline(always)]
fn gpt_part(
    disk: &mut dyn super::Disk,
    part: structs::GPTPart,
    lba: usize
) {
    let array: [u8; 128] = unsafe {
        core::mem::transmute(part)
    };
    disk.write(&array, lba)
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn gpt_header(
    disk: &mut dyn super::Disk, 
    header: structs::GPTHeader,
) {
    let array: [u8; 112] = unsafe {
        core::mem::transmute(header)
    };
    disk.write(&array, header.host_lba as usize);
}



fn crc32_compute_table() -> [u32; 256] {
    let mut crc32_table = [0; 256];

    for n in 0..256 {
        crc32_table[n as usize] = (0..8).fold(n as u32, |acc, _| {
            match acc & 1 {
                1 => 0x04c11db7 ^ (acc >> 1),
                _ => acc >> 1,
            }
        });
    }

    crc32_table
}

fn crc32(buf: &[u8]) -> u32 {
    let crc_table = crc32_compute_table();

    !buf.iter().fold(!0, |acc, octet| {
        (acc >> 8) ^ crc_table[((acc & 0xff) ^ *octet as u32) as usize]
    })
}