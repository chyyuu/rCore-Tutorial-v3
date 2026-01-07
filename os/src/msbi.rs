//! M-Mode SBI implementation for -bios none boot
//! This module provides a minimal SBI implementation that runs in M-Mode

use core::arch::asm;

const UART_BASE: usize = 0x1000_0000;

/// SBI return structure
#[repr(C)]
pub struct SbiRet {
    /// Error code (0 = success)
    pub error: usize,
    /// Return value
    pub value: usize,
}

impl SbiRet {
    /// Create a successful return
    pub fn success(value: usize) -> Self {
        Self { error: 0, value }
    }
    /// Create a not-supported error return
    pub fn not_supported() -> Self {
        Self { error: usize::MAX, value: 0 }
    }
}

/// Legacy SBI call numbers
mod legacy {
    pub const CONSOLE_PUTCHAR: usize = 1;
    pub const CONSOLE_GETCHAR: usize = 2;
    pub const SHUTDOWN: usize = 8;
}

/// SBI Extension IDs
mod eid {
    pub const BASE: usize = 0x10;
    pub const TIMER: usize = 0x54494D45;
    pub const SRST: usize = 0x53525354;
}

/// SBI Function IDs
mod fid {
    // Base extension
    pub const BASE_GET_SBI_VERSION: usize = 0;
    pub const BASE_GET_IMPL_ID: usize = 1;
    pub const BASE_GET_IMPL_VERSION: usize = 2;
    pub const BASE_PROBE_EXTENSION: usize = 3;
    pub const BASE_GET_MVENDORID: usize = 4;
    pub const BASE_GET_MARCHID: usize = 5;
    pub const BASE_GET_MIMPID: usize = 6;

    // System Reset extension
    pub const SRST_SHUTDOWN: usize = 0;
    #[allow(dead_code)]
    pub const SRST_COLD_REBOOT: usize = 1;
    #[allow(dead_code)]
    pub const SRST_WARM_REBOOT: usize = 2;
}

/// Write a character to UART
fn uart_putchar(c: u8) {
    unsafe {
        (UART_BASE as *mut u8).write_volatile(c);
    }
}

/// Read a character from UART (non-blocking)
fn uart_getchar() -> Option<u8> {
    // UART LSR (Line Status Register) is at offset 5
    let lsr = unsafe { ((UART_BASE + 5) as *mut u8).read_volatile() };
    // Check if data is ready (bit 0)
    if lsr & 1 != 0 {
        Some(unsafe { (UART_BASE as *mut u8).read_volatile() })
    } else {
        None
    }
}

/// Handle Base extension calls
fn handle_base(fid: usize, _a0: usize, _a1: usize, _a2: usize) -> SbiRet {
    match fid {
        fid::BASE_GET_SBI_VERSION => SbiRet::success(0x01000000), // SBI v1.0.0
        fid::BASE_GET_IMPL_ID => SbiRet::success(1),
        fid::BASE_GET_IMPL_VERSION => SbiRet::success(1),
        fid::BASE_PROBE_EXTENSION => SbiRet::success(1),
        fid::BASE_GET_MVENDORID => SbiRet::success(0),
        fid::BASE_GET_MARCHID => SbiRet::success(0),
        fid::BASE_GET_MIMPID => SbiRet::success(0),
        _ => SbiRet::not_supported(),
    }
}

/// Handle Timer extension calls
fn handle_timer(time: u64) -> SbiRet {
    const CLINT_MTIMECMP: usize = 0x200_4000;
    unsafe {
        #[cfg(target_pointer_width = "64")]
        (CLINT_MTIMECMP as *mut u64).write_volatile(time);
        #[cfg(target_pointer_width = "32")]
        {
            (CLINT_MTIMECMP as *mut u32).write_volatile(time as u32);
            ((CLINT_MTIMECMP + 4) as *mut u32).write_volatile((time >> 32) as u32);
        }
    }
    // Clear supervisor timer interrupt pending bit
    unsafe {
        asm!(
            "csrc mip, {}",
            in(reg) (1 << 5),
        );
    }
    SbiRet::success(0)
}

/// Handle System Reset extension calls
fn handle_srst(fid: usize, _a0: usize, _a1: usize, _a2: usize) -> SbiRet {
    match fid {
        fid::SRST_SHUTDOWN => {
            // QEMU virt machine test device
            const VIRT_TEST: usize = 0x10_0000;
            const EXIT_SUCCESS: u32 = 0x5555;
            unsafe {
                core::ptr::write_volatile(VIRT_TEST as *mut u32, EXIT_SUCCESS);
            }
            loop {
                unsafe { asm!("wfi"); }
            }
        }
        _ => {
            // Reset
            const VIRT_TEST: usize = 0x10_0000;
            const EXIT_RESET: u32 = 0x7777;
            unsafe {
                core::ptr::write_volatile(VIRT_TEST as *mut u32, EXIT_RESET);
            }
            loop {
                unsafe { asm!("wfi"); }
            }
        }
    }
}

/// Handle legacy shutdown
fn handle_legacy_shutdown() -> SbiRet {
    const VIRT_TEST: usize = 0x10_0000;
    const EXIT_SUCCESS: u32 = 0x5555;
    unsafe {
        core::ptr::write_volatile(VIRT_TEST as *mut u32, EXIT_SUCCESS);
    }
    loop {
        unsafe { asm!("wfi"); }
    }
}

/// M-Mode trap handler - called from assembly
/// Note: For legacy SBI calls, eid contains the function number (a7)
/// For new SBI calls, eid is the extension ID and fid is the function ID
#[unsafe(no_mangle)]
pub fn m_trap_handler(
    a0: usize,
    a1: usize,
    a2: usize,
    _a3: usize,
    _a4: usize,
    _a5: usize,
    eid: usize,
    fid: usize,
) -> SbiRet {
    // Check for legacy SBI calls first (eid is actually the function number for legacy calls)
    match eid {
        // Legacy SBI calls
        legacy::CONSOLE_PUTCHAR => {
            uart_putchar(a0 as u8);
            SbiRet::success(0)
        }
        legacy::CONSOLE_GETCHAR => {
            if let Some(c) = uart_getchar() {
                SbiRet::success(c as usize)
            } else {
                SbiRet::success(usize::MAX)
            }
        }
        legacy::SHUTDOWN => handle_legacy_shutdown(),
        // New SBI extension calls
        eid::BASE => handle_base(fid, a0, a1, a2),
        eid::TIMER => {
            #[cfg(target_pointer_width = "64")]
            let time = a0 as u64;
            #[cfg(target_pointer_width = "32")]
            let time = (a0 as u64) | ((a1 as u64) << 32);
            handle_timer(time)
        }
        eid::SRST => handle_srst(fid, a0, a1, a2),
        _ => SbiRet::not_supported(),
    }
}
