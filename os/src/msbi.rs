//! M-Mode SBI implementation
//! 
//! This module provides a minimal SBI implementation for M-Mode,
//! handling ecalls from S-Mode.

/// QEMU virt UART base address
const UART_BASE: usize = 0x1000_0000;

/// UART registers (16550 compatible)
mod uart {
    use super::UART_BASE;
    
    const THR: usize = UART_BASE + 0; // Transmit Holding Register
    const LSR: usize = UART_BASE + 5; // Line Status Register
    
    /// Check if UART is ready to transmit
    #[inline]
    fn is_tx_ready() -> bool {
        unsafe {
            let lsr = (LSR as *const u8).read_volatile();
            (lsr & 0x20) != 0 // Check THRE bit
        }
    }
    
    /// Write a byte to UART
    pub fn putchar(c: u8) {
        // Wait for UART to be ready
        while !is_tx_ready() {}
        unsafe {
            (THR as *mut u8).write_volatile(c);
        }
    }
    
    /// Read a byte from UART (non-blocking)
    pub fn getchar() -> Option<u8> {
        unsafe {
            let lsr = (LSR as *const u8).read_volatile();
            if (lsr & 0x01) != 0 { // Data Ready
                Some((UART_BASE as *const u8).read_volatile())
            } else {
                None
            }
        }
    }
}

/// SBI Extension IDs
mod eid {
    pub const LEGACY_CONSOLE_PUTCHAR: usize = 0x1;
    pub const LEGACY_CONSOLE_GETCHAR: usize = 0x2;
    pub const LEGACY_SHUTDOWN: usize = 0x8;
    
    pub const BASE: usize = 0x10;
    pub const TIMER: usize = 0x54494D45;
    pub const SRST: usize = 0x53525354;
}

/// SBI error codes
#[allow(dead_code)]
mod error {
    pub const SUCCESS: isize = 0;
    pub const ERR_FAILED: isize = -1;
    pub const ERR_NOT_SUPPORTED: isize = -2;
    pub const ERR_INVALID_PARAM: isize = -3;
}

/// SBI return value structure
#[repr(C)]
pub struct SbiRet {
    pub error: isize,
    pub value: usize,
}

impl SbiRet {
    fn success(value: usize) -> Self {
        SbiRet { error: error::SUCCESS, value }
    }
    
    fn not_supported() -> Self {
        SbiRet { error: error::ERR_NOT_SUPPORTED, value: 0 }
    }
}

/// Handle legacy console putchar (EID 0x01)
fn handle_console_putchar(c: usize) -> SbiRet {
    uart::putchar(c as u8);
    SbiRet::success(0)
}

/// Handle legacy console getchar (EID 0x02)
fn handle_console_getchar() -> SbiRet {
    match uart::getchar() {
        Some(c) => SbiRet::success(c as usize),
        None => SbiRet::success(usize::MAX), // -1 in legacy interface means no char
    }
}

/// Handle system reset (SRST extension)
fn handle_system_reset(_reset_type: usize, reset_reason: usize) -> SbiRet {
    // For QEMU, we use the test device to shutdown
    const VIRT_TEST: usize = 0x100000;
    const FINISHER_PASS: u32 = 0x5555;
    const FINISHER_FAIL: u32 = 0x3333;
    
    let code = if reset_reason == 0 {
        FINISHER_PASS
    } else {
        FINISHER_FAIL
    };
    
    unsafe {
        // QEMU virt test device
        (VIRT_TEST as *mut u32).write_volatile(code);
    }
    
    // Should not reach here
    loop {}
}

/// Handle legacy shutdown (EID 0x08)
fn handle_legacy_shutdown() -> SbiRet {
    handle_system_reset(0, 0)
}

/// Handle SBI base extension (EID 0x10)
fn handle_base(fid: usize) -> SbiRet {
    match fid {
        0 => SbiRet::success(2), // get_spec_version: SBI 0.2
        1 => SbiRet::success(0), // get_impl_id: custom
        2 => SbiRet::success(1), // get_impl_version
        3 => {
            // probe_extension: check if extension is supported
            SbiRet::success(1) // We support legacy extensions
        }
        4 => SbiRet::success(0), // get_mvendorid
        5 => SbiRet::success(0), // get_marchid
        6 => SbiRet::success(0), // get_mimpid
        _ => SbiRet::not_supported(),
    }
}

/// Handle timer extension (EID 0x54494D45)
fn handle_timer(time: u64) -> SbiRet {
    // Set mtimecmp for the timer interrupt
    const CLINT_MTIMECMP: usize = 0x200_4000;
    unsafe {
        (CLINT_MTIMECMP as *mut u64).write_volatile(time);
    }
    // Clear pending timer interrupt
    unsafe {
        core::arch::asm!(
            "csrc mip, {}", 
            in(reg) (1 << 5), // Clear STIP by clearing MTIP
        );
    }
    SbiRet::success(0)
}

/// M-Mode trap handler called from assembly
/// 
/// Arguments are passed in registers:
/// - a0-a5: SBI call arguments
/// - a6: FID (function ID)
/// - a7: EID (extension ID)
/// 
/// Returns (error, value) in a0, a1
#[unsafe(no_mangle)]
pub extern "C" fn m_trap_handler(
    a0: usize,
    a1: usize,
    _a2: usize,
    _a3: usize,
    _a4: usize,
    _a5: usize,
    fid: usize,
    eid: usize,
) -> SbiRet {
    // Check mcause - we only handle ecall from S-Mode (cause = 9)
    let mcause: usize;
    unsafe {
        core::arch::asm!("csrr {}, mcause", out(reg) mcause);
    }
    
    // Exception code 9 = Environment call from S-mode
    if mcause != 9 {
        // For now, just return error for other exceptions
        return SbiRet::not_supported();
    }
    
    // Handle SBI call based on EID
    match eid {
        // Legacy extensions (SBI v0.1)
        eid::LEGACY_CONSOLE_PUTCHAR => handle_console_putchar(a0),
        eid::LEGACY_CONSOLE_GETCHAR => handle_console_getchar(),
        eid::LEGACY_SHUTDOWN => handle_legacy_shutdown(),
        
        // Base extension (SBI v0.2+)
        eid::BASE => handle_base(fid),
        
        // Timer extension
        eid::TIMER => handle_timer(a0 as u64 | ((a1 as u64) << 32)),
        
        // System Reset extension
        eid::SRST => {
            if fid == 0 {
                handle_system_reset(a0, a1)
            } else {
                SbiRet::not_supported()
            }
        }
        
        // Unsupported extensions
        _ => SbiRet::not_supported(),
    }
}
