//! M-Mode SBI implementation for -bios none boot
//!
//! This module provides a minimal SBI implementation that runs in M-Mode,
//! allowing the kernel to boot without an external bootloader like RustSBI.

use core::arch::asm;
use crate::board::QEMUExit;

const UART_BASE: usize = 0x1000_0000;

#[repr(C)]
pub struct SbiRet {
    pub error: usize,
    pub value: usize,
}

impl SbiRet {
    pub fn success(value: usize) -> Self {
        Self { error: 0, value }
    }
    pub fn not_supported() -> Self {
        Self { error: usize::MAX, value: 0 }
    }
}

mod eid {
    pub const BASE: usize = 0x10;
    pub const TIMER: usize = 0x54494D45;
    pub const CONSOLE: usize = 0x01;
    pub const CONSOLE_GETCHAR: usize = 0x02;
    pub const SRST: usize = 0x53525354;
}

mod fid {
    pub const BASE_GET_SBI_VERSION: usize = 0;
    pub const BASE_GET_IMPL_ID: usize = 1;
    pub const BASE_GET_IMPL_VERSION: usize = 2;
    pub const BASE_PROBE_EXTENSION: usize = 3;
    pub const BASE_GET_MVENDORID: usize = 4;
    pub const BASE_GET_MARCHID: usize = 5;
    pub const BASE_GET_MIMPID: usize = 6;

    pub const CONSOLE_PUTCHAR: usize = 0;
    #[allow(dead_code)]
    pub const CONSOLE_GETCHAR: usize = 0;

    pub const SRST_SHUTDOWN: usize = 0;
}

fn uart_putchar(c: u8) {
    unsafe {
        (UART_BASE as *mut u8).write_volatile(c);
    }
}

fn uart_getchar() -> Option<u8> {
    let c = unsafe { ((UART_BASE + 5) as *mut u8).read_volatile() };
    if c & 0x01 != 0 {
        Some(unsafe { (UART_BASE as *mut u8).read_volatile() })
    } else {
        None
    }
}

fn handle_base(fid: usize, _a0: usize, _a1: usize, _a2: usize) -> SbiRet {
    match fid {
        fid::BASE_GET_SBI_VERSION => SbiRet::success(0x01000000), // SBI v1.0.0
        fid::BASE_GET_IMPL_ID => SbiRet::success(1), // rCore-Tutorial-v3
        fid::BASE_GET_IMPL_VERSION => SbiRet::success(1),
        fid::BASE_PROBE_EXTENSION => SbiRet::success(1), // All extensions supported
        fid::BASE_GET_MVENDORID => SbiRet::success(0),
        fid::BASE_GET_MARCHID => SbiRet::success(0),
        fid::BASE_GET_MIMPID => SbiRet::success(0),
        _ => SbiRet::not_supported(),
    }
}

fn handle_console(fid: usize, a0: usize, _a1: usize, _a2: usize) -> SbiRet {
    match fid {
        fid::CONSOLE_PUTCHAR => {
            uart_putchar(a0 as u8);
            SbiRet::success(0)
        }
        _ => SbiRet::not_supported(),
    }
}

fn handle_console_getchar() -> SbiRet {
    if let Some(c) = uart_getchar() {
        SbiRet::success(c as usize)
    } else {
        SbiRet::success(usize::MAX) // No character available
    }
}

fn handle_timer(a0: usize, _a1: usize) -> SbiRet {
    const CLINT_MTIMECMP: usize = 0x200_4000;
    
    #[cfg(target_pointer_width = "64")]
    {
        let time = a0 as u64;
        unsafe {
            (CLINT_MTIMECMP as *mut u64).write_volatile(time);
        }
    }
    
    #[cfg(target_pointer_width = "32")]
    {
        let time = (a0 as u64) | ((_a1 as u64) << 32);
        unsafe {
            (CLINT_MTIMECMP as *mut u32).write_volatile(time as u32);
            ((CLINT_MTIMECMP + 4) as *mut u32).write_volatile((time >> 32) as u32);
        }
    }
    
    // Clear pending timer interrupt
    unsafe {
        asm!(
            "csrc mip, {}",
            in(reg) (1 << 5), // Clear STIP by clearing MTIP
        );
    }
    SbiRet::success(0)
}

fn handle_srst(fid: usize, _a0: usize, _a1: usize, _a2: usize) -> ! {
    match fid {
        fid::SRST_SHUTDOWN => {
            // QEMU exit
            crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
        _ => {
            crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
    }
}

/// M-Mode trap handler
/// Arguments passed in registers:
/// a0-a5: SBI call arguments
/// a6: FID (function ID)
/// a7: EID (extension ID)
#[unsafe(no_mangle)]
pub fn m_trap_handler(
    a0: usize, 
    a1: usize, 
    a2: usize, 
    _a3: usize, 
    _a4: usize, 
    _a5: usize, 
    fid: usize,  // a6
    eid: usize,  // a7
) -> SbiRet {
    match eid {
        eid::BASE => handle_base(fid, a0, a1, a2),
        eid::CONSOLE => handle_console(fid, a0, a1, a2),
        eid::CONSOLE_GETCHAR => handle_console_getchar(),
        eid::TIMER => handle_timer(a0, a1),
        eid::SRST => handle_srst(fid, a0, a1, a2),
        _ => SbiRet::not_supported(),
    }
}
