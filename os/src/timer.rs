//! Timer module for getting system time
//!
//! Uses RISC-V time CSR or CLINT mtime register
//! Supports both RV32 and RV64

#![allow(dead_code)]

/// QEMU virt platform clock frequency (10 MHz)
const CLOCK_FREQ: u64 = 10_000_000;

/// Get current time in clock cycles
#[inline]
pub fn get_time() -> u64 {
    #[cfg(target_pointer_width = "64")]
    {
        let time: u64;
        unsafe {
            core::arch::asm!("rdtime {}", out(reg) time);
        }
        time
    }
    
    #[cfg(target_pointer_width = "32")]
    {
        // RV32: need to read timeh and time separately
        // and handle the case where time overflows between reads
        loop {
            let hi: u32;
            let lo: u32;
            let hi2: u32;
            unsafe {
                core::arch::asm!("rdtimeh {}", out(reg) hi);
                core::arch::asm!("rdtime {}", out(reg) lo);
                core::arch::asm!("rdtimeh {}", out(reg) hi2);
            }
            // If hi changed between reads, try again
            if hi == hi2 {
                return ((hi as u64) << 32) | (lo as u64);
            }
        }
    }
}

/// Get current time in milliseconds
#[inline]
pub fn get_time_ms() -> u64 {
    get_time() / (CLOCK_FREQ / 1000)
}

/// Get current time in microseconds
#[inline]
pub fn get_time_us() -> u64 {
    get_time() / (CLOCK_FREQ / 1_000_000)
}
