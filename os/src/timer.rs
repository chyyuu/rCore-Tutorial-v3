use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use core::arch::asm;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;

/// Read the `mtime` register - returns 64-bit time value
#[inline]
pub fn get_time() -> u64 {
    #[cfg(target_pointer_width = "64")]
    {
        let time: u64;
        unsafe {
            asm!("rdtime {}", out(reg) time);
        }
        time
    }
    #[cfg(target_pointer_width = "32")]
    {
        // For RV32, we need to read the 64-bit time in two parts
        let mut time_high: u32;
        let mut time_low: u32;
        unsafe {
            loop {
                asm!("rdtimeh {}", out(reg) time_high);
                asm!("rdtime {}", out(reg) time_low);
                let time_high2: u32;
                asm!("rdtimeh {}", out(reg) time_high2);
                if time_high == time_high2 {
                    break;
                }
            }
        }
        ((time_high as u64) << 32) | (time_low as u64)
    }
}

/// Get current time in milliseconds
#[allow(dead_code)]
pub fn get_time_ms() -> u64 {
    get_time() / (CLOCK_FREQ as u64 / MSEC_PER_SEC as u64)
}

/// Get current time in microseconds
#[allow(dead_code)]
pub fn get_time_us() -> u64 {
    get_time() / (CLOCK_FREQ as u64 / (MSEC_PER_SEC * 1000) as u64)
}

/// Set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ as u64 / TICKS_PER_SEC as u64);
}
