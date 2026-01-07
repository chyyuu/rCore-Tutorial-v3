//! Rust wrapper around `__switch`.
//!
//! Switching to a different task's context happens here. The actual
//! implementation must not be in Rust and (essentially) has to be in assembly
//! language (Do you know why?), so this module really is just a wrapper around
//! `switch.S`.

use super::TaskContext;
use core::arch::global_asm;

#[cfg(target_pointer_width = "64")]
global_asm!(include_str!("switch_rv64.S"));

#[cfg(target_pointer_width = "32")]
global_asm!(include_str!("switch_rv32.S"));

unsafe extern "C" {
    /// Switch to the context of `next_task_cx_ptr`, saving the current context
    /// in `current_task_cx_ptr`.
    pub safe fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
