#![allow(unused)]

use core::arch::asm;

/// SBI call wrapper
#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> (isize, usize) {
    let error: isize;
    let value: usize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
            in("a6") fid,
            in("a7") eid,
        );
    }
    (error, value)
}

/// SBI Extension IDs
const SBI_CONSOLE_PUTCHAR: usize = 0x01;
const SBI_CONSOLE_GETCHAR: usize = 0x02;
const SBI_SHUTDOWN: usize = 0x08;
const SBI_SRST: usize = 0x53525354;

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, 0, c, 0, 0);
}

/// use sbi call to getchar from console (qemu uart handler)
pub fn console_getchar() -> usize {
    let (_, value) = sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0, 0);
    value
}

/// use sbi call to shutdown the kernel
pub fn shutdown(failure: bool) -> ! {
    // Try SRST extension first (SBI v0.3+)
    let reason = if failure { 1usize } else { 0usize };
    sbi_call(SBI_SRST, 0, 0, reason, 0);
    
    // Fallback to legacy shutdown
    sbi_call(SBI_SHUTDOWN, 0, 0, 0, 0);
    
    // Should not reach here
    loop {}
}
