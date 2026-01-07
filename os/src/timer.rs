//! RISC-V timer-related functionality
//!
//! Supports both RV32 and RV64

use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;

/// read the `mtime` register (supports RV32 and RV64)
#[inline]
pub fn get_time() -> usize {
    #[cfg(target_pointer_width = "64")]
    {
        let time: usize;
        unsafe {
            core::arch::asm!("rdtime {}", out(reg) time);
        }
        time
    }
    
    #[cfg(target_pointer_width = "32")]
    {
        // RV32: read only lower 32 bits for simplicity
        // For accurate 64-bit time, we would need to handle overflow
        let time: usize;
        unsafe {
            core::arch::asm!("rdtime {}", out(reg) time);
        }
        time
    }
}

/// get current time in milliseconds
#[inline]
pub fn get_time_ms() -> usize {
    get_time() / (CLOCK_FREQ / MSEC_PER_SEC)
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
