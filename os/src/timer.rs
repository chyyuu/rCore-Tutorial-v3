//! Timer module for getting system time
//!
//! Uses RISC-V time CSR or CLINT mtime register

#![allow(dead_code)]

/// QEMU virt platform clock frequency (10 MHz)
const CLOCK_FREQ: u64 = 10_000_000;

/// Get current time in clock cycles
#[inline]
pub fn get_time() -> u64 {
    let time: u64;
    unsafe {
        core::arch::asm!("rdtime {}", out(reg) time);
    }
    time
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

