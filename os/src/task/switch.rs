use super::TaskContext;
use core::arch::global_asm;

#[cfg(target_pointer_width = "64")]
global_asm!(include_str!("switch.S"));
#[cfg(target_pointer_width = "32")]
global_asm!(include_str!("switch_rv32.S"));

unsafe extern "C" {
    /// Switch to the context of `next_task_cx_ptr`, saving the current context
    /// in `current_task_cx_ptr`.
    pub unsafe fn __switch(
        current_task_cx_ptr: *mut TaskContext,
        next_task_cx_ptr: *const TaskContext,
    );
}
