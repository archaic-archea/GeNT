use core::num::NonZeroUsize;

use alloc::{vec::Vec, sync::Arc, collections::BTreeMap};
use crossbeam_queue::SegQueue;
use spin::{Mutex, MutexGuard};

pub fn init() {
    WINDOW_ID.add(1, usize::MAX).unwrap();
}

#[derive(Debug)]
/// Last boolean is always the pointer to where to store if the request has completed
pub enum Request {
    /// Remove window with given ID, returns true if success
    /// Params: @window_id, @success_ptr, @status_ptr
    RemoveWindow(NonZeroUsize, *mut bool, *mut bool),
    /// Add window with given width and height, returns non-null ID, and pointer to the buffer
    /// Params: @width, @height, @window_id, @buffer_ptr_ptr, @status_ptr
    AddWindow(usize, usize, *mut usize, *mut *mut u32, *mut bool),
    /// Add framebuffer with given base address, width, height, and stride
    /// Params: @address, @width, @height, @stride
    AddFrameBuffer(*mut u32, usize, usize, usize),
    /// Put the specified window into the front
    /// Params: @window_id
    Focus(NonZeroUsize),
    /// Move a framebuffer to a different coordinate
    /// Params: @window_id, @x, @y
    Move(NonZeroUsize, usize, usize),
}

unsafe impl Send for Request {}
unsafe impl Sync for Request {}

pub static DISPLAY_QUEUE: SegQueue<Request> = SegQueue::new();
static FRAME_BUFFERS: Mutex<Vec<Arc<FrameBuffer>>> = Mutex::new(Vec::new());
static WINDOWS: Mutex<BTreeMap<NonZeroUsize, Arc<Mutex<Window>>>> = Mutex::new(BTreeMap::new());
static WINDOW_ID: vmem::Vmem = vmem::Vmem::new(
    alloc::borrow::Cow::Borrowed("WIN_ID"), 
    1, 
    None
);

struct FrameBuffer {
    addr: Mutex<*mut u32>,
    width: usize,
    height: usize,
    stride: usize,
    buf: Mutex<*mut u32>,
    child: Mutex<Option<Arc<Mutex<Window>>>>,
    last: Mutex<Option<Arc<Mutex<Window>>>>,
}

unsafe impl Send for FrameBuffer {}
unsafe impl Sync for FrameBuffer {}

