use core::mem::transmute;

use crate::mem::PhysicalAddress;

bitfield::bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct VirtualAddress(u64);

    pub page_offset, set_page_offset: 11, 0;
    vpn0, set_vpn0: 20, 12;
    vpn1, set_vpn1: 29, 21;
    vpn2, set_vpn2: 38, 30;
    vpn3, set_vpn3: 47, 39;
    vpn4, set_vpn4: 56, 48;
}

impl VirtualAddress {
    pub fn addr(&self) -> usize {
        self.0 as usize
    }

    pub fn vpn(&self, idx: usize) -> usize {
        let vpn = match idx {
            1 => self.vpn0(),
            2 => self.vpn1(),
            3 => self.vpn2(),
            4 => self.vpn3(),
            5 => self.vpn4(),
            _ => panic!("Indexed too far {}", idx)
        };

        vpn as usize
    }

    pub fn set_vpn(&mut self, idx: usize, val: u64) {
        match idx {
            0 => self.set_vpn0(val),
            1 => self.set_vpn1(val),
            2 => self.set_vpn2(val),
            3 => self.set_vpn3(val),
            4 => self.set_vpn4(val),
            _ => panic!("Indexed too far")
        }
    }

    pub fn to_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub fn to_mut_ptr<T>(&mut self) -> *mut T {
        self.0 as *mut T
    }

    pub fn new(bits: usize) -> Self {
        unsafe {
            transmute(bits)
        }
    }

    pub fn is_kern(&self) -> bool {
        (self.0 >> 63) == 1
    }

    pub fn to_phys(self) -> PhysicalAddress {
        PhysicalAddress::new(self.addr() - crate::mem::HHDM_OFFSET.load(core::sync::atomic::Ordering::Relaxed))
    }
}