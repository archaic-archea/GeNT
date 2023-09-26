extern {
    pub static __global_pointer: LinkerSymbol;
    pub static __tdata_start: LinkerSymbol;
    pub static __tdata_end: LinkerSymbol;
}

#[repr(transparent)]
pub struct LinkerSymbol(u8);

impl LinkerSymbol {
    pub fn as_ptr(&self) -> *const u8 {
        core::ptr::addr_of!(self).cast()
    }

    pub fn as_usize(&self) -> usize {
        self.as_ptr() as usize
    }
}