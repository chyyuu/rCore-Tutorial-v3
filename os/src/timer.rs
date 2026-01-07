use core::cmp::Ordering;

use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use crate::sync::UPSafeCell;
use crate::task::{TaskControlBlock, wakeup_task};
use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use core::arch::asm;
use lazy_static::*;

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

pub struct TimerCondVar {
    pub expire_ms: usize,
    pub task: Arc<TaskControlBlock>,
}

impl PartialEq for TimerCondVar {
    fn eq(&self, other: &Self) -> bool {
        self.expire_ms == other.expire_ms
    }
}
impl Eq for TimerCondVar {}
impl PartialOrd for TimerCondVar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let a = -(self.expire_ms as isize);
        let b = -(other.expire_ms as isize);
        Some(a.cmp(&b))
    }
}

impl Ord for TimerCondVar {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

lazy_static! {
    static ref TIMERS: UPSafeCell<BinaryHeap<TimerCondVar>> =
        unsafe { UPSafeCell::new(BinaryHeap::<TimerCondVar>::new()) };
}

pub fn add_timer(expire_ms: usize, task: Arc<TaskControlBlock>) {
    let mut timers = TIMERS.exclusive_access();
    timers.push(TimerCondVar { expire_ms, task });
}

pub fn remove_timer(task: Arc<TaskControlBlock>) {
    let mut timers = TIMERS.exclusive_access();
    let mut temp = BinaryHeap::<TimerCondVar>::new();
    for condvar in timers.drain() {
        if Arc::as_ptr(&task) != Arc::as_ptr(&condvar.task) {
            temp.push(condvar);
        }
    }
    timers.clear();
    timers.append(&mut temp);
}

pub fn check_timer() {
    let current_ms = get_time_ms() as usize;
    let mut timers = TIMERS.exclusive_access();
    while let Some(timer) = timers.peek() {
        if timer.expire_ms <= current_ms {
            wakeup_task(Arc::clone(&timer.task));
            timers.pop();
        } else {
            break;
        }
    }
}