struct Window {
    parent: Mutex<Option<Arc<Mutex<Window>>>>,
    child: Mutex<Option<Arc<Mutex<Window>>>>,
    fb: Arc<FrameBuffer>,
    x: isize,
    y: isize,
    buf: *mut u32,
    width: usize,
    height: usize,
    id: usize,
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

pub fn display_thread() -> ! {
    let mut frame_buffers = Vec::new();
    loop {
        if let Some(display_req) = DISPLAY_QUEUE.pop() {
            match display_req {
                Request::AddFrameBuffer(
                    addr, 
                    width, 
                    height, 
                    stride
                ) => {
                    for y in 0..height {
                        let base = (stride / 4) * y;
                        for x in 0..width {
                            let idx = base + x;
                            unsafe {addr.add(idx).write_volatile(0xffff69b4);}
                        }
                    }

                    let buffer = crate::mem::PHYS.alloc(height * stride, vmem::AllocStrategy::NextFit).unwrap() + crate::mem::HHDM_OFFSET.load(core::sync::atomic::Ordering::Relaxed);

                    let frame_buffer = Arc::new(FrameBuffer {
                        addr: Mutex::new(addr),
                        width,
                        height,
                        stride: stride / 4,
                        buf: Mutex::new(buffer as *mut u32),
                        child: Mutex::new(None),
                        last: Mutex::new(None),
                    });

                    frame_buffers.push(frame_buffer.clone());
                    FRAME_BUFFERS.lock().push(frame_buffer.clone());
                },
                Request::AddWindow(width, height, id_ptr, buf_ptr, finished_ptr) => {
                    let buf = (
                        crate::mem::PHYS.alloc(height * width * 4, vmem::AllocStrategy::NextFit).unwrap() + crate::mem::HHDM_OFFSET.load(core::sync::atomic::Ordering::Relaxed)
                    ) as *mut u32;

                    let fb = frame_buffers[0].clone();
                    let id = WINDOW_ID.alloc(1, vmem::AllocStrategy::NextFit).unwrap();
                    let window = Arc::new(Mutex::new(Window { 
                        parent: Mutex::new(None), 
                        child: Mutex::new(fb.child.lock().clone()), 
                        fb: fb.clone(),
                        x: 0, 
                        y: 0, 
                        buf, 
                        width, 
                        height,
                        id
                    }));

                    *fb.child.lock() = Some(window.clone());

                    if fb.child.lock().is_none() {
                        *fb.child.lock() = Some(window.clone());
                    }
                    *fb.last.lock() = Some(window.clone());


                    WINDOWS.lock().insert(NonZeroUsize::new(id).unwrap(), window);

                    unsafe {
                        id_ptr.write_volatile(id);
                        buf_ptr.write_volatile(buf);
                        finished_ptr.write_volatile(true);
                    }
                }
                Request::RemoveWindow(id, success, finished) => {
                    let status = true;

                    let window = WINDOWS.lock().remove(&id).unwrap();
                    let window = window.lock();

                    unsafe {WINDOW_ID.free(window.id, 1)};

                    // Set parent's or fb's child to point to current child
                    if let Some(parent) = &*window.parent.lock() {
                        *parent.lock().child.lock() = window.child.lock().clone();
                    } else {
                        *window.fb.child.lock() = window.child.lock().clone();
                    }
                    
                    // Set child's parent to current parent
                    if let Some(child) = &*window.child.lock() {
                        *child.lock().child.lock() = window.parent.lock().clone();
                    };

                    let buf = window.buf as usize - crate::mem::HHDM_OFFSET.load(core::sync::atomic::Ordering::Relaxed);
                    let size = window.width * window.height * 4;

                    unsafe {crate::mem::PHYS.free(buf, size)};

                    unsafe {
                        success.write_volatile(status);
                        finished.write_volatile(true);
                    }
                }
                Request::Focus(id) => {
                    let window = WINDOWS.lock();
                    let window = window.get(&id).unwrap();
                    let win_arc = window.clone();
                    let window = window.lock();

                    // Set parent's child to point to current child
                    if let Some(parent) = &*window.parent.lock() {
                        *parent.lock().child.lock() = window.child.lock().clone();
                    }
                    
                    // Set child's parent to current parent
                    if let Some(child) = &*window.child.lock() {
                        *child.lock().child.lock() = window.parent.lock().clone();
                    };

                    // Set current's child to fb's child
                    *window.child.lock() = window.fb.child.lock().clone();
                    // Set fb's child to current
                    *window.fb.child.lock() = Some(win_arc.clone());
                }
                Request::Move(id, x, y) => {
                    let window = WINDOWS.lock();
                    let window = window.get(&id).unwrap();
                    window.lock().x = x as isize;
                    window.lock().y = y as isize;
                }
                val => todo!("Handle {:?} request", val)
            }
        }

        for fb in frame_buffers.iter() {
            // Render one frame buffer, then switch
            if let Some(child) = fb.clone().child.try_lock() {
                for i in 0..fb.height * fb.stride {
                    unsafe {fb.buf.lock().add(i).write_volatile(0x00)};
                }

                if let Some(child) = child.as_ref() {
                    render_child(fb.clone(), child.lock());
                }

                let dst = *fb.addr.lock();
                let src = *fb.buf.lock();

                unsafe {core::ptr::copy_nonoverlapping(src, dst, fb.height * fb.stride)};
                
                break;
            }
        }
    }
}

fn render_child(fb: Arc<FrameBuffer>, child: MutexGuard<Window>) {
    if let Some(child) = child.child.lock().as_ref() {
        render_child(fb.clone(), child.lock());
    } 

    let x_lim = child.x + child.width as isize;
    let y_lim = child.y + child.height as isize;
    let x_lim = core::cmp::min(x_lim, fb.width as isize);
    let y_lim = core::cmp::min(y_lim, fb.height as isize);

    let x_base = core::cmp::max(child.x, 0);
    let y_base = core::cmp::max(child.y, 0);
    let x_lim = core::cmp::max(x_lim, 0);
    let y_lim = core::cmp::max(y_lim, 0);

    let x_size = x_lim - x_base;

    for (rel_y, abs_y) in (y_base..y_lim).enumerate() {
        unsafe {
            let dest = fb.buf.lock().offset(abs_y * fb.stride as isize).offset(x_base);
            let src = child.buf.add(rel_y * child.width);

            core::ptr::copy_nonoverlapping(src, dest, x_size as usize)
        }
    }
}