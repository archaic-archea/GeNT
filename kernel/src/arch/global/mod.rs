pub(crate) mod trap;

use core::mem::MaybeUninit;

use libsa::endian::BigEndianU32;

#[derive(Clone, Copy, Debug)]
pub enum IOType {
    Port(usize),
    Mem(usize)
}

impl IOType {
    pub fn loc(&self) -> usize {
        *match self {
            Self::Port(location) => location,
            Self::Mem(location) => location,
        }
    }
}

pub struct IOTransit {
    location: IOType
}

impl IOTransit {
    /// # Safety
    /// The provided address must be valid in memory or port space.
    /// No guarantees are made about how accessing this address will effect things due to IO
    pub unsafe fn new(location: IOType) -> Self {
        Self { location }
    }

    pub fn write<T: PortAccess + Sized>(&self, offset: usize, val: T) {
        match self.iotype() {
            IOType::Port(location) => unsafe {PortAccess::write(location + offset, val);},
            IOType::Mem(_) => self.memwrite(offset, val)
        }
    }

    pub fn read<T: PortAccess + Copy>(&self, offset: usize) -> T {
        match self.iotype() {
            IOType::Port(location) => unsafe {PortAccess::read(location + offset)},
            IOType::Mem(_) => self.memread(offset)
        }
    }

    fn memwrite<T: Sized>(&self, offset: usize, val: T) {
        unsafe {
            let ptr = (self.iotype().loc() + offset) as *mut T;
            ptr.write_volatile(val);
        }
    }

    fn memread<T: Copy>(&self, offset: usize) -> T {
        unsafe {
            let ptr = (self.iotype().loc() + offset) as *const T;

            ptr.read_volatile()
        }
    }

    pub fn iotype(&self) -> IOType {
        self.location
    }

    /// # Safety
    /// The location here must follow the same rules as the address provided to `Self::new`
    pub unsafe fn set_loc(&mut self, loc: usize) {
        match self.location {
            IOType::Mem(_) => self.location = IOType::Mem(loc),
            IOType::Port(_) => self.location = IOType::Port(loc),
        }
    }

    unsafe fn port_read_raw<S: Copy + PortAccess>(&self, offset: usize, dst: *mut S, size: usize) {
        let mut written = 0;
        while written < size {
            dst.add(written).write(PortAccess::read(self.iotype().loc() + offset));
            written += 1;
        }
    }

    unsafe fn mem_read_raw<S: Copy>(&self, offset: usize, dst: *mut S, size: usize) {
        let iters = size / core::mem::size_of::<S>();

        let mut written = 0;
        while written < iters {
            dst.add(written).write(self.memread::<S>(offset));
            written += 1;
        }
    }

    pub fn read_serial<T: Sized, S: Copy + PortAccess>(&self, offset: usize) -> T {
        let mut uninit = MaybeUninit::<T>::uninit();
        unsafe {
            match self.iotype() {
                IOType::Mem(_) => self.mem_read_raw(offset, uninit.as_mut_ptr().cast::<S>(), core::mem::size_of::<T>()),
                IOType::Port(_) => self.port_read_raw(offset, uninit.as_mut_ptr().cast::<S>(), core::mem::size_of::<T>())
            }
            uninit.assume_init()
        }
    }
}

pub trait PortAccess {
    /// # Safety
    /// The address must be a valid address in port space.
    unsafe fn write(location: usize, val: Self);
    
    /// # Safety
    /// The address must be a valid address in port space.
    unsafe fn read(location: usize) -> Self;
}

impl PortAccess for BigEndianU32 {
    unsafe fn read(location: usize) -> Self {
        let base: u32 = PortAccess::read(location);

        BigEndianU32::new(base)
    }

    unsafe fn write(location: usize, val: Self) {
        PortAccess::write(location, val.get())
    }
}
