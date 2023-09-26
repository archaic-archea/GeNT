use libsa::endian::LittleEndianU32;
use crate::utils::{Volatile, Read, Write, ReadWrite};

#[repr(C)]
pub struct VirtIoHeader {
    /// MUST return 0x74726976
    pub magic: Volatile<LittleEndianU32, Read>,
    /// MUST return 0x2
    pub version: Volatile<LittleEndianU32, Read>,
    pub device_id: Volatile<LittleEndianU32, Read>,
    pub vendor_id: Volatile<LittleEndianU32, Read>,

    pub device_feat: Volatile<LittleEndianU32, Read>,
    pub device_feat_sel: Volatile<LittleEndianU32, Write>,
    _res0: [u8; 8],

    pub driver_feat: Volatile<u32, Write>,
    pub driver_feat_sel: Volatile<LittleEndianU32, Write>,
    _res1: [u8; 8],

    pub queue_sel: Volatile<LittleEndianU32, Write>,
    pub queue_size_max: Volatile<LittleEndianU32, Read>,
    pub queue_size: Volatile<LittleEndianU32, Write>,
    _res2: [u8; 8],

    pub queue_ready: Volatile<LittleEndianU32, ReadWrite>,
    _res3: [u8; 8],

    pub queue_notify: Volatile<LittleEndianU32, Write>,
    _res4: [u8; 12],

    pub int_stat: Volatile<LittleEndianU32, Read>,
    pub int_ack: Volatile<LittleEndianU32, Write>,
    _res5: [u8; 8],

    pub status: Volatile<LittleEndianU32, ReadWrite>,
    _res6: [u8; 12],

    pub queue_desc_lo: Volatile<LittleEndianU32, Write>,
    pub queue_desc_hi: Volatile<LittleEndianU32, Write>,
    _res7: [u8; 8],

    pub queue_avail_lo: Volatile<LittleEndianU32, Write>,
    pub queue_avail_hi: Volatile<LittleEndianU32, Write>,
    _res8: [u8; 8],

    pub queue_used_lo: Volatile<LittleEndianU32, Write>,
    pub queue_used_hi: Volatile<LittleEndianU32, Write>,
    _res9: [u8; 4],

    pub shared_mem_sel: Volatile<LittleEndianU32, Write>,
    
    pub shared_mem_len_lo: Volatile<LittleEndianU32, Read>,
    pub shared_mem_len_hi: Volatile<LittleEndianU32, Read>,
    
    pub shared_mem_base_lo: Volatile<LittleEndianU32, Read>,
    pub shared_mem_base_hi: Volatile<LittleEndianU32, Read>,

    pub queue_reset: Volatile<LittleEndianU32, ReadWrite>,
    _res10: [u8; 56],

    pub config_generation: Volatile<u32, Read>,
}

impl VirtIoHeader {
    /// # Safety
    /// Must be valid pointer
    pub unsafe fn from_mut_ptr<T>(ptr: *mut T) -> Option<&'static mut VirtIoHeader> {
        let ptr = ptr as *mut VirtIoHeader;
        if !(*ptr).is_valid() {
            None
        } else {
            Some(&mut *ptr)
        }
    }

    pub fn is_valid(&self) -> bool {
        let mut bool = self.magic.read().get() == 0x74726976;
        bool |= self.version.read().get() == 2;

        bool
    }

    pub fn dev_id(&self) -> DeviceType {
        DeviceType::from(self.device_id.read().get() as usize)
    }
}

#[derive(Debug)]
pub enum DeviceType {
    Reserved = 0,
    NetworkCard = 1,
    BlockDev = 2,
    Console = 3,
    EntropyDev = 4,
    MemBalloonTrad = 5,
    IoMem = 6,
    RPMsg = 7,
    SCSIHost = 8,
    Transport9P = 9,
    Mac80211Wlan = 10,
    RProcSerial = 11,
    Caif = 12,
    MemBalloon = 13,
    GPUDev = 16,
    ClockDev = 17,
    InputDev = 18,
    SocketDev = 19,
    CryptoDev = 20,
    SigDistMod = 21,
    PStoreDev = 22,
    IOMMUDev = 23,
    MemDev = 24,
    AudioDev = 25,
    FSDev = 26,
    PMemDev = 27,
    RPMBDev = 28,
    Mac80211HWSim = 29,
    VideoEncoder = 30,
    VideoDecoder = 31,
    SCMIDev = 32,
    NitroSecureModule = 33,
    I2cAdapter = 34,
    WatchDog = 35,
    CANDev = 36,
    ParamServer = 38,
    AudioPolDev = 39,
    BluetoothDev = 40,
    GPIODev = 41,
    RDMADev = 42,
}

impl From<usize> for DeviceType {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Reserved,
            1 => Self::NetworkCard,
            2 => Self::BlockDev,
            3 => Self::Console,
            4 => Self::EntropyDev,
            5 => Self::MemBalloonTrad,
            6 => Self::IoMem,
            7 => Self::RPMsg,
            8 => Self::SCSIHost,
            9 => Self::Transport9P,
            10 => Self::Mac80211Wlan,
            11 => Self::RProcSerial,
            12 => Self::Caif,
            13 => Self::MemBalloon,
            16 => Self::GPUDev,
            17 => Self::ClockDev,
            18 => Self::InputDev,
            19 => Self::SocketDev,
            20 => Self::CryptoDev,
            21 => Self::SigDistMod,
            22 => Self::PStoreDev,
            23 => Self::IOMMUDev,
            24 => Self::MemDev,
            25 => Self::AudioDev,
            26 => Self::FSDev,
            27 => Self::PMemDev,
            28 => Self::RPMBDev,
            29 => Self::Mac80211HWSim,
            30 => Self::VideoEncoder,
            31 => Self::VideoDecoder,
            32 => Self::SCMIDev,
            33 => Self::NitroSecureModule,
            34 => Self::I2cAdapter,
            35 => Self::WatchDog,
            36 => Self::CANDev,
            38 => Self::ParamServer,
            39 => Self::AudioPolDev,
            40 => Self::BluetoothDev,
            41 => Self::GPIODev,
            42 => Self::RDMADev,
            num => panic!("Invalid device id {num}")
        }
    }
}