use core::{
    alloc::{GlobalAlloc, Layout},
    ffi::c_void,
    panic::PanicInfo,
};

struct CppAllocator {}

unsafe impl GlobalAlloc for CppAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (unsafe { cpp_alloc(layout.size(), layout.align()) }) as _
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        cpp_free(ptr as _, layout.align());
    }
}

#[global_allocator]
static ALLOCATOR: CppAllocator = CppAllocator {};

#[cfg_attr(not(test), panic_handler)]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

extern "C" {
    pub fn cpp_alloc(size: usize, al: usize) -> *mut c_void;
}
extern "C" {
    pub fn cpp_free(ptr: *mut c_void, al: usize);
}
