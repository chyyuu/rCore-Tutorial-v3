//! Wrap `switch.S` as a function

use super::TaskContext;
use core::arch::global_asm;

// Include architecture-specific switch assembly
#[cfg(target_pointer_width = "64")]
global_asm!(include_str!("switch_rv64.S"));
#[cfg(target_pointer_width = "32")]
global_asm!(include_str!("switch_rv32.S"));

unsafe extern "C" {
    pub safe fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
