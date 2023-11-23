use crate::mem::PhysicalAddress;

bitfield::bitfield! {
    pub struct Satp(u64);
    impl Debug;

    ppn, set_ppn: 43, 0;
    asid, set_asid: 59, 44;
    mode_raw, set_mode_raw: 63, 60;
}

impl Satp {
    pub unsafe fn load(self) {
        let satp_raw: usize = unsafe {core::mem::transmute(self)};

        unsafe {
            core::arch::asm!(
                "csrw satp, {satp}",
                satp = in(reg) satp_raw,
            );
        }
    }

    pub fn phys(&self) -> crate::mem::PhysicalAddress {
        crate::mem::PhysicalAddress::new((self.ppn() << 12) as usize)
    }

    pub fn set_phys(&mut self, addr: PhysicalAddress) {
        self.set_ppn((addr.addr() >> 12) as u64);
    }

    pub fn mode(&self) -> super::paging::Mode {
        use super::paging::Mode;

        match self.mode_raw() {
            0 => Mode::Bare,
            8 => Mode::Sv39,
            9 => Mode::Sv48,
            10 => Mode::Sv57,
            11 => Mode::Sv64,
            _ => panic!("Unrecognized paging mode")
        }
    }
}

impl Default for Satp {
    fn default() -> Self {
        let satp_raw: usize;
        unsafe {
            core::arch::asm!(
                "csrr {satp}, satp",
                satp = out(reg) satp_raw,
            );
        }
    
        let satp: super::csr::Satp = unsafe {core::mem::transmute(satp_raw)};

        satp
    }
}

pub struct Stval(usize);

impl Stval {
    pub fn new() -> Self {
        unsafe {
            let val: usize;

            core::arch::asm!("csrr {val}, stval", val = out(reg) val);

            Self(val)
        }
    }

    pub fn addr(&self) -> crate::mem::VirtualAddress {
        crate::mem::VirtualAddress::new(self.0)
    }
}

bitfield::bitfield! {
    pub struct Sstatus(u64);
    impl Debug;

    pub sie, set_sie: 1;
    spie, set_spie: 5;
    ube, set_ube: 6;
    pub spp, set_spp: 8;
    vs, set_vs: 10, 9;
    fs, set_fs: 14, 13;
    xs, set_xs: 16, 15;
    sum, set_sum: 18;
    mxr, set_mxr: 19;
    uxl, set_uxl: 33, 32;
    sd, set_sd: 63;
}

impl Sstatus {
    pub unsafe fn load(self) {
        let sstatus_raw: usize = unsafe {core::mem::transmute(self)};

        unsafe {
            core::arch::asm!(
                "csrw sstatus, {sstatus}",
                sstatus = in(reg) sstatus_raw,
            );
        }
    }
}

impl Default for Sstatus {
    fn default() -> Self {
        let sstatus_raw: usize;
        unsafe {
            core::arch::asm!(
                "csrr {sstatus}, sstatus",
                sstatus = out(reg) sstatus_raw,
            );
        }
    
        let sstatus: super::csr::Sstatus = unsafe {core::mem::transmute(sstatus_raw)};

        sstatus
    }
}

bitfield::bitfield! {
    pub struct Sie(u16);
    impl Debug;

    pub ssie, set_ssie: 1;
    pub stie, set_stie: 5;
    pub seie, set_seie: 9;
}

impl Sie {
    pub unsafe fn load(self) {
        let sie_raw: u16 = unsafe {core::mem::transmute(self)};

        unsafe {
            core::arch::asm!(
                "csrw sie, {sie}",
                sie = in(reg) sie_raw,
            );
        }
    }
}

impl Default for Sie {
    fn default() -> Self {
        let sie_raw: u16;
        unsafe {
            core::arch::asm!(
                "csrr {sie}, sie",
                sie = out(reg) sie_raw,
            );
        }
    
        let sie: super::csr::Sie = unsafe {core::mem::transmute(sie_raw)};

        sie
    }
}