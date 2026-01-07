//! SBI call wrappers
#![allow(unused)]

use core::arch::asm;

// Legacy SBI call numbers (for compatibility)
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;

// SBI Extension IDs
const SBI_EXT_TIMER: usize = 0x54494D45;
const SBI_EXT_SRST: usize = 0x53525354;

/// General SBI call with extension ID and function ID
#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x16") fid,
            in("x17") eid,
        );
    }
    ret
}

/// Set timer using the SBI Timer Extension
pub fn set_timer(timer: u64) {
    #[cfg(target_pointer_width = "64")]
    sbi_call(SBI_EXT_TIMER, 0, timer as usize, 0, 0);
    #[cfg(target_pointer_width = "32")]
    sbi_call(SBI_EXT_TIMER, 0, timer as usize, (timer >> 32) as usize, 0);
}

/// Use SBI call to put a character to console
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, 0, c, 0, 0);
}

/// Use SBI call to get a character from console
pub fn console_getchar() -> usize {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0, 0)
}

/// Use SBI call to shutdown the system
pub fn shutdown() -> ! {
    sbi_call(SBI_EXT_SRST, 0, 0, 0, 0);
    panic!("It should shutdown!");
}
