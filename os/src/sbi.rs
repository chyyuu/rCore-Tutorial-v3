//! SBI call wrappers
//!
//! Supports both RustSBI (external) and nobios (built-in M-Mode SBI) modes

/// SBI call implementation using ecall
#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> (isize, usize) {
    let error;
    let value;
    unsafe {
        core::arch::asm!(
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

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    sbi_call(0x01, 0, c, 0, 0);
}

/// use sbi call to getchar from console (qemu uart handler)
#[allow(unused)]
pub fn console_getchar() -> usize {
    let (_, value) = sbi_call(0x02, 0, 0, 0, 0);
    value
}

/// use sbi call to shutdown the kernel
pub fn shutdown(failure: bool) -> ! {
    // Use SRST extension (0x53525354) for system reset
    const SRST_EID: usize = 0x53525354;
    const SRST_SHUTDOWN: usize = 0;
    let reason = if failure { 1 } else { 0 };
    sbi_call(SRST_EID, SRST_SHUTDOWN, 0, reason, 0);
    
    // Fallback to legacy shutdown
    sbi_call(0x08, 0, 0, 0, 0);
    unreachable!()
}
