//! Minimal M-Mode SBI implementation for -bios none boot
//!
//! This module provides basic SBI services when running without an external
//! bootloader like RustSBI. It handles ecalls from S-mode and provides:
//! - Console I/O (UART)
//! - Timer management
//! - System reset

use core::arch::asm;
use crate::board::QEMUExit;

const UART_BASE: usize = 0x1000_0000;

/// SBI return value structure
#[repr(C)]
pub struct SbiRet {
    /// Error code (0 = success)
    pub error: usize,
    /// Return value
    pub value: usize,
}

impl SbiRet {
    /// Create a successful return value
    pub fn success(value: usize) -> Self {
        Self { error: 0, value }
    }
    /// Create a "not supported" return value
    pub fn not_supported() -> Self {
        Self { error: usize::MAX, value: 0 }
    }
}

/// SBI Extension IDs
mod eid {
    pub const BASE: usize = 0x10;
    pub const TIMER: usize = 0x54494D45;
    pub const CONSOLE_PUTCHAR: usize = 0x01;
    pub const CONSOLE_GETCHAR: usize = 0x02;
    pub const SRST: usize = 0x53525354;
}

/// SBI Function IDs
mod fid {
    pub const BASE_GET_SBI_VERSION: usize = 0;
    pub const BASE_GET_IMPL_ID: usize = 1;
    pub const BASE_GET_IMPL_VERSION: usize = 2;
    pub const BASE_PROBE_EXTENSION: usize = 3;
    pub const BASE_GET_MVENDORID: usize = 4;
    pub const BASE_GET_MARCHID: usize = 5;
    pub const BASE_GET_MIMPID: usize = 6;

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
    let lsr = unsafe { ((UART_BASE + 5) as *const u8).read_volatile() };
    if lsr & 1 != 0 {
        Some(unsafe { (UART_BASE as *const u8).read_volatile() })
    } else {
        None
    }
}

/// Handle SBI Base extension calls
fn handle_base(fid: usize, _a0: usize, _a1: usize, _a2: usize) -> SbiRet {
    match fid {
        fid::BASE_GET_SBI_VERSION => SbiRet::success(0x01000000), // SBI v1.0.0
        fid::BASE_GET_IMPL_ID => SbiRet::success(0xFFFF), // Custom implementation
        fid::BASE_GET_IMPL_VERSION => SbiRet::success(1),
        fid::BASE_PROBE_EXTENSION => SbiRet::success(1), // All extensions supported
        fid::BASE_GET_MVENDORID => SbiRet::success(0),
        fid::BASE_GET_MARCHID => SbiRet::success(0),
        fid::BASE_GET_MIMPID => SbiRet::success(0),
        _ => SbiRet::not_supported(),
    }
}

/// Handle console putchar (legacy call)
fn handle_console_putchar(c: usize) -> SbiRet {
    uart_putchar(c as u8);
    SbiRet::success(0)
}

/// Handle console getchar (legacy call)
fn handle_console_getchar() -> SbiRet {
    if let Some(c) = uart_getchar() {
        SbiRet::success(c as usize)
    } else {
        SbiRet::success(usize::MAX) // No character available
    }
}

/// Handle timer extension
fn handle_timer(time: u64) -> SbiRet {
    const CLINT_MTIMECMP: usize = 0x200_4000;
    unsafe {
        #[cfg(target_pointer_width = "64")]
        (CLINT_MTIMECMP as *mut u64).write_volatile(time);
        #[cfg(target_pointer_width = "32")]
        {
            // For RV32, write low 32 bits first, then high 32 bits
            (CLINT_MTIMECMP as *mut u32).write_volatile(time as u32);
            ((CLINT_MTIMECMP + 4) as *mut u32).write_volatile((time >> 32) as u32);
        }
    }
    // Clear pending timer interrupt by setting mtimecmp
    unsafe {
        asm!(
            "csrc mip, {}",
            in(reg) (1 << 5), // Clear STIP
        );
    }
    SbiRet::success(0)
}

/// Handle system reset extension
fn handle_srst(fid: usize, _a0: usize, _a1: usize, _a2: usize) -> SbiRet {
    match fid {
        fid::SRST_SHUTDOWN => {
            crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
        _ => {
            crate::board::QEMU_EXIT_HANDLE.exit_reset();
        }
    }
}

/// Main M-mode trap handler called from assembly
///
/// This function handles ecalls from S-mode and dispatches them to the
/// appropriate handler based on the extension ID and function ID.
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
    // Check for legacy SBI calls (EID < 0x10)
    // Legacy calls use a7 as the function ID directly
    if eid < 0x10 {
        match eid {
            // Legacy Console Putchar (SBI_CONSOLE_PUTCHAR = 1)
            1 => handle_console_putchar(a0),
            // Legacy Console Getchar (SBI_CONSOLE_GETCHAR = 2)
            2 => handle_console_getchar(),
            // Legacy Set Timer (SBI_SET_TIMER = 0)
            0 => {
                #[cfg(target_pointer_width = "64")]
                let time = a0 as u64;
                #[cfg(target_pointer_width = "32")]
                let time = (a0 as u64) | ((a1 as u64) << 32);
                handle_timer(time)
            }
            // Legacy Shutdown (SBI_SHUTDOWN = 8)
            8 => handle_srst(fid::SRST_SHUTDOWN, a0, a1, a2),
            _ => SbiRet::not_supported(),
        }
    } else {
        // Handle modern SBI extensions
        match eid {
            eid::BASE => handle_base(fid, a0, a1, a2),
            eid::CONSOLE_PUTCHAR => handle_console_putchar(a0),
            eid::CONSOLE_GETCHAR => handle_console_getchar(),
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
}


