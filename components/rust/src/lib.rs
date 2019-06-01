#![no_std]
#![feature(lang_items, core_intrinsics)]
use core::intrinsics;
use core::panic::PanicInfo;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt;
use core::fmt::Write;

extern {
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    fn write(file: isize, buffer: *const u8, count: usize) -> usize;
}

struct LibcAllocator;
unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > 8 {
            panic!("Unsupported alignment")
        }
        malloc(layout.size())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr)
    }
}

struct Stdout;
impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            let buffer = s.as_bytes();
            if write(0, buffer.as_ptr(), buffer.len()) > 0 {
                Ok(())
            } else {
                Err(()).map_err(|_| fmt::Error)
            }
        }
    }
}

fn stdout() -> Stdout {
    Stdout
}

fn print_str(s: &str) -> Result<(), ()> {
    stdout().write_str(s).map_err(drop)
}
fn print_fmt(args: fmt::Arguments) -> Result<(), ()> {
    stdout().write_fmt(args).map_err(drop)
}


#[global_allocator]
static A: LibcAllocator = LibcAllocator;


#[no_mangle]
pub extern fn rust_main() {
    print_str("Hello from Rust!\n");
    print_fmt(format_args!("Hello formatted from {}\n", "Rust"));
}

#[lang = "panic_impl"]
#[no_mangle]
pub extern fn rust_begin_panic(_info: &PanicInfo) -> ! {
    unsafe { intrinsics::abort() }
}
