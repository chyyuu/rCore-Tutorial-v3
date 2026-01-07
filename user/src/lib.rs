#![no_std]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    // SAFETY: main is provided by each binary and must be correctly implemented
    exit(unsafe { main() });
}

// External main function - each binary must define its own main
unsafe extern "C" {
    fn main() -> i32;
}

use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
    panic!("unreachable after sys_exit!");
}
pub fn yield_() -> isize {
    sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}
